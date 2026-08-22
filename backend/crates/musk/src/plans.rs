//! Plan 数据模型 + PlansStore — AutoPlan 架构的动态执行实体。
//!
//! Plan 文件落 `{workspace_root}/docs/plans/`（含 `archived/` 子目录），磁盘
//! 为唯一事实源（仿 [`crate::wiki::WikiStore`] 的目录扫描模式，但不需要
//! manifest —— frontmatter 已含全部元数据）。每个文件 = `NNN-name.md`
//! （3 位序号 + YAML frontmatter + markdown 正文）。
//!
//! 设计文档：`docs/designs/008-auto-plan.md`（定稿 v1.0）。
//! 生命周期状态机见 008 §7.2：drafting → executing → execution_done →
//! reviewed → archived（PLAN-033 单一终态：reviewed 经 merge 沉淀进 Spec 后
//! 归档；非 reviewed 计划可直接 archive 搁置）；沉淀由 [`crate::plan_merge`]
//! 负责。

use crate::specs::now_sec;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================
// PlanStatus — lifecycle (008 §7.2)
// ============================================================

/// Plan 生命周期状态。序列化为 snake_case 字符串，与 008 §4.2 frontmatter
/// 字段值一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// 新建，未执行（`drafting`）。
    Drafting,
    /// `/auto-plan:work` 执行中（`executing`）。
    Executing,
    /// 步骤全部完成（`execution_done`）。
    ExecutionDone,
    /// 复审通过，待 merge（`reviewed`；旧值 `review_done` 兼容读取）。
    Reviewed,
    /// 已终局归档（`archived`，终态；reviewed 经 merge 沉淀后进入，或非
    /// reviewed 计划直接搁置；旧值 `merged` 兼容读取）。
    Archived,
}

impl PlanStatus {
    /// → snake_case 字符串（frontmatter / JSON 字段值）。
    pub fn as_str(self) -> &'static str {
        match self {
            PlanStatus::Drafting => "drafting",
            PlanStatus::Executing => "executing",
            PlanStatus::ExecutionDone => "execution_done",
            PlanStatus::Reviewed => "reviewed",
            PlanStatus::Archived => "archived",
        }
    }

    /// 解析 frontmatter 字符串；未知值降级为 [`PlanStatus::Drafting`]
    /// （旧格式 plan 无 frontmatter 时的兜底）。`review_done` / `merged` 为
    /// PLAN-033 改名前的旧值，读取时映射到新枚举（写盘时自愈）。
    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim() {
            "executing" => PlanStatus::Executing,
            "execution_done" => PlanStatus::ExecutionDone,
            "reviewed" | "review_done" => PlanStatus::Reviewed,
            "archived" | "merged" => PlanStatus::Archived,
            _ => PlanStatus::Drafting,
        }
    }

    /// 状态机合法迁移校验（008 §7.2，PLAN-033 修订）。允许前进 + 审失败
    /// 回退 + 自身幂等。
    ///
    /// 合法路径：
    /// - `Drafting → Executing | Reviewed`
    /// - `Executing → ExecutionDone | Drafting`
    /// - `ExecutionDone → Reviewed | Executing`
    /// - `Reviewed → Executing`（复审不通过回退）
    /// - `Archived` 为终态（仅允许自身）。进入终态不经 transition 端点：
    ///   非 reviewed 计划走 `archive()`（搁置），reviewed 计划走 merge
    ///   沉淀（[`merge_plan_stores`]），两者共用 `move_to_archived`。
    pub fn can_transition(from: Self, to: Self) -> bool {
        if from == to {
            return true; // 幂等
        }
        match (from, to) {
            (PlanStatus::Drafting, PlanStatus::Executing)
            | (PlanStatus::Drafting, PlanStatus::Reviewed)
            | (PlanStatus::Executing, PlanStatus::ExecutionDone)
            | (PlanStatus::Executing, PlanStatus::Drafting)
            | (PlanStatus::ExecutionDone, PlanStatus::Reviewed)
            | (PlanStatus::ExecutionDone, PlanStatus::Executing)
            | (PlanStatus::Reviewed, PlanStatus::Executing) => true,
            _ => false,
        }
    }
}

// ============================================================
// PlanFile — 一个 plan 文件
// ============================================================

/// 一个 Plan 文件（磁盘 `NNN-*.md`）。
///
/// `content` 是完整 markdown（含 frontmatter）；`created_at` / `updated_at`
/// 保留 ISO 原文字符串（frontmatter 权威），其字典序天然等于时间序，便于
/// 不引入时间库即可排序。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanFile {
    /// `PLAN-024`（= 文件名前缀 024，frontmatter `plan_id` 权威）。
    pub id: String,
    /// 3 位序号的数值（24）。
    pub seq: u32,
    /// 文件名（`024-auto-plan-architecture.md`）。
    pub filename: String,
    /// 生命周期状态。
    pub status: PlanStatus,
    /// frontmatter `feature_name`。
    pub feature_name: String,
    /// 正文首行标题（`# [PLAN-024] xxx`），无则空。
    pub title: String,
    /// 是否位于 `archived/` 子目录。
    pub archived: bool,
    /// 完整 markdown（含 frontmatter）。
    pub content: String,
    /// frontmatter `created_at`（ISO 字符串）。
    pub created_at: String,
    /// frontmatter `updated_at`（ISO 字符串）。
    pub updated_at: String,
    /// 相对 `plans_dir` 的路径（`024-xxx.md` 或 `archived/024-xxx.md`）。
    pub path: String,
}

impl PlanFile {
    /// 从磁盘文件读取并解析 frontmatter。`plans_dir` 用于计算相对路径与
    /// `archived` 标志。
    pub fn from_path(path: &Path, plans_dir: &Path) -> Result<Self, String> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("invalid filename: {}", path.display()))?;
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        Self::from_content(content, path, plans_dir)
    }

    /// 解析已读入的 content（便于测试）。`path` / `plans_dir` 仅用于元数据。
    pub fn from_content(content: String, path: &Path, plans_dir: &Path) -> Result<Self, String> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "invalid filename".to_string())?;
        // seq 从文件名 3 位前缀推导
        let seq = filename
            .get(..3)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| format!("filename lacks NNN prefix: {}", filename))?;
        let fm = parse_frontmatter(&content);
        let id = fm
            .get("plan_id")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("PLAN-{:03}", seq));
        let status = fm
            .get("status")
            .map(|s| PlanStatus::from_str_lossy(s))
            .unwrap_or(PlanStatus::Drafting);
        let feature_name = fm.get("feature_name").cloned().unwrap_or_default();
        let created_at = fm.get("created_at").cloned().unwrap_or_default();
        let updated_at = fm.get("updated_at").cloned().unwrap_or_else(|| created_at.clone());
        let title = extract_title(&content);
        let archived_dir = plans_dir.join("archived");
        let archived = path.starts_with(&archived_dir);
        let path_rel = path
            .strip_prefix(plans_dir)
            .ok()
            .and_then(|p| p.to_str())
            .map(|s| s.replace('\\', "/"))
            .unwrap_or_else(|| filename.to_string());
        Ok(Self {
            id,
            seq,
            filename: filename.to_string(),
            status,
            feature_name,
            title,
            archived,
            content,
            created_at,
            updated_at,
            path: path_rel,
        })
    }
}

// ============================================================
// frontmatter 解析辅助（手写轻量，不引入 YAML 依赖）
// ============================================================

/// 解析 frontmatter 顶层 `key: value` 对（跳过列表项 `- ...` 与空行）。
/// 无 frontmatter 时返回空 map。value 去掉行尾 ` # comment` 与包裹引号。
pub fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut lines = content.lines();
    let first = lines.next().unwrap_or("").trim();
    if first != "---" {
        return map;
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if trimmed.starts_with('#') || trimmed.starts_with('-') || trimmed.is_empty() {
            continue; // 注释行 / 列表项 / 空行跳过
        }
        let Some(colon) = line.find(':') else { continue };
        let key = line[..colon].trim().to_string();
        let mut val = line[colon + 1..].trim().to_string();
        if let Some(c) = val.find(" #") {
            val.truncate(c);
            val = val.trim_end().to_string();
        }
        if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
            val = val[1..val.len() - 1].to_string();
        }
        // 跳过空 value（列表/映射头部如 "supersedes_spec_components:"）
        if val.is_empty() {
            continue;
        }
        if !key.is_empty() {
            map.insert(key, val);
        }
    }
    map
}

/// 提取正文第一个 `# ` 标题（剥掉前导 `#` 与 `[PLAN-xxx]` 前缀）。
pub fn extract_title(content: &str) -> String {
    for line in content.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("# ") {
            let rest = rest.trim();
            // 去掉 [PLAN-NNN] 前缀
            let title = if rest.starts_with('[') {
                if let Some(end) = rest.find(']') {
                    rest[end + 1..].trim().to_string()
                } else {
                    rest.to_string()
                }
            } else {
                rest.to_string()
              };
            return title;
        }
    }
    String::new()
}

/// 在 frontmatter 范围内替换 `key` 的值；不存在则插入到首行 `---` 之后。
/// 若 content 无 frontmatter，前置一个最小 frontmatter。保留其它所有字节。
pub fn set_field(content: &str, key: &str, value: &str) -> String {
    let first_line_end = content.find('\n').unwrap_or(content.len());
    let first_line = content[..first_line_end].trim_end();
    if first_line.trim() != "---" {
        // 无 frontmatter —— 前置创建
        return format!("---\n{}: {}\n---\n\n{}", key, value, content);
    }
    let body_start = if first_line_end < content.len() {
        first_line_end + 1
    } else {
        content.len()
    };
    let rest = &content[body_start..];
    // frontmatter 结束 = 独占一行的 "---"
    let (fm_body, tail) = match rest.find("\n---") {
        Some(i) => (&rest[..i + 1], &rest[i + 1..]),
        None => (rest, ""),
    };
    let re = Regex::new(&format!(r"(?m)^{}:\s*[^\r\n]*", regex::escape(key))).unwrap();
    let mut new_body = if re.is_match(fm_body) {
        re.replace(fm_body, &format!("{}: {}", key, value)).to_string()
    } else {
        format!("{}: {}\n{}", key, value, fm_body)
    };
    if !new_body.ends_with('\n') {
        new_body.push('\n');
    }
    let mut result = String::with_capacity(content.len() + key.len() + value.len() + 8);
    result.push_str(&content[..body_start]);
    result.push_str(&new_body);
    result.push_str(tail);
    result
}

// ============================================================
// 时间辅助（手写 ISO8601，避免引入 chrono/time）
// ============================================================

/// 当前时刻的 UTC ISO8601 字符串（`YYYY-MM-DDTHH:MM:SSZ`）。
pub fn now_iso() -> String {
    epoch_to_iso(now_sec())
}

/// epoch 秒 → UTC ISO8601。用 Howard Hinnant 的 civil_from_days 算法。
pub fn epoch_to_iso(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = (secs % 86400) as u64;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hour, min, sec)
}

/// days since 1970-01-01 → (year, month, day). Howard Hinnant 算法。
fn days_to_ymd(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// feature_name → ASCII kebab-case slug；非 ASCII 兜底为 `plan`。
pub fn slugify(s: &str) -> String {
    let mut slug: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug = slug.trim_matches('-').to_string();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    if slug.is_empty() {
        "plan".to_string()
    } else {
        slug
    }
}

// ============================================================
// PlansStore — 目录扫描型
// ============================================================

/// 以 `{root}/docs/plans/` 为根的 plan 存储。磁盘为唯一事实源，每次调用
/// 直接扫描目录（plan 数量级小，无需常驻缓存）。`archived/` 子目录承载
/// 已 merge 的 plan。
pub struct PlansStore {
    /// 活跃 plan 目录（`root/docs/plans`）。
    pub plans_dir: PathBuf,
    /// 归档目录（`root/docs/plans/archived`）。
    pub archived_dir: PathBuf,
}

impl PlansStore {
    /// 新建 store。`plans_dir` 通常 = `{workspace_root}/docs/plans`；
    /// `archived_dir` 固定为 `plans_dir/archived`。构造时确保两目录存在。
    pub fn new(plans_dir: PathBuf) -> Self {
        let archived_dir = plans_dir.join("archived");
        let _ = std::fs::create_dir_all(&plans_dir);
        let _ = std::fs::create_dir_all(&archived_dir);
        Self {
            plans_dir,
            archived_dir,
        }
    }

    /// 扫描某目录下所有 `NNN-*.md` 文件（NNN = 3 位数字），返回解析结果。
    fn scan_dir(&self, dir: &Path) -> Vec<PlanFile> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut plans = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !(name.len() >= 4 && name.as_bytes()[0].is_ascii_digit() && name.ends_with(".md")) {
                continue;
            }
            // 必须形如 NNN-（前 3 位数字 + '-'）
            if name.len() < 4 || !name.as_bytes()[..3].iter().all(|b| b.is_ascii_digit()) || name.as_bytes()[3] != b'-' {
                continue;
            }
            if let Ok(pf) = PlanFile::from_path(&path, &self.plans_dir) {
                plans.push(pf);
            }
        }
        plans
    }

    /// 列出全部计划，按序号升序。`include_archived=true` 时合并 `archived/`。
    pub fn list(&self, include_archived: bool) -> Vec<PlanFile> {
        let mut all = self.scan_dir(&self.plans_dir);
        if include_archived {
            all.extend(self.scan_dir(&self.archived_dir));
        }
        all.sort_by_key(|p| p.seq);
        all
    }

    /// 按序号读取单个计划（自动含 `archived/`）。
    pub fn get(&self, seq: u32) -> Option<PlanFile> {
        self.list(true).into_iter().find(|p| p.seq == seq)
    }

    /// 防漏号：扫描 `plans_dir` + `archived_dir` 所有 `NNN-*.md`，取最大序号
    /// + 1（空目录 → 1）。008 §8 确定性算法。
    pub fn next_seq(&self) -> u32 {
        let mut max = 0u32;
        for p in self.list(true) {
            if p.seq > max {
                max = p.seq;
            }
        }
        max + 1
    }

    /// 新建 plan：自动分配序号、注入 frontmatter、写盘。返回新建的 PlanFile。
    /// `content` 为空则使用最小模板。
    pub fn create(&self, feature_name: &str, content: &str) -> Result<PlanFile, String> {
        let seq = self.next_seq();
        let id = format!("PLAN-{:03}", seq);
        let slug = slugify(feature_name);
        let filename = format!("{:03}-{}.md", seq, slug);
        let path = self.plans_dir.join(&filename);
        if path.exists() {
            return Err(format!("plan file already exists: {}", path.display()));
        }
        let now = now_iso();
        let body = if content.trim().is_empty() {
            default_template(&id, feature_name)
        } else {
            content.to_string()
        };
        // 注入/更新 frontmatter 必需字段
        let body = set_field(&body, "plan_id", &id);
        let body = set_field(&body, "status", PlanStatus::Drafting.as_str());
        let body = set_field(&body, "feature_name", feature_name);
        let body = set_field(&body, "created_at", &now);
        let body = set_field(&body, "updated_at", &now);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, &body).map_err(|e| format!("failed to write plan: {}", e))?;
        PlanFile::from_path(&path, &self.plans_dir)
    }

    /// 覆盖正文（保留 frontmatter 的 plan_id；刷新 updated_at）。
    pub fn update(&self, seq: u32, content: &str) -> Result<PlanFile, String> {
        let pf = self.get(seq).ok_or_else(|| format!("plan {:03} not found", seq))?;
        // 保留原 plan_id（若新 content 的 frontmatter 缺失/冲突，以原 id 为准）
        let body = set_field(content, "plan_id", &pf.id);
        let body = set_field(&body, "updated_at", &now_iso());
        let path = self.plans_dir.join(&pf.filename);
        let path = if path.exists() {
            path
        } else {
            self.archived_dir.join(&pf.filename)
        };
        std::fs::write(&path, &body).map_err(|e| format!("failed to write plan: {}", e))?;
        PlanFile::from_path(&path, &self.plans_dir)
    }

    /// 状态机流转（校验合法迁移；刷新 updated_at）。
    pub fn transition(&self, seq: u32, new_status: PlanStatus) -> Result<PlanFile, String> {
        let pf = self.get(seq).ok_or_else(|| format!("plan {:03} not found", seq))?;
        if !PlanStatus::can_transition(pf.status, new_status) {
            return Err(format!(
                "illegal transition {:?} → {:?} for plan {:03}",
                pf.status, new_status, seq
            ));
        }
        if pf.status == new_status {
            return Ok(pf); // 幂等，无需写盘
        }
        let body = set_field(&pf.content, "status", new_status.as_str());
        let body = set_field(&body, "updated_at", &now_iso());
        let path = if pf.archived {
            self.archived_dir.join(&pf.filename)
        } else {
            self.plans_dir.join(&pf.filename)
        };
        std::fs::write(&path, &body).map_err(|e| format!("failed to write plan: {}", e))?;
        PlanFile::from_path(&path, &self.plans_dir)
    }

    /// 归档：置 `status: archived` 并移入 `archived/`（PLAN-033 单一终态，
    /// 状态与位置恒一致）。reviewed 计划拒绝直接归档——必须先 merge 沉淀
    /// 进 Spec。已是 archived 则原样返回（幂等）。
    pub fn archive(&self, seq: u32) -> Result<PlanFile, String> {
        let pf = self.get(seq).ok_or_else(|| format!("plan {:03} not found", seq))?;
        if pf.archived {
            return Ok(pf);
        }
        if pf.status == PlanStatus::Reviewed {
            return Err(format!(
                "plan {:03} is reviewed; merge it to spec instead of archiving",
                seq
            ));
        }
        self.move_to_archived(seq)
    }

    /// 终态漏斗：直接写 `status: archived`（不经 `can_transition`——两条进入
    /// 路径 archive 搁置 / merge 沉淀都不受手动转移状态机约束）+ 刷新
    /// updated_at + 移入 `archived/`。调用方须保证计划当前在活跃目录。
    fn move_to_archived(&self, seq: u32) -> Result<PlanFile, String> {
        let pf = self.get(seq).ok_or_else(|| format!("plan {:03} not found", seq))?;
        let body = if pf.status == PlanStatus::Archived {
            pf.content.clone()
        } else {
            set_field(&pf.content, "status", PlanStatus::Archived.as_str())
        };
        let body = set_field(&body, "updated_at", &now_iso());
        let src = self.plans_dir.join(&pf.filename);
        std::fs::write(&src, &body).map_err(|e| format!("failed to write plan: {}", e))?;
        let dst = self.archived_dir.join(&pf.filename);
        if dst.exists() {
            return Err(format!(
                "archived target already exists: {}",
                dst.display()
            ));
        }
        let _ = std::fs::create_dir_all(&self.archived_dir);
        std::fs::rename(&src, &dst).map_err(|e| format!("failed to archive plan: {}", e))?;
        PlanFile::from_path(&dst, &self.plans_dir)
    }
}

/// 最小 plan 模板（create 时 content 为空用）。
fn default_template(id: &str, feature_name: &str) -> String {
    format!(
        "# [{id}] {feature_name}\n\n## 0. 变更摘要\n\n## 1. 目标\n\n## 8. 执行步骤\n\n## 7. 验收标准\n"
    )
}

/// merge 核心（HTTP handler 与 plan_tools::MergePlan 共用，PLAN-030 T2）：
/// 门禁 reviewed → `plan_to_items` 拆解 → upsert 进 specs doc → save →
/// `move_to_archived`（置 archived + 移入 archived/，单一终态）。
/// 返回触及的 section + item 数。
pub fn merge_plan_stores(
    plans: &PlansStore,
    specs: &crate::specs::SpecsStore,
    seq: u32,
) -> Result<crate::plan_merge::MergeResult, String> {
    let plan = plans
        .get(seq)
        .ok_or_else(|| format!("plan {:03} not found", seq))?;
    if plan.status != PlanStatus::Reviewed {
        return Err(format!(
            "plan {:03} is {:?}, must be reviewed to merge",
            seq, plan.status
        ));
    }
    let (items, result) = crate::plan_merge::plan_to_items(&plan);
    let mut doc = specs.load().map_err(|e| e.to_string())?;
    crate::plan_merge::upsert_items_into_doc(&mut doc, items);
    specs.save(&doc).map_err(|e| e.to_string())?;
    plans.move_to_archived(seq)?;
    Ok(result)
}

// ============================================================
// HTTP routes (hw escape hatch — PLAN-024 §3.6)
//
// `/api/plans/*` is served by hand-written axum routes (not the ag/.at
// track) because the a2r transpiler has drifted (verified during task 2:
// regenerated outputs diverge on a2r_std path, .clone() redundancy, and
// statement-terminator style). Routes delegate to `ws.plans` via
// `state.registry.get(...)`, mirroring `wiki::wiki_routes()`.
// KNOWN-DEBT: re-route through the ag track once the transpiler is realigned.
// ============================================================

use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};

use crate::server::AppState;
use crate::workspace::WorkspaceQuery;

/// Query params for plan list/get endpoints.
#[derive(Deserialize)]
pub struct PlansQuery {
    /// `?workspace=<id>` (flattened) — empty → default workspace.
    #[serde(flatten)]
    pub workspace: WorkspaceQuery,
    /// `?include_archived=true` — include `archived/` plans in the listing.
    #[serde(default)]
    pub include_archived: Option<bool>,
}

#[derive(Serialize)]
pub struct PlansListResponse {
    pub plans: Vec<PlanFile>,
}

#[derive(Deserialize)]
pub struct CreatePlanRequest {
    pub feature_name: String,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePlanRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct TransitionPlanRequest {
    pub status: String,
}

/// `GET /api/plans?include_archived=true` — list all plans (sorted by seq).
async fn plans_list(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
) -> Json<PlansListResponse> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    let plans = ws.plans.list(q.include_archived.unwrap_or(false));
    Json(PlansListResponse { plans })
}

/// `GET /api/plans/{seq}` — read a single plan (searches archived too).
async fn plans_get(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
    AxumPath(seq): AxumPath<u32>,
) -> Result<Json<PlanFile>, StatusCode> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    ws.plans.get(seq).map(Json).ok_or(StatusCode::NOT_FOUND)
}

/// `POST /api/plans` — create a new plan (auto-assigns seq).
async fn plans_create(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
    Json(req): Json<CreatePlanRequest>,
) -> Result<Json<PlanFile>, (StatusCode, String)> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    ws.plans
        .create(&req.feature_name, req.content.as_deref().unwrap_or(""))
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// `PUT /api/plans/{seq}` — update plan body (plan_id preserved).
async fn plans_update(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
    AxumPath(seq): AxumPath<u32>,
    Json(req): Json<UpdatePlanRequest>,
) -> Result<Json<PlanFile>, (StatusCode, String)> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    ws.plans
        .update(seq, &req.content)
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e))
}

/// `POST /api/plans/{seq}/transition` — state-machine transition.
async fn plans_transition(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
    AxumPath(seq): AxumPath<u32>,
    Json(req): Json<TransitionPlanRequest>,
) -> Result<Json<PlanFile>, (StatusCode, String)> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    let new_status = PlanStatus::from_str_lossy(&req.status);
    ws.plans
        .transition(seq, new_status)
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

/// `POST /api/plans/{seq}/archive` — 置 archived 并移入 `archived/`
/// （reviewed 计划拒绝：400 提示走 merge 沉淀）。
async fn plans_archive(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
    AxumPath(seq): AxumPath<u32>,
) -> Result<Json<PlanFile>, (StatusCode, String)> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    ws.plans
        .archive(seq)
        .map(Json)
        .map_err(|e| {
            let code = if e.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (code, e)
        })
}

/// `POST /api/plans/{seq}/merge` — 沉淀到 Spec（门禁 review_done → 拆解进 6 区
/// → transition Merged → archive）。返回触及的 section + item 数。
async fn plans_merge(
    State(state): State<AppState>,
    Query(q): Query<PlansQuery>,
    AxumPath(seq): AxumPath<u32>,
) -> Result<Json<crate::plan_merge::MergeResult>, (StatusCode, String)> {
    let ws = state.registry.get(&q.workspace.id_or_default(&state.registry));
    merge_plan_stores(&ws.plans, &ws.specs, seq)
        .map(Json)
        .map_err(|e| {
            let code = if e.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (code, e)
        })
}

/// All `/api/plans/*` routes. Merged into the main router in `server::serve()`.
pub fn plans_routes() -> Router<AppState> {
    Router::new()
        .route("/api/plans", get(plans_list).post(plans_create))
        .route("/api/plans/{seq}", get(plans_get).put(plans_update))
        .route("/api/plans/{seq}/transition", post(plans_transition))
        .route("/api/plans/{seq}/archive", post(plans_archive))
        .route("/api/plans/{seq}/merge", post(plans_merge))
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_store() -> (tempfile::TempDir, PlansStore) {
        let td = tempfile::tempdir().unwrap();
        let store = PlansStore::new(td.path().join("docs/plans"));
        (td, store)
    }

    #[test]
    fn status_roundtrip() {
        for s in [
            PlanStatus::Drafting,
            PlanStatus::Executing,
            PlanStatus::ExecutionDone,
            PlanStatus::Reviewed,
            PlanStatus::Archived,
        ] {
            assert_eq!(PlanStatus::from_str_lossy(s.as_str()), s);
        }
    }

    #[test]
    fn status_unknown_falls_back_to_drafting() {
        assert_eq!(PlanStatus::from_str_lossy("nonsense"), PlanStatus::Drafting);
        assert_eq!(PlanStatus::from_str_lossy(""), PlanStatus::Drafting);
    }

    #[test]
    fn legacy_status_strings_map_to_new_enum() {
        // PLAN-033 改名前的旧值：读取时映射，写盘自愈
        assert_eq!(PlanStatus::from_str_lossy("review_done"), PlanStatus::Reviewed);
        assert_eq!(PlanStatus::from_str_lossy("merged"), PlanStatus::Archived);
    }

    #[test]
    fn state_machine_legal_paths() {
        // 前进
        assert!(PlanStatus::can_transition(PlanStatus::Drafting, PlanStatus::Executing));
        assert!(PlanStatus::can_transition(PlanStatus::Executing, PlanStatus::ExecutionDone));
        assert!(PlanStatus::can_transition(PlanStatus::ExecutionDone, PlanStatus::Reviewed));
        // 跳过执行直接 review
        assert!(PlanStatus::can_transition(PlanStatus::Drafting, PlanStatus::Reviewed));
        // 回退（复审不通过）
        assert!(PlanStatus::can_transition(PlanStatus::Reviewed, PlanStatus::Executing));
        assert!(PlanStatus::can_transition(PlanStatus::ExecutionDone, PlanStatus::Executing));
        // 幂等
        for s in [
            PlanStatus::Drafting,
            PlanStatus::Executing,
            PlanStatus::ExecutionDone,
            PlanStatus::Reviewed,
            PlanStatus::Archived,
        ] {
            assert!(PlanStatus::can_transition(s, s));
        }
    }

    #[test]
    fn state_machine_illegal_paths() {
        // archived 是终态
        assert!(!PlanStatus::can_transition(PlanStatus::Archived, PlanStatus::Drafting));
        assert!(!PlanStatus::can_transition(PlanStatus::Archived, PlanStatus::Reviewed));
        // 不能从 drafting 直接跳到 execution_done / archived
        assert!(!PlanStatus::can_transition(PlanStatus::Drafting, PlanStatus::ExecutionDone));
        assert!(!PlanStatus::can_transition(PlanStatus::Drafting, PlanStatus::Archived));
        // 不能从 executing 直接 archived
        assert!(!PlanStatus::can_transition(PlanStatus::Executing, PlanStatus::Archived));
        // reviewed 不能回 drafting，也不能手动进终态（只能 archive 搁置或 merge 沉淀）
        assert!(!PlanStatus::can_transition(PlanStatus::Reviewed, PlanStatus::Drafting));
        assert!(!PlanStatus::can_transition(PlanStatus::Reviewed, PlanStatus::Archived));
    }

    #[test]
    fn frontmatter_parse_basic() {
        let content = "---\nplan_id: PLAN-024\nstatus: executing\nfeature_name: Hello World\ncreated_at: 2026-08-11T10:00:00Z\nupdated_at: 2026-08-11T12:00:00Z\n---\n\nbody";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("plan_id").unwrap(), "PLAN-024");
        assert_eq!(fm.get("status").unwrap(), "executing");
        assert_eq!(fm.get("feature_name").unwrap(), "Hello World");
    }

    #[test]
    fn frontmatter_parse_strips_inline_comment_and_quotes() {
        let content = "---\nstatus: drafting   # drafting → executing\nfeature_name: \"Some Name\"\n---\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("status").unwrap(), "drafting");
        assert_eq!(fm.get("feature_name").unwrap(), "Some Name");
    }

    #[test]
    fn frontmatter_no_fence_returns_empty() {
        let fm = parse_frontmatter("# just a title\n\nbody");
        assert!(fm.is_empty());
    }

    #[test]
    fn frontmatter_skips_list_items() {
        // supersedes_spec_components 等列表项不应进 map
        let content = "---\nstatus: drafting\nsupersedes_spec_components:\n  - \"a: 修改\"\n  - \"b: 新增\"\n---\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("status").unwrap(), "drafting");
        assert!(!fm.contains_key("supersedes_spec_components"));
    }

    #[test]
    fn extract_title_strips_plan_prefix() {
        let content = "---\nstatus: drafting\n---\n\n# [PLAN-042] 支持新语法解析 - 实施计划\n\nbody";
        assert_eq!(extract_title(content), "支持新语法解析 - 实施计划");
    }

    #[test]
    fn extract_title_no_prefix() {
        assert_eq!(extract_title("# Plain Title\nbody"), "Plain Title");
    }

    #[test]
    fn set_field_updates_existing() {
        let content = "---\nstatus: drafting\nfeature_name: old\n---\n\nbody";
        let out = set_field(content, "status", "executing");
        let fm = parse_frontmatter(&out);
        assert_eq!(fm.get("status").unwrap(), "executing");
        // 其它字段保留
        assert_eq!(fm.get("feature_name").unwrap(), "old");
        // body 保留
        assert!(out.ends_with("body"));
    }

    #[test]
    fn set_field_inserts_when_missing() {
        let content = "---\nstatus: drafting\n---\n\nbody";
        let out = set_field(content, "feature_name", "New");
        let fm = parse_frontmatter(&out);
        assert_eq!(fm.get("feature_name").unwrap(), "New");
        assert_eq!(fm.get("status").unwrap(), "drafting");
    }

    #[test]
    fn set_field_creates_frontmatter_when_absent() {
        let content = "# No frontmatter\n\nbody";
        let out = set_field(content, "status", "drafting");
        assert!(out.starts_with("---\nstatus: drafting\n---\n"));
        let fm = parse_frontmatter(&out);
        assert_eq!(fm.get("status").unwrap(), "drafting");
    }

    #[test]
    fn set_field_preserves_other_frontmatter_bytes() {
        // 列表项等非 key:value 行必须原样保留
        let content = "---\nstatus: drafting\nsupersedes_spec_components:\n  - \"a: 修改\"\n---\n\nbody";
        let out = set_field(content, "status", "executing");
        assert!(out.contains("  - \"a: 修改\""), "list items must be preserved");
        let fm = parse_frontmatter(&out);
        assert_eq!(fm.get("status").unwrap(), "executing");
    }

    #[test]
    fn slugify_ascii() {
        assert_eq!(slugify("AutoPlan Architecture"), "autoplan-architecture");
        assert_eq!(slugify("Hello-World!"), "hello-world");
    }

    #[test]
    fn slugify_non_ascii_fallback() {
        assert_eq!(slugify("架构升级"), "plan");
        assert_eq!(slugify(""), "plan");
    }

    #[test]
    fn epoch_to_iso_known_value() {
        // 2026-08-11T13:00:00Z = epoch 1786453200（算法验证）
        assert_eq!(epoch_to_iso(1786453200), "2026-08-11T13:00:00Z");
    }

    #[test]
    fn now_iso_is_well_formed() {
        let s = now_iso();
        assert_eq!(s.len(), 20);
        assert_eq!(s.chars().nth(4), Some('-'));
        assert!(s.ends_with('Z'));
    }

    #[test]
    fn next_seq_empty_dir_is_one() {
        let (_td, store) = tmp_store();
        assert_eq!(store.next_seq(), 1);
    }

    #[test]
    fn next_seq_includes_archived() {
        let (_td, store) = tmp_store();
        store.create("alpha", "").unwrap();
        store.archive(1).unwrap();
        // archived 的 001 也算，下一个 = 002
        assert_eq!(store.next_seq(), 2);
    }

    #[test]
    fn next_seq_skips_gaps() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap(); // 001
        store.create("b", "").unwrap(); // 002
        store.create("c", "").unwrap(); // 003
        // 手动删 002（filename = 002-b.md）
        fs::remove_file(store.plans_dir.join("002-b.md")).unwrap();
        assert_eq!(store.next_seq(), 4);
    }

    #[test]
    fn create_assigns_seq_and_frontmatter() {
        let (_td, store) = tmp_store();
        let pf = store.create("Feature One", "").unwrap();
        assert_eq!(pf.seq, 1);
        assert_eq!(pf.id, "PLAN-001");
        assert_eq!(pf.filename, "001-feature-one.md");
        assert_eq!(pf.status, PlanStatus::Drafting);
        assert_eq!(pf.feature_name, "Feature One");
        assert!(!pf.archived);
        // 文件确实落盘
        assert!(store.plans_dir.join("001-feature-one.md").exists());
    }

    #[test]
    fn create_with_content_injects_frontmatter() {
        let (_td, store) = tmp_store();
        let body = "# My Plan\n\nsome body";
        let pf = store.create("X", body).unwrap();
        assert!(pf.content.starts_with("---"));
        let fm = parse_frontmatter(&pf.content);
        assert_eq!(fm.get("plan_id").unwrap(), "PLAN-001");
        assert_eq!(fm.get("status").unwrap(), "drafting");
        // 原正文保留
        assert!(pf.content.contains("# My Plan"));
        assert!(pf.content.contains("some body"));
    }

    #[test]
    fn get_reads_back_created() {
        let (_td, store) = tmp_store();
        store.create("A", "").unwrap();
        store.create("B", "").unwrap();
        let got = store.get(2).unwrap();
        assert_eq!(got.id, "PLAN-002");
        assert_eq!(got.feature_name, "B");
    }

    #[test]
    fn list_excludes_archived_by_default() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        store.create("b", "").unwrap();
        store.archive(1).unwrap();
        let active = store.list(false);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].seq, 2);
        let all = store.list(true);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn transition_valid_updates_status() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        let pf = store.transition(1, PlanStatus::Executing).unwrap();
        assert_eq!(pf.status, PlanStatus::Executing);
        // 写盘生效
        let reloaded = store.get(1).unwrap();
        assert_eq!(reloaded.status, PlanStatus::Executing);
    }

    #[test]
    fn transition_illegal_rejected() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        // drafting → archived 非法（终态不经 transition，走 archive/merge）
        let err = store.transition(1, PlanStatus::Archived);
        assert!(err.is_err());
        // 状态未变
        assert_eq!(store.get(1).unwrap().status, PlanStatus::Drafting);
    }

    #[test]
    fn transition_idempotent_no_write() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        let pf = store.transition(1, PlanStatus::Drafting).unwrap();
        assert_eq!(pf.status, PlanStatus::Drafting);
    }

    #[test]
    fn archive_moves_file_and_sets_status() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        let pf = store.archive(1).unwrap();
        assert!(pf.archived);
        assert_eq!(pf.status, PlanStatus::Archived, "归档即终态：状态与位置一致");
        // 源文件不在活跃目录
        assert!(!store.plans_dir.join("001-a.md").exists());
        // 在归档目录
        assert!(store.archived_dir.join("001-a.md").exists());
        // get 仍能找到（含 archived）
        assert!(store.get(1).is_some());
    }

    #[test]
    fn archive_rejects_reviewed() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        store.transition(1, PlanStatus::Reviewed).unwrap();
        let err = store.archive(1);
        assert!(err.is_err(), "reviewed 计划必须走 merge 沉淀，不能直接归档");
        // 文件未移动、状态未变
        assert!(store.plans_dir.join("001-a.md").exists());
        assert_eq!(store.get(1).unwrap().status, PlanStatus::Reviewed);
    }

    #[test]
    fn archive_idempotent() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        store.archive(1).unwrap();
        let again = store.archive(1).unwrap();
        assert!(again.archived);
    }

    #[test]
    fn update_preserves_plan_id_refreshes_updated_at() {
        let (_td, store) = tmp_store();
        store.create("a", "").unwrap();
        let original = store.get(1).unwrap();
        let new_body = "---\nstatus: drafting\nplan_id: HACK\n---\n\n# new body";
        let updated = store.update(1, new_body).unwrap();
        // plan_id 被强制保留为原值
        assert_eq!(updated.id, "PLAN-001");
        assert!(updated.content.contains("# new body"));
        // updated_at 刷新为有效 ISO（同秒内不比较；刷新逻辑由 set_field 测试覆盖）
        assert!(updated.updated_at.contains('T') && updated.updated_at.ends_with('Z'));
    }

    #[test]
    fn legacy_plan_without_frontmatter_parses_as_drafting() {
        let (_td, store) = tmp_store();
        // 手写一个无 frontmatter 的旧式 plan
        let path = store.plans_dir.join("042-legacy.md");
        fs::write(&path, "# Legacy Plan\n\nold style, no frontmatter").unwrap();
        let pf = store.get(42).unwrap();
        assert_eq!(pf.status, PlanStatus::Drafting);
        assert_eq!(pf.id, "PLAN-042");
        assert_eq!(pf.title, "Legacy Plan");
    }

    #[test]
    fn migrate_legacy_injects_frontmatter() {
        let (_td, store) = tmp_store();
        let path = store.plans_dir.join("005-old.md");
        fs::write(&path, "# Old\n\nbody").unwrap();
        // 通过 update（传原内容）触发 frontmatter 注入
        let original = fs::read_to_string(&path).unwrap();
        let updated = store.update(5, &original).unwrap();
        assert!(updated.content.starts_with("---"));
        assert_eq!(updated.id, "PLAN-005");
    }
}
