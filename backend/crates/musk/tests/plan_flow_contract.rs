//! plan_flow_contract.rs — PLAN-096 V02: the plan-flow execution contract
//! matrix. T-02 initial subset: complete frontmatter reading (lists), identity
//! rejection, semantic-hash positive/negative fixtures, skill snapshot
//! hard-failure, frozen-skill template injection. T-03+ extends with
//! approval/binding, stage results, budget and routing cases.
//!
//! Serial: cases share process env (MUSK_SKILLS_DIR overrides).

use std::collections::BTreeMap;
use std::path::PathBuf;

use musk::relay::plan_contract::{
    parse_frontmatter_yaml, sha256_hex, PlanContract, SkillEntry, SkillSnapshot,
};
use musk::relay::plan_flow::phase_task;
use serial_test::serial;

fn fixture(content: &str, dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, content).unwrap();
    p
}

const BASE_PLAN: &str = "---\n\
plan_id: PLAN-042\n\
status: drafting\n\
feature_name: 测试特性\n\
created_at: 2026-10-01T10:00:00Z\n\
updated_at: 2026-10-01T10:00:00Z\n\
plan_revision: 1\n\
current_step: 0\n\
total_steps: 2\n\
supersedes_spec_components:\n\
  - docs/specs/modules/alpha.md\n\
new_spec_components:\n\
  - docs/specs/modules/beta.md\n\
touched_goals:\n\
  - goal-x\n\
---\n\n\
# [PLAN-042] 测试特性\n\n\
## 0. 变更摘要\n\n概要。\n\n\
## 1. 目标\n\n- 交付 A\n\n\
## 2. 架构方案\n\n模块化。\n\n\
## 5. 详细设计\n\n核心设计。\n\n\
## 7. 验收标准\n\n- [ ] AC-01 A 可验证\n- [ ] AC-02 B 可验证\n\n\
## 8. 执行步骤\n\n- [ ] T-01 第一步\n- [ ] T-02 第二步\n\n\
## 9. 复审记录\n\n（空）\n";

fn full_snap() -> SkillSnapshot {
    let mut s = SkillSnapshot::default();
    for (n, body) in [
        ("auto-plan-new", "# auto-plan-new discipline"),
        ("auto-plan-work", "# auto-plan-work discipline"),
        ("auto-plan-review", "# auto-plan-review discipline"),
        ("auto-plan-merge", "# auto-plan-merge discipline"),
    ] {
        s.skills.insert(
            n.to_string(),
            SkillEntry {
                name: n.to_string(),
                path: format!("/{n}/SKILL.md"),
                sha256: sha256_hex(body.as_bytes()),
                content: body.to_string(),
            },
        );
    }
    s
}

/// 完整 frontmatter（含 spec-impact 三列表）无损读取；列表在显示 parser
/// （plans.rs 标量）中丢失，在合同读取中必须完整。
#[test]
fn plan_contract_reads_full_lists_and_tasks() {
    let td = tempfile::tempdir().unwrap();
    let path = fixture(BASE_PLAN, td.path(), "docs/plans/042-x.md");
    let c = PlanContract::read(&path, Some("PLAN-042")).expect("complete contract");
    assert_eq!(c.supersedes_spec_components, vec!["docs/specs/modules/alpha.md"]);
    assert_eq!(c.new_spec_components, vec!["docs/specs/modules/beta.md"]);
    assert_eq!(c.touched_goals, vec!["goal-x"]);
    assert_eq!(c.tasks.len(), 2);
    assert_eq!(c.tasks[0].id, "T-01");
    assert_eq!(c.acceptance.len(), 2);
    assert_eq!(c.acceptance[1].id, "AC-02");
    assert!(c.validate_complete().is_ok());
    // 显示用标量 parser 依旧看不到列表（旧路径兼容、不作权威）。
    let scalar = musk::plans::parse_frontmatter(BASE_PLAN);
    assert!(!scalar.contains_key("touched_goals"));
}

/// 身份与完整性反例：文件缺失 / id 不符 / 外来同名 / 无 frontmatter /
/// 缺任务 / 缺 AC / total_steps 不符 → 全部明确拒绝。
#[test]
fn contract_rejections_are_loud() {
    let td = tempfile::tempdir().unwrap();
    // 不存在。
    assert!(PlanContract::read(&td.path().join("nope.md"), None).is_err());
    let path = fixture(BASE_PLAN, td.path(), "042-x.md");
    // id 不符。
    assert!(PlanContract::read(&path, Some("PLAN-999")).is_err());
    // 无 frontmatter。
    let bare = fixture("# bare\n\n- [ ] T-01 x\n", td.path(), "043-bare.md");
    assert!(PlanContract::read(&bare, None).is_err());
    // 缺 §8 任务。
    let no_tasks = fixture(
        &BASE_PLAN.replace("## 8. 执行步骤\n\n- [ ] T-01 第一步\n- [ ] T-02 第二步\n", "## 8. 执行步骤\n\n"),
        td.path(),
        "044-notasks.md",
    );
    let c = PlanContract::read(&no_tasks, None).unwrap();
    assert!(c.validate_complete().is_err());
    // 缺 AC。
    let no_ac = fixture(
        &BASE_PLAN.replace("- [ ] AC-01 A 可验证\n- [ ] AC-02 B 可验证\n", ""),
        td.path(),
        "045-noac.md",
    );
    let c = PlanContract::read(&no_ac, None).unwrap();
    assert!(c.validate_complete().is_err());
    // total_steps 不符。
    let mismatch = fixture(
        &BASE_PLAN.replace("total_steps: 2", "total_steps: 9"),
        td.path(),
        "046-mismatch.md",
    );
    let c = PlanContract::read(&mismatch, None).unwrap();
    let err = c.validate_complete().unwrap_err();
    assert!(err.contains("total_steps"), "{err}");
}

/// AC-06 正反样例：仅进度（勾选/证据/状态/时间戳/§9/§10）→ 语义哈希不变；
/// 目标/AC/任务动作/规范增量 → 必变。
#[test]
fn semantic_hash_progress_positive_and_semantic_negative() {
    let td = tempfile::tempdir().unwrap();
    let p0 = fixture(BASE_PLAN, td.path(), "042-a.md");
    let c0 = PlanContract::read(&p0, None).unwrap();

    // 正例：进度-only（真实落盘形态：勾选+✅尾标+证据行+状态/时间戳/§9/§10）。
    let progressed = BASE_PLAN
        .replace(
            "- [ ] T-01 第一步",
            "- [x] T-01 第一步 [✅ 已完成]\n  证据：cargo test 绿 (commit abc)",
        )
        .replace("status: drafting", "status: execution_done")
        .replace("current_step: 0", "current_step: 2")
        .replace("updated_at: 2026-10-01T10:00:00Z", "updated_at: 2026-10-03T09:00:00Z")
        .replace("## 9. 复审记录\n\n（空）", "## 9. 复审记录\n\nstage: work | pass | 证据满");
    let p1 = fixture(&progressed, td.path(), "042-b.md");
    let c1 = PlanContract::read(&p1, None).unwrap();
    assert_eq!(c0.semantic_hash, c1.semantic_hash, "progress-only must not rotate the semantic hash");
    assert_ne!(c0.contract_hash, c1.contract_hash);

    // 反例组：任一语义面变化都要换哈希。
    let negatives: [(&str, String); 5] = [
        ("goal", BASE_PLAN.replace("交付 A", "交付 A（范围扩大）")),
        ("ac", BASE_PLAN.replace("AC-01 A 可验证", "AC-01 A2 可验证")),
        ("task", BASE_PLAN.replace("T-01 第一步", "T-01 第一步改道")),
        ("design", BASE_PLAN.replace("核心设计。", "核心设计（改）。")),
        ("delta", BASE_PLAN.replace("- goal-x", "- goal-y")),
    ];
    for (name, edited) in negatives {
        let p = fixture(&edited, td.path(), &format!("042-neg-{name}.md"));
        let c = PlanContract::read(&p, None).unwrap();
        assert_ne!(
            c0.semantic_hash, c.semantic_hash,
            "{name} edit must rotate the semantic hash"
        );
    }
}

/// 技能快照缺源/空源硬失败（产品内正式 plan 流不得默默用旧模板）。
#[test]
fn skill_snapshot_missing_source_hard_fails() {
    let td = tempfile::tempdir().unwrap();
    let err = musk::relay::plan_contract::snapshot_plan_skills(td.path()).unwrap_err();
    assert!(err.contains("auto-plan-new"), "{err}");
    // 只差一个也失败。
    for name in ["auto-plan-new", "auto-plan-work", "auto-plan-review"] {
        let d = td.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("SKILL.md"), format!("# {name}")).unwrap();
    }
    let err = musk::relay::plan_contract::snapshot_plan_skills(td.path()).unwrap_err();
    assert!(err.contains("auto-plan-merge"), "{err}");
    // 空内容同拒。
    let d = td.path().join("auto-plan-merge");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("SKILL.md"), " \n").unwrap();
    assert!(musk::relay::plan_contract::snapshot_plan_skills(td.path()).is_err());
}

/// 真源技能指纹（AC-01 live 证据）：从仓内 .agents/skills 快照——四份内容
/// 存在、hash 与磁盘字节一致、mid-run 二次快照稳定。
#[test]
#[serial]
fn real_skill_snapshot_matches_disk_bytes() {
    // 构建期仓库根：backend/crates/musk → 上三级 = 仓根。
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let src = repo.join(".agents").join("skills");
    assert!(src.is_dir(), "vendored skills must exist at {}", src.display());
    let snap = musk::relay::plan_contract::snapshot_plan_skills(&src).expect("snapshot ok");
    for name in musk::relay::plan_contract::PLAN_SKILLS {
        let entry = &snap.skills[name];
        let bytes = std::fs::read(src.join(name).join("SKILL.md")).unwrap();
        assert_eq!(entry.sha256, sha256_hex(&bytes), "{name} hash == disk bytes");
        assert_eq!(entry.content, String::from_utf8_lossy(&bytes));
    }
    // 快照幂等：重跑一致（中途技能不变则 hash 不变）。
    let snap2 = musk::relay::plan_contract::snapshot_plan_skills(&src).unwrap();
    assert_eq!(snap.hashes(), snap2.hashes());
}

/// 相位模板消费冻结技能：完整快照 → 技能全文+hash 注入；空快照 → 阻断
/// 条款点名缺失技能（不静默降级）。
#[test]
fn phase_task_consumes_frozen_skills() {
    use std::collections::HashMap;
    let ctx = HashMap::new();
    let t = phase_task("plan", "execute", "需求", &ctx, &full_snap()).unwrap();
    assert!(t.contains("# auto-plan-work discipline"));
    assert!(t.contains("complete_plan_stage"));
    assert!(t.contains("Canvas 生成指引"));
    let empty = SkillSnapshot {
        skills: BTreeMap::new(),
    };
    let t = phase_task("plan", "review", "需求", &ctx, &empty).unwrap();
    assert!(t.contains("阻断性缺陷"));
    assert!(t.contains("auto-plan-review"));
}

/// frontmatter YAML 解析直接给出标量/列表访问（Q-02：列表不丢）。
#[test]
fn frontmatter_yaml_parses_lists() {
    let doc = parse_frontmatter_yaml(BASE_PLAN).unwrap();
    let list = &doc["touched_goals"];
    match list {
        yaml_rust::Yaml::Array(items) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].as_str().unwrap(), "goal-x");
        }
        other => panic!("touched_goals must be a list, got {other:?}"),
    }
    assert_eq!(doc["plan_id"].as_str().unwrap(), "PLAN-042");
}
