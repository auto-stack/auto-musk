//! Plan → Spec 合并引擎（PLAN-024 §5.3 / 008 §6.5 落地）。
//!
//! 把一个 `review_done` 的 Plan 拆解成知识片段，映射写入 Spec 6 区 ledger
//! 的对应 section（goals/architecture/designs/tests/reviews/reports）。每个
//! 生成的 [`SpecItem`] 用 `file`（来源 plan 路径）+ `related`（`PLAN-NNN`）
//! 溯源。
//!
//! 本模块是**纯逻辑**（输入 [`PlanFile`]，输出 `(section_id, SpecItem)` 列表
//! + [`MergeResult`]），不触碰磁盘；IO（upsert/save/archive/transition）由
//! `plans::plans_merge` handler 负责，便于单元测试。

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::plans::PlanFile;
use crate::specs::{SectionType, SpecItem, SpecStatus, SpecsDocument};

/// merge 结果（handler 返回给前端）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MergeResult {
    pub plan_id: String,
    /// 触及的 spec section id 列表（如 ["goals", "architecture", ...]）。
    pub sections_touched: Vec<String>,
    /// 生成的 SpecItem 数量。
    pub items_created: usize,
}

/// Plan 正文章节编号 → Spec section id 映射（008 §6.5 / PLAN-024 §5.3）。
///
/// | Plan 章节 | Spec section |
/// |:---|:---|
/// | §0 变更摘要 | reports |
/// | §1 目标 | goals |
/// | §2 架构方案 | architecture |
/// | §5 详细设计 | designs |
/// | §6 测试设计 | tests |
/// | §7 验收标准 / §9 复审 | reviews |
///
/// 其余章节（§3 技术栈 / §4 需求分析 / §8 执行步骤 / §10 待澄清）不映射 ——
/// 它们是过程性信息，归档后的 Plan 文件本身保留。
fn section_id_for(num: u32) -> Option<&'static str> {
    match num {
        0 => Some("reports"),
        1 => Some("goals"),
        2 => Some("architecture"),
        5 => Some("designs"),
        6 => Some("tests"),
        7 | 9 => Some("reviews"),
        _ => None,
    }
}

/// merge 生成的 SpecItem 的合理初值状态（按 section 类型）。
fn status_for_section(section_id: &str) -> SpecStatus {
    match section_id {
        "goals" => SpecStatus::Proposed,
        "architecture" | "designs" => SpecStatus::Stable,
        "tests" => SpecStatus::Verified,
        "reviews" | "reports" => SpecStatus::Published,
        _ => SpecStatus::Empty,
    }
}

/// 无编号中文标题 → 章节编号的 canonical 映射（PLAN-030 T1）。
///
/// 028/029 起的新式 plan 用 `## 变更摘要` 等无编号标题（008 §4.2 模板则带
/// `## N.` 编号）。按标题前缀匹配识别，容忍 `## 目标 (Goal)` 这类带英文注记
/// 的变体。
fn number_for_title(title: &str) -> Option<u32> {
    const CANONICAL: [(&str, u32); 11] = [
        ("变更摘要", 0),
        ("目标", 1),
        ("架构方案", 2),
        ("技术栈", 3),
        ("需求分析", 4),
        ("详细设计", 5),
        ("测试设计", 6),
        ("验收标准", 7),
        ("执行步骤", 8),
        ("复审记录", 9),
        ("待澄清", 10),
    ];
    let t = title.trim();
    CANONICAL
        .iter()
        .find(|(key, _)| t.starts_with(key))
        .map(|(_, num)| *num)
}

/// 从 Plan 正文提取章节，返回 (编号, 标题, 正文) 列表。
///
/// 兼容两种标题：`## N. 标题`（008 §4.2 模板）与 `## 标题`（028/029 起的无编号
/// 中文，按 canonical 名称表识别）。非 canonical 的无编号 `##` 标题不算章节；
/// 正文范围取相邻两个**已识别**章节之间（到文件末尾封底）。
fn extract_sections(content: &str) -> Vec<(u32, String, String)> {
    let re = Regex::new(r"(?m)^##\s+(?:(\d+)\.\s*)?(.+?)\s*$").unwrap();
    // 第一遍：收集已识别章节的 (编号, 标题, 标题行结束位置, 标题行起始位置)。
    let mut matched: Vec<(u32, String, usize, usize)> = Vec::new();
    for cap in re.captures_iter(content) {
        let title = cap[2].trim().to_string();
        let num = match cap.get(1) {
            Some(digits) => digits.as_str().parse().unwrap_or(u32::MAX),
            None => match number_for_title(&title) {
                Some(n) => n,
                None => continue,
            },
        };
        matched.push((
            num,
            title,
            cap.get(0).unwrap().end(),
            cap.get(0).unwrap().start(),
        ));
    }
    // 第二遍：按相邻已识别章节切正文。
    let mut out = Vec::with_capacity(matched.len());
    for (i, (num, title, head_end, _)) in matched.iter().enumerate() {
        let end = if i + 1 < matched.len() {
            matched[i + 1].3
        } else {
            content.len()
        };
        let body = content[*head_end..end].trim().to_string();
        out.push((*num, title.clone(), body));
    }
    out
}

/// 把一个 Plan 拆解成 `(section_id, SpecItem)` 列表 + `MergeResult`。
///
/// 每个 item：
/// - `id` = `P{seq:03}-{n}`（如 `P024-1`），便于溯源且不与既有 spec id 冲突。
/// - `title` = `{plan.feature_name} — {章节标题}`。
/// - `content` = 章节正文 markdown 原文。
/// - `file` = `Some("docs/plans/{filename}")`，溯源到 plan。
/// - `related` = `[plan.id]`（`PLAN-NNN`），反链。
/// - `status` = 按 section 类型给合理初值（如 architecture → Stable）。
///
/// 同一 plan 重复 merge 时 item id 稳定（`P{seq}-{n}`），配合 upsert 幂等 ——
/// 不重复写 item。
pub fn plan_to_items(plan: &PlanFile) -> (Vec<(String, SpecItem)>, MergeResult) {
    let sections = extract_sections(&plan.content);
    let mut items: Vec<(String, SpecItem)> = Vec::new();
    let mut touched: Vec<String> = Vec::new();
    let mut n = 1usize;
    let source = format!("docs/plans/{}", plan.filename);
    for (num, title, body) in sections {
        let Some(section_id) = section_id_for(num) else {
            continue;
        };
        let id = format!("P{:03}-{}", plan.seq, n);
        n += 1;
        let mut item = SpecItem::new(id.clone(), format!("{} — {}", plan.feature_name, title));
        item.content = body;
        item.file = Some(source.clone());
        item.related = vec![plan.id.clone()];
        item.status = status_for_section(section_id);
        if !touched.contains(&section_id.to_string()) {
            touched.push(section_id.to_string());
        }
        items.push((section_id.to_string(), item));
    }
    let result = MergeResult {
        plan_id: plan.id.clone(),
        sections_touched: touched,
        items_created: items.len(),
    };
    (items, result)
}

/// 把 merge items upsert 进 SpecsDocument（内存操作，不 save）。
/// 同 id 的 item 替换，否则追加。幂等。
pub fn upsert_items_into_doc(doc: &mut SpecsDocument, items: Vec<(String, SpecItem)>) {
    for (section_id, item) in items {
        if let Some(section) = doc.sections.iter_mut().find(|s| s.id == section_id) {
            if let Some(existing) = section.items.iter_mut().find(|i| i.id == item.id) {
                *existing = item;
            } else {
                section.items.push(item);
            }
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plans::PlanStatus;

    fn sample_plan() -> PlanFile {
        let content = "\
---
plan_id: PLAN-042
status: review_done
feature_name: 新语法解析
created_at: 2026-08-11T10:00:00Z
updated_at: 2026-08-11T16:00:00Z
---

# [PLAN-042] 新语法解析 - 实施计划

## 0. 变更摘要

新增 parser 模块支持 X 语法。

## 1. 目标 (Goal)

支持把 X 语法解析为 AST。

## 2. 架构方案 (Architecture)

lexer → parser → ast_builder 三层。

## 3. 技术栈 (Tech Stack)

Rust + nom.

## 4. 需求分析与背景调查

现有 parser 不支持 X。

## 5. 详细设计 (Detailed Design)

### 5.1 grammar

新增产生式 X.

## 6. 测试设计 (Test Design)

正向 + 负向用例。

## 7. 验收标准 (Acceptance Criteria)

- [ ] X 能解析为 AST

## 8. 执行步骤 (Execution Tasks)

### 任务 1: ...

## 9. 复审记录 (Review Log)

复审通过。

## 10. 待澄清事项 (Open Questions)

无。
";
        PlanFile {
            id: "PLAN-042".into(),
            seq: 42,
            filename: "042-new-syntax.md".into(),
            status: PlanStatus::Reviewed,
            feature_name: "新语法解析".into(),
            title: "新语法解析 - 实施计划".into(),
            archived: false,
            content: content.into(),
            created_at: "2026-08-11T10:00:00Z".into(),
            updated_at: "2026-08-11T16:00:00Z".into(),
            path: "042-new-syntax.md".into(),
        }
    }

    #[test]
    fn extract_sections_finds_all_numbered_sections() {
        let plan = sample_plan();
        let secs = extract_sections(&plan.content);
        let nums: Vec<u32> = secs.iter().map(|(n, _, _)| *n).collect();
        assert_eq!(nums, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn extract_sections_body_excludes_next_heading() {
        let plan = sample_plan();
        let secs = extract_sections(&plan.content);
        let goals = secs.iter().find(|(n, _, _)| *n == 1).unwrap();
        assert!(goals.2.contains("支持把 X 语法解析为 AST"));
        // body 不应含下一个章节标题
        assert!(!goals.2.contains("## 2."));
    }

    #[test]
    fn section_id_for_maps_known_numbers() {
        assert_eq!(section_id_for(0), Some("reports"));
        assert_eq!(section_id_for(1), Some("goals"));
        assert_eq!(section_id_for(2), Some("architecture"));
        assert_eq!(section_id_for(5), Some("designs"));
        assert_eq!(section_id_for(6), Some("tests"));
        assert_eq!(section_id_for(7), Some("reviews"));
        assert_eq!(section_id_for(9), Some("reviews"));
    }

    #[test]
    fn section_id_for_skips_process_sections() {
        // §3 §4 §8 §10 是过程性信息，不映射
        assert_eq!(section_id_for(3), None);
        assert_eq!(section_id_for(4), None);
        assert_eq!(section_id_for(8), None);
        assert_eq!(section_id_for(10), None);
    }

    #[test]
    fn plan_to_items_produces_correct_sections() {
        let plan = sample_plan();
        let (items, result) = plan_to_items(&plan);
        // §0,1,2,5,6,7,9 → 7 items（§9 复审也映射 reviews）
        assert_eq!(items.len(), 7);
        let section_ids: Vec<&str> = items.iter().map(|(s, _)| s.as_str()).collect();
        assert!(section_ids.contains(&"goals"));
        assert!(section_ids.contains(&"architecture"));
        assert!(section_ids.contains(&"designs"));
        assert!(section_ids.contains(&"tests"));
        assert!(section_ids.contains(&"reviews"));
        assert!(section_ids.contains(&"reports"));
        // sections_touched 去重（§7+§9 都 → reviews，只算一次）
        assert!(result.sections_touched.len() <= 6);
        assert_eq!(result.items_created, 7);
        assert_eq!(result.plan_id, "PLAN-042");
    }

    #[test]
    fn plan_to_items_item_has_source_file_and_related() {
        let plan = sample_plan();
        let (items, _) = plan_to_items(&plan);
        let (goals_section, goals_item) = items
            .iter()
            .find(|(s, _)| s == "goals")
            .expect("goals item exists");
        assert_eq!(goals_section, "goals");
        assert_eq!(goals_item.file.as_deref(), Some("docs/plans/042-new-syntax.md"));
        assert!(goals_item.related.contains(&"PLAN-042".to_string()));
        // title 含 feature_name
        assert!(goals_item.title.contains("新语法解析"));
        // content 含目标章节正文
        assert!(goals_item.content.contains("支持把 X 语法解析为 AST"));
    }

    #[test]
    fn plan_to_items_ids_are_stable_and_prefixed() {
        let plan = sample_plan();
        let (items, _) = plan_to_items(&plan);
        // 所有 id 形如 P042-{n}
        for (_s, item) in &items {
            assert!(
                item.id.starts_with("P042-"),
                "id {} should start with P042-",
                item.id
            );
        }
        // 重复 merge 同一 plan 产生相同 id（幂等前提）
        let (items2, _) = plan_to_items(&plan);
        let ids1: Vec<&str> = items.iter().map(|(_, i)| i.id.as_str()).collect();
        let ids2: Vec<&str> = items2.iter().map(|(_, i)| i.id.as_str()).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn plan_to_items_status_per_section() {
        let plan = sample_plan();
        let (items, _) = plan_to_items(&plan);
        for (section_id, item) in &items {
            let expected = status_for_section(section_id);
            assert_eq!(item.status, expected, "section {} status", section_id);
        }
        // architecture → Stable
        let arch = items.iter().find(|(s, _)| s == "architecture").unwrap();
        assert_eq!(arch.1.status, SpecStatus::Stable);
    }

    #[test]
    fn upsert_into_doc_adds_items_to_correct_sections() {
        let plan = sample_plan();
        let (items, _) = plan_to_items(&plan);
        let mut doc = SpecsDocument::new("proj");
        let before = doc.sections.iter().map(|s| s.items.len()).sum::<usize>();
        upsert_items_into_doc(&mut doc, items);
        let after = doc.sections.iter().map(|s| s.items.len()).sum::<usize>();
        assert_eq!(after, before + 7);
        // goals section 有一个 item
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 1);
        assert!(goals.items[0].id.starts_with("P042-"));
    }

    #[test]
    fn upsert_into_doc_is_idempotent() {
        let plan = sample_plan();
        let (items, _) = plan_to_items(&plan);
        let mut doc = SpecsDocument::new("proj");
        upsert_items_into_doc(&mut doc, items.clone());
        // 再 merge 一次（相同 id）—— 不应新增 item
        upsert_items_into_doc(&mut doc, items);
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 1, "re-merge must not duplicate");
    }

    #[test]
    fn merge_preserves_unrelated_items() {
        // 既有的 goals item 不应被 merge 碰
        let plan = sample_plan();
        let (items, _) = plan_to_items(&plan);
        let mut doc = SpecsDocument::new("proj");
        let mut existing = SpecItem::new("G1", "existing goal");
        existing.content = "hands off".into();
        doc.sections
            .iter_mut()
            .find(|s| s.id == "goals")
            .unwrap()
            .items
            .push(existing);
        upsert_items_into_doc(&mut doc, items);
        let goals = doc.sections.iter().find(|s| s.id == "goals").unwrap();
        assert_eq!(goals.items.len(), 2); // G1 + P042-x
        assert!(goals.items.iter().any(|i| i.id == "G1"));
        assert!(goals.items.iter().any(|i| i.id.starts_with("P042-")));
    }

    // ── PLAN-030 T1：无编号中文标题兼容 ──────────────────────────────────

    #[test]
    fn extract_sections_supports_unnumbered_chinese_headings() {
        let content = "# [PLAN-031] x\n\n## 变更摘要\n\n摘要内容。\n\n## 目标\n\n目标内容。\n\n## 架构方案\n\n架构内容。\n\n## 技术栈\n\nRust。\n\n## 需求分析与背景调查\n\n背景。\n\n## 详细设计\n\n设计。\n\n## 测试设计\n\n测试。\n\n## 验收标准\n\n- [ ] A1\n\n## 执行步骤\n\n- [ ] T1\n\n## 复审记录\n\n复审通过。\n\n## 待澄清事项\n\n无。\n";
        let secs = extract_sections(content);
        let nums: Vec<u32> = secs.iter().map(|(n, _, _)| *n).collect();
        assert_eq!(nums, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn extract_sections_ignores_non_canonical_level2_headings() {
        // 非 canonical 的无编号 `##` 不算章节；前一章节正文跨越它延伸到下一个
        // canonical 章节。
        let content =
            "## 目标\n\n目标内容。\n\n## 附录A 其他\n\n附内容。\n\n## 架构方案\n\n架构内容。\n";
        let secs = extract_sections(content);
        let nums: Vec<u32> = secs.iter().map(|(n, _, _)| *n).collect();
        assert_eq!(nums, vec![1, 2]);
        let goals = secs.iter().find(|(n, _, _)| *n == 1).unwrap();
        assert!(
            goals.2.contains("附内容"),
            "goals body 应跨越非 canonical 章节延伸"
        );
    }

    #[test]
    fn plan_to_items_maps_unnumbered_plan() {
        // 028/029 风格：无编号中文标题的 plan 也要能拆出 7 个映射 item。
        let mut plan = sample_plan();
        plan.content = plan.content
            .replace("## 0. 变更摘要", "## 变更摘要")
            .replace("## 1. 目标 (Goal)", "## 目标")
            .replace("## 2. 架构方案 (Architecture)", "## 架构方案")
            .replace("## 3. 技术栈 (Tech Stack)", "## 技术栈")
            .replace("## 5. 详细设计 (Detailed Design)", "## 详细设计")
            .replace("## 6. 测试设计 (Test Design)", "## 测试设计")
            .replace("## 7. 验收标准 (Acceptance Criteria)", "## 验收标准")
            .replace("## 8. 执行步骤 (Execution Tasks)", "## 执行步骤")
            .replace("## 9. 复审记录 (Review Log)", "## 复审记录")
            .replace("## 10. 待澄清事项 (Open Questions)", "## 待澄清事项");
        let (items, result) = plan_to_items(&plan);
        assert_eq!(items.len(), 7, "无编号 plan 也必须拆出 7 个映射 item");
        assert_eq!(result.items_created, 7);
    }
}
