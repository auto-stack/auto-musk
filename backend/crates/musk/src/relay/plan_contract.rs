//! PLAN-096 T-02: Plan execution contract — the mechanical single source for
//! the product-internal `/auto-plan:*` flow (SD-01/SD-02).
//!
//! Three responsibilities:
//! 1. **Skill snapshots** ([`snapshot_plan_skills`]): the four auto-plan
//!    skills are the sole discipline source. At run start their content +
//!    SHA-256 are frozen; a missing / unreadable / hash-mismatched skill is a
//!    hard failure (no silent fallback to the retired inline templates).
//!    Mid-run skill changes never alter a running run (snapshot semantics).
//! 2. **Plan contract reading** ([`PlanContract::read`]): a complete,
//!    lossless machine view of a plan file — full YAML frontmatter (including
//!    the spec-impact lists the legacy scalar parser drops), required
//!    sections, task list (§8) and acceptance criteria (§7). Parse failures
//!    are loud rejections, not degraded guesses.
//! 3. **Semantic identity** ([`PlanContract::semantic_hash`]): a
//!    canonical, progress-insensitive hash over 目标/设计/验收标准/任务动作/
//!    规范增量 (§1/§2/§5/§7/§8 + the three spec-impact lists). Checkbox
//!    ticks, `证据：` annotations, §9 review log, §10 clarifications,
//!    status/current_step/total_steps and timestamps are NOT semantic —
//!    progress alone must never invalidate an approval (AC-06), while any
//!    goal/design/AC/task/delta change must (old approval expires).
//!
//! The B-track consumption contract draft lives in the serde shape of
//! [`PlanExecutionState`] / [`RunPlanEvent`] facts (fields are the published
//! surface; finalized in T-13's spec-delta report).

use std::collections::BTreeMap;
use std::path::Path;

use sha2::{Digest, Sha256};

/// Contract version of the plan-execution binding + facts surface. Bumping
/// invalidates old bindings (re-entry reads refuse mismatched versions).
pub const PLAN_EXECUTION_CONTRACT_VERSION: u32 = 1;

/// The four vendored auto-plan skills, in flow order (§5.1): the plan→work→
/// review→merge stages consume exactly these; the profession stays fixed
/// (advisor/coder/reviewer/assistant) on top of them.
pub const PLAN_SKILLS: [&str; 4] = [
    "auto-plan-new",
    "auto-plan-work",
    "auto-plan-review",
    "auto-plan-merge",
];

// ── Skill snapshots ─────────────────────────────────────────────────────────

/// One frozen skill file: content + hash captured at snapshot time.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillEntry {
    /// Skill directory name (e.g. `auto-plan-work`).
    pub name: String,
    /// Absolute path the content was read from (diagnostics only).
    pub path: String,
    /// SHA-256 hex of the SKILL.md bytes.
    pub sha256: String,
    /// Full SKILL.md content (injected into phase tasks verbatim).
    pub content: String,
}

/// Frozen snapshot of the four plan skills for one run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkillSnapshot {
    pub skills: BTreeMap<String, SkillEntry>,
}

impl SkillSnapshot {
    /// Hash map form for bindings/receipts (`skills_hashes`).
    pub fn hashes(&self) -> BTreeMap<String, String> {
        self.skills
            .iter()
            .map(|(k, v)| (k.clone(), v.sha256.clone()))
            .collect()
    }
}

/// Snapshot the four auto-plan skills from a trusted source root (the
/// repository's `.agents/skills/`). Missing or unreadable skills are hard
/// failures — the product plan flow must never silently run on a stale
/// inline template (§5.1).
pub fn snapshot_plan_skills(source_root: &Path) -> Result<SkillSnapshot, String> {
    let mut skills = BTreeMap::new();
    for name in PLAN_SKILLS {
        let skill_md = source_root.join(name).join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md).map_err(|e| {
            format!(
                "plan skill '{}' unreadable at {}: {} — refusing to run the plan flow without its discipline source",
                name,
                skill_md.display(),
                e
            )
        })?;
        if content.trim().is_empty() {
            return Err(format!(
                "plan skill '{}' is empty at {} — refusing to run",
                name,
                skill_md.display()
            ));
        }
        skills.insert(
            name.to_string(),
            SkillEntry {
                name: name.to_string(),
                path: skill_md.display().to_string(),
                sha256: sha256_hex(content.as_bytes()),
                content,
            },
        );
    }
    Ok(SkillSnapshot { skills })
}

/// Resolve the trusted skills source root using the same chain as
/// `builtin_skills::skills_source_root` (env → CWD → build-time repo root),
/// kept as a separate function so the contract module does not depend on
/// serve-time sync state.
pub fn plan_skills_source_root() -> Result<std::path::PathBuf, String> {
    if let Ok(dir) = std::env::var("MUSK_SKILLS_DIR") {
        let p = std::path::PathBuf::from(dir);
        if p.is_dir() {
            return Ok(p);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(".agents").join("skills");
        if p.is_dir() {
            return Ok(p);
        }
    }
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join(".agents")
        .join("skills");
    if p.is_dir() {
        return Ok(p);
    }
    Err("plan skills source root not found (MUSK_SKILLS_DIR / CWD / build-time repo)".into())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

// ── Plan contract ───────────────────────────────────────────────────────────

/// Full machine view of one plan file. Built by [`PlanContract::read`];
/// construction is fallible — anything unreadable is a rejection.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanContract {
    pub plan_id: String,
    pub status: String,
    pub feature_name: String,
    pub plan_revision: u32,
    pub current_step: u32,
    pub total_steps: u32,
    /// spec-impact lists (complete, order-preserving — the legacy scalar
    /// parser loses these entirely).
    pub supersedes_spec_components: Vec<String>,
    pub new_spec_components: Vec<String>,
    pub touched_goals: Vec<String>,
    /// Numbered sections (`## N. Title`), keyed by the number string.
    pub sections: BTreeMap<String, PlanSection>,
    /// §8 execution tasks in file order.
    pub tasks: Vec<PlanTask>,
    /// §7 acceptance criteria in file order.
    pub acceptance: Vec<PlanAc>,
    /// Progress-insensitive semantic hash (see module docs / AC-06).
    pub semantic_hash: String,
    /// SHA-256 of the exact file bytes (approval binds this revision).
    pub contract_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSection {
    pub number: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTask {
    /// Stable id token when present (e.g. `T-01`), else positional `#1`.
    pub id: String,
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAc {
    pub id: String,
    pub text: String,
    pub done: bool,
}

/// Extract the YAML frontmatter block text (between the first `---` line and
/// the closing `---` line). None when there is no frontmatter fence.
pub fn frontmatter_text(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let start = content.find('\n').map(|i| i + 1).unwrap_or(content.len());
    let rest = &content[start..];
    let end = rest.find("\n---")?; // closing fence (start of its line)
    Some(&rest[..end])
}

/// Parse the frontmatter as YAML into a scalar/list accessor map. Returns a
/// descriptive error on malformed YAML — never a silent empty map (Q-02:
/// the legacy scalar parser is display-compat only, not the contract).
pub fn parse_frontmatter_yaml(content: &str) -> Result<yaml_rust::Yaml, String> {
    let fm = frontmatter_text(content)
        .ok_or_else(|| "plan file has no YAML frontmatter fence (--- … ---)".to_string())?;
    let docs = yaml_rust::YamlLoader::load_from_str(fm)
        .map_err(|e| format!("frontmatter is not valid YAML: {e:?}"))?;
    let doc = docs
        .into_iter()
        .next()
        .ok_or_else(|| "frontmatter parsed to an empty YAML document".to_string())?;
    Ok(doc)
}

fn yaml_str(doc: &yaml_rust::Yaml, key: &str) -> Result<String, String> {
    match &doc[key] {
        yaml_rust::Yaml::String(s) => Ok(s.clone()),
        yaml_rust::Yaml::Real(r) => Ok(r.clone()),
        yaml_rust::Yaml::Integer(i) => Ok(i.to_string()),
        yaml_rust::Yaml::Boolean(b) => Ok(b.to_string()),
        yaml_rust::Yaml::Null | yaml_rust::Yaml::BadValue => Err(format!(
            "frontmatter field '{key}' missing (complete contract required)"
        )),
        other => Err(format!(
            "frontmatter field '{key}' has unsupported YAML shape: {other:?}"
        )),
    }
}

fn yaml_u32(doc: &yaml_rust::Yaml, key: &str) -> Result<u32, String> {
    match &doc[key] {
        yaml_rust::Yaml::Integer(i) if *i >= 0 && *i <= u32::MAX as i64 => Ok(*i as u32),
        yaml_rust::Yaml::String(s) => s
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("frontmatter field '{key}' is not a u32: '{s}'")),
        _ => Err(format!(
            "frontmatter field '{key}' missing or not an integer (complete contract required)"
        )),
    }
}

fn yaml_list(doc: &yaml_rust::Yaml, key: &str) -> Result<Vec<String>, String> {
    match &doc[key] {
        yaml_rust::Yaml::Null | yaml_rust::Yaml::BadValue => Ok(Vec::new()),
        yaml_rust::Yaml::Array(items) => items
            .iter()
            .map(|it| match it {
                yaml_rust::Yaml::String(s) => Ok(s.clone()),
                other => Err(format!(
                    "frontmatter list '{key}' has non-string item: {other:?}"
                )),
            })
            .collect(),
        yaml_rust::Yaml::String(s) if s.trim().is_empty() => Ok(Vec::new()),
        other => Err(format!(
            "frontmatter field '{key}' must be a YAML list, got: {other:?}"
        )),
    }
}

/// Split the body into numbered `## N. Title` sections. Everything before the
/// first section (typically the `# [PLAN-NNN] …` title) is ignored here.
pub fn parse_sections(content: &str) -> Vec<PlanSection> {
    let mut sections: Vec<PlanSection> = Vec::new();
    let mut current: Option<(String, String, Vec<&str>)> = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            // Close the previous section.
            if let Some((num, title, body)) = current.take() {
                sections.push(PlanSection {
                    number: num,
                    title,
                    body: body.join("\n"),
                });
            }
            let (num, title) = match rest.find('.') {
                Some(dot) if rest[..dot].trim().chars().all(|c| c.is_ascii_digit()) && !rest[..dot].trim().is_empty() => (
                    rest[..dot].trim().to_string(),
                    rest[dot + 1..].trim().to_string(),
                ),
                _ => (String::new(), rest.trim().to_string()),
            };
            current = Some((num, title, Vec::new()));
        } else if let Some((_, _, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((num, title, body)) = current.take() {
        sections.push(PlanSection {
            number: num,
            title,
            body: body.join("\n"),
        });
    }
    sections
}

/// Count §8 checklist tasks from raw content (mechanical backfill helper;
/// lenient — 0 when §8 absent).
pub fn task_count_of(content: &str) -> u32 {
    parse_sections(content)
        .into_iter()
        .find(|s| s.number == "8")
        .map(|s| parse_checklist(&s.body).len() as u32)
        .unwrap_or(0)
}

/// One task/criterion line: `- [ ] T-01 something` / `- [x] AC-03 …`.
/// Checkbox inner text: space/`x`/`X`/`✅ …` (the skill's `[✅ 已完成]` form).
fn parse_checklist(body: &str) -> Vec<(bool, String)> {
    // checkbox 组用惰性量词：任务行内可能还有第二个 `]`（live 实证 L1：
    // `- [x] T-01 … [✅ 已完成] 证据…`）——贪婪 `[^]]*` 会吞掉整段致行
    // 解析失败、任务丢失。
    let re = regex::Regex::new(r"(?m)^[ \t]*[-*][ \t]*\[[ xX✅][^]]*?\][ \t]*(.+)$")
        .expect("static regex");
    re.captures_iter(body)
        .map(|c| {
            let whole = c.get(0).map(|m| m.as_str()).unwrap_or_default();
            let done = {
                let inner_start = whole.find('[').map(|i| i + 1).unwrap_or(0);
                let inner = &whole[inner_start..whole[inner_start..]
                    .find(']')
                    .map(|i| inner_start + i)
                    .unwrap_or(inner_start)];
                let t = inner.trim();
                t == "x" || t == "X" || t.starts_with('✅')
            };
            (done, c[1].trim().to_string())
        })
        .collect()
}

fn id_token(text: &str, fallback_index: usize, prefix_hint: &str) -> String {
    // First token shaped like an identifier (`T-01`, `AC-03`, `G-16`…).
    let token = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|c: char| "`*_".contains(c))
        .to_string();
    let looks_like_id = {
        let mut parts = token.splitn(2, '-');
        let head = parts.next().unwrap_or_default();
        let tail = parts.next().unwrap_or_default();
        !head.is_empty()
            && head.chars().all(|c| c.is_ascii_alphabetic())
            && !tail.is_empty()
            && tail.chars().all(|c| c.is_ascii_digit())
    };
    if looks_like_id {
        token
    } else if text.contains(prefix_hint) {
        // id appears later in the line — try to pull it out.
        for word in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
            if word.starts_with(prefix_hint)
                && word.len() > prefix_hint.len()
                && word[prefix_hint.len()..]
                    .chars()
                    .all(|c| c.is_ascii_digit())
            {
                return word.to_string();
            }
        }
        format!("#{fallback_index}")
    } else {
        format!("#{fallback_index}")
    }
}

impl PlanContract {
    /// Read + fully parse a plan file from disk. `path` is absolute or
    /// relative to the process CWD; identity checks against `expect_plan_id`
    /// (when provided) reject foreign files sharing the number.
    pub fn read(path: &Path, expect_plan_id: Option<&str>) -> Result<PlanContract, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("plan file unreadable at {}: {e}", path.display()))?;
        Self::from_content(&content, expect_plan_id)
    }

    /// Parse an already-read plan body (tests / in-memory flows).
    pub fn from_content(content: &str, expect_plan_id: Option<&str>) -> Result<PlanContract, String> {
        let doc = parse_frontmatter_yaml(content)?;
        let plan_id = yaml_str(&doc, "plan_id")?;
        if let Some(want) = expect_plan_id {
            if plan_id != want {
                return Err(format!(
                    "plan identity mismatch: file declares plan_id '{plan_id}', expected '{want}'"
                ));
            }
        }
        let contract = PlanContract {
            status: yaml_str(&doc, "status")?,
            feature_name: yaml_str(&doc, "feature_name")?,
            plan_revision: yaml_u32(&doc, "plan_revision")?,
            current_step: yaml_u32(&doc, "current_step")?,
            total_steps: yaml_u32(&doc, "total_steps")?,
            supersedes_spec_components: yaml_list(&doc, "supersedes_spec_components")?,
            new_spec_components: yaml_list(&doc, "new_spec_components")?,
            touched_goals: yaml_list(&doc, "touched_goals")?,
            sections: parse_sections(content)
                .into_iter()
                .filter(|s| !s.number.is_empty())
                .map(|s| (s.number.clone(), s))
                .collect(),
            tasks: Vec::new(),
            acceptance: Vec::new(),
            semantic_hash: String::new(),
            contract_hash: sha256_hex(content.as_bytes()),
            plan_id,
        };
        let mut contract = contract;
        // Tasks (§8) and acceptance criteria (§7).
        if let Some(s8) = contract.sections.get("8") {
            contract.tasks = parse_checklist(&s8.body)
                .into_iter()
                .enumerate()
                .map(|(i, (done, text))| PlanTask {
                    id: id_token(&text, i + 1, "T-"),
                    text,
                    done,
                })
                .collect();
        }
        if let Some(s7) = contract.sections.get("7") {
            contract.acceptance = parse_checklist(&s7.body)
                .into_iter()
                .enumerate()
                .map(|(i, (done, text))| PlanAc {
                    id: id_token(&text, i + 1, "AC-"),
                    text,
                    done,
                })
                .collect();
        }
        contract.semantic_hash = semantic_hash_of(&contract);
        Ok(contract)
    }

    /// Sections required for the plan to be machine-executable (§5.2:
    /// "必要章节/任务/验收可被读取"). 目标/验收标准/执行步骤 at minimum; the
    /// full §0–§10 skeleton is the advisor's authoring duty (checked at
    /// plan-stage pass, not at every read).
    pub fn required_sections_missing(&self) -> Vec<&'static str> {
        ["1", "7", "8"]
            .into_iter()
            .filter(|n| !self.sections.contains_key(*n))
            .collect()
    }

    /// Validate the contract is complete enough to leave the plan stage:
    /// required sections present, at least one task and one acceptance
    /// criterion, and `total_steps` matches the actual §8 task count.
    pub fn validate_complete(&self) -> Result<(), String> {
        let missing = self.required_sections_missing();
        if !missing.is_empty() {
            return Err(format!(
                "plan {} missing required sections: {missing:?}",
                self.plan_id
            ));
        }
        if self.tasks.is_empty() {
            return Err(format!(
                "plan {} §8 has no executable checklist tasks",
                self.plan_id
            ));
        }
        if self.acceptance.is_empty() {
            return Err(format!(
                "plan {} §7 has no acceptance criteria",
                self.plan_id
            ));
        }
        if self.total_steps != self.tasks.len() as u32 {
            return Err(format!(
                "plan {} frontmatter total_steps={} != §8 task count {}",
                self.plan_id,
                self.total_steps,
                self.tasks.len()
            ));
        }
        Ok(())
    }
}

// ── Semantic normalization (AC-06 core) ─────────────────────────────────────

/// Strip progress noise from one line: `证据：`/`evidence:` annotations cut,
/// `[✅ …]` completion tags removed entirely, remaining checkboxes (ticked
/// or not) → `[_]`. Task/action text itself is preserved verbatim.
fn normalize_semantic_line(line: &str) -> String {
    let mut out = line.trim_end().to_string();
    // Cut evidence annotations (keep the task/action text itself).
    for marker in ["证据：", "证据:", "evidence:"] {
        if let Some(i) = out.find(marker) {
            out.truncate(i);
        }
    }
    let mut out = out.trim_end().to_string();
    // 进度标记（✅/⏳，含 `[✅ 已完成]`/`[⏳ …]` 形态）之后的一切内容都是
    // tick/证据注记——live 实证（L1）：模型把完成标记与证据全部追加在
    // 任务行内，标记后文本不得进语义。
    if let Some(i) = out.find(|c: char| c == '✅' || c == '⏳') {
        out.truncate(i);
    }
    // 悬挂括号/破折号（live diff 实录：模型 tick 用孤立 em-dash 接注记）。
    let out = out
        .trim_end()
        .trim_end_matches(['[', '(', '（', '—', '–'])
        .trim_end();
    // 剩余 checkbox（`[x]`/`[X]`/`[ ]`）归一为 `[_]`。
    let cb_re = regex::Regex::new(r"\[[xX ]\]").expect("static regex");
    let replaced = cb_re.replace_all(out, "[_]");
    replaced.trim_end().to_string()
}

/// Canonical semantic text: identity + spec-impact lists + 目标/架构/详细
/// 设计/验收标准/执行步骤 sections with progress noise stripped.
pub fn semantic_canonical_text(c: &PlanContract) -> String {
    let mut buf = String::new();
    buf.push_str("plan_id: ");
    buf.push_str(&c.plan_id);
    buf.push_str("\nfeature_name: ");
    buf.push_str(&c.feature_name);
    for (label, list) in [
        ("supersedes", &c.supersedes_spec_components),
        ("new", &c.new_spec_components),
        ("goals", &c.touched_goals),
    ] {
        buf.push_str(&format!("\n{label}: [{}]", list.join(" | ")));
    }
    for num in ["1", "2", "5", "7", "8"] {
        // §7/§8 的语义面 = checklist 行本身（id + 规范化文本）；节内其余行
        // （证据子行、执行记录）是进度噪声——live 实证（L1）：coder 在
        // §8 内追加执行记录曾被误判为语义漂移。
        match num {
            "7" => {
                buf.push_str("\n## 7 验收标准\n");
                for ac in &c.acceptance {
                    let norm = normalize_semantic_line(&ac.text);
                    if !norm.is_empty() {
                        buf.push_str(&format!("- [_] {norm}\n"));
                    }
                }
                continue;
            }
            "8" => {
                buf.push_str("\n## 8 执行步骤\n");
                for t in &c.tasks {
                    let norm = normalize_semantic_line(&t.text);
                    if !norm.is_empty() {
                        buf.push_str(&format!("- [_] {norm}\n"));
                    }
                }
                continue;
            }
            _ => {}
        }
        match c.sections.get(num) {
            Some(s) => {
                buf.push_str(&format!("\n## {} {}\n", s.number, s.title));
                for line in s.body.lines() {
                    let norm = normalize_semantic_line(line);
                    if !norm.is_empty() {
                        buf.push_str(&norm);
                        buf.push('\n');
                    }
                }
            }
            // A missing required section IS a semantic fact (its absence
            // distinguishes plans); missing optional sections contribute
            // nothing either way.
            None => {
                if ["1", "7", "8"].contains(&num) {
                    buf.push_str(&format!("\n## {num} <missing>\n"));
                }
            }
        }
    }
    buf
}

fn semantic_hash_of(c: &PlanContract) -> String {
    sha256_hex(semantic_canonical_text(c).as_bytes())
}

/// 分部件语义哈希（诊断面）：漂移时报文可点名哪个语义面变化
/// （identity / lists / s1 / s2 / s5 / s7 / s8）。
pub fn semantic_parts(c: &PlanContract) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    let mut ident = String::new();
    ident.push_str(&format!("plan_id: {}\nfeature_name: {}", c.plan_id, c.feature_name));
    m.insert("identity".into(), sha256_hex(ident.as_bytes()));
    let lists = format!(
        "supersedes: [{}]\nnew: [{}]\ngoals: [{}]",
        c.supersedes_spec_components.join(" | "),
        c.new_spec_components.join(" | "),
        c.touched_goals.join(" | ")
    );
    m.insert("lists".into(), sha256_hex(lists.as_bytes()));
    for num in ["1", "2", "5", "7", "8"] {
        let mut buf = String::new();
        match num {
            "7" => {
                buf.push_str("\n## 7 验收标准\n");
                for ac in &c.acceptance {
                    let norm = normalize_semantic_line(&ac.text);
                    if !norm.is_empty() {
                        buf.push_str(&format!("- [_] {norm}\n"));
                    }
                }
            }
            "8" => {
                buf.push_str("\n## 8 执行步骤\n");
                for t in &c.tasks {
                    let norm = normalize_semantic_line(&t.text);
                    if !norm.is_empty() {
                        buf.push_str(&format!("- [_] {norm}\n"));
                    }
                }
            }
            _ => {
                if let Some(sec) = c.sections.get(num) {
                    buf.push_str(&format!("\n## {} {}\n", sec.number, sec.title));
                    for line in sec.body.lines() {
                        let norm = normalize_semantic_line(line);
                        if !norm.is_empty() {
                            buf.push_str(&norm);
                            buf.push('\n'.to_string().pop().unwrap());
                        }
                    }
                }
            }
        }
        m.insert(format!("s{num}"), sha256_hex(buf.as_bytes()));
    }
    m
}

// ── Execution facts surface (B-track consumption contract draft) ───────────

/// Structured result submitted by an agent through the `complete_plan_stage`
/// tool (§5.4). Natural-language handoffs are display-only; this record is
/// what the controller routes on.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StageResult {
    pub stage: String,
    pub plan_id: String,
    /// Attempt this claim belongs to (server-stamped at record time; the
    /// model's copy is ignored).
    #[serde(default)]
    pub attempt: u32,
    #[serde(default)]
    pub plan_revision: u32,
    /// pass | needs_fix | needs_replan | blocked
    pub outcome: String,
    #[serde(default)]
    pub commit: Option<String>,
    /// Per-AC verification records (id/status/evidence).
    #[serde(default)]
    pub acceptance_results: Vec<AcResult>,
    /// Review findings with stable ids (needs_fix routing keys).
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// Evidence pointers (command + path pairs) the server may re-check.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// Reference to the reviewed spec delta (paths/hash) for document stage.
    #[serde(default)]
    pub spec_delta_ref: Option<String>,
    #[serde(default)]
    pub timestamp: u64,
    /// Server-side read-back facts appended at validation time (not
    /// model-authored).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_facts: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AcResult {
    pub id: String,
    /// pass | partial | fail
    pub status: String,
    #[serde(default)]
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    /// Stable finding id — the no-progress comparison key across attempts.
    pub id: String,
    #[serde(default)]
    pub task: Option<String>,
    #[serde(default)]
    pub ac: Option<String>,
    #[serde(default)]
    pub description: String,
}

/// Per-run plan-execution facts (AC-13). Surfaced on RunState as the
/// optional `plan_execution` object and updated by the controller; all
/// fields default for old-JSON compatibility.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlanExecutionState {
    #[serde(default)]
    pub contract_version: u32,
    pub plan_id: String,
    #[serde(default)]
    pub plan_seq: u32,
    /// Plan file path relative to the main root (`docs/plans/NNN-*.md`).
    pub plan_path: String,
    #[serde(default)]
    pub plan_revision: u32,
    /// plan | execute | review | document | delivered
    pub phase: String,
    /// Current attempt for the phase (1-based).
    #[serde(default)]
    pub attempt: u32,
    /// Last stage outcome, if one was validated.
    #[serde(default)]
    pub outcome: Option<String>,
    /// Completed work→review repair rounds (limit: [`Self::repair_limit`]).
    #[serde(default)]
    pub repair_count: u32,
    #[serde(default)]
    pub repair_limit: u32,
    /// Bounded continuation counter per stage (output-truncation retry, max 1).
    #[serde(default)]
    pub continuations: BTreeMap<String, u32>,
    #[serde(default)]
    pub blocker: Option<String>,
    #[serde(default)]
    pub binding: Option<PlanExecutionBinding>,
    #[serde(default)]
    pub reviewed_commit: Option<String>,
    /// Delivery checkpoints (prepared/landed/ledger_refreshed/archived/cleaned
    /// → receipt objects), keyed in execution order.
    #[serde(default)]
    pub delivery_checkpoints: BTreeMap<String, serde_json::Value>,
    /// Workspace-relative durable receipt path (`.autoos/plan-delivery/…`).
    #[serde(default)]
    pub receipt_ref: Option<String>,
    /// Frozen skill snapshot (name → entry) for this run.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub skills: BTreeMap<String, SkillEntry>,
    /// Validated stage results (routing + no-progress evidence).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_results: Vec<StageResult>,
}

impl PlanExecutionState {
    /// New state for a plan flow run with the default repair limit (§5.4:
    /// three work→review repair rounds, the work-skill bound adopted as the
    /// product default — not a user-configured budget).
    pub fn new(plan_id: &str, plan_seq: u32, plan_path: &str, plan_revision: u32) -> Self {
        Self {
            contract_version: PLAN_EXECUTION_CONTRACT_VERSION,
            plan_id: plan_id.to_string(),
            plan_seq,
            plan_path: plan_path.to_string(),
            plan_revision,
            phase: "plan".into(),
            attempt: 1,
            repair_limit: 3,
            ..Default::default()
        }
    }
}

/// The binding frozen at approval time (§5.2). Progress fields (timestamps,
/// §9 log) deliberately excluded from the semantic approval scope; the
/// semantic hash covers 目标/设计/AC/任务/规范增量.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlanExecutionBinding {
    #[serde(default)]
    pub contract_version: u32,
    pub workspace_id: String,
    /// Main checkout absolute path (plans/specs shared state lives here).
    pub main_root: String,
    pub plan_id: String,
    pub plan_path: String,
    #[serde(default)]
    pub plan_revision: u32,
    /// Exact-bytes hash bound by the approval.
    pub contract_hash: String,
    /// Progress-insensitive semantic hash (execution may tick progress
    /// 分部件语义哈希（诊断面）：漂移报文点名哪个语义面变化。
    #[serde(default)]
    pub semantic_parts: std::collections::BTreeMap<String, String>,
    /// 批准时 canonical 文本快照（仅内存；不进 receipt——重入后无快照时
    /// 漂移报文退化为部件名）。
    #[serde(skip)]
    pub approved_canonical: Option<String>,
    /// without invalidating; semantic changes expire the approval).
    pub semantic_hash: String,
    #[serde(default)]
    pub skills_hashes: BTreeMap<String, String>,
    /// Git facts frozen at binding time (branch detected, never hardcoded).
    pub default_branch: String,
    pub base_commit: String,
    #[serde(default)]
    pub execution_root: Option<String>,
    #[serde(default)]
    pub dev_branch: Option<String>,
    /// human | auto (recorded adoption scope — never agent-asserted).
    pub authorization: String,
    #[serde(default)]
    pub repair_limit: u32,
    #[serde(default)]
    pub dependency_revisions: BTreeMap<String, String>,
}

/// A controller-emitted facts event payload (mirrored onto RunEvent
/// `plan_stage_facts` in T-10; kept here so the serde shape is single-sourced
/// for the B-track consumption contract).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunPlanEvent {
    #[serde(default)]
    pub timestamp: u64,
    pub plan_id: String,
    /// The phase the facts belong to.
    pub stage: String,
    #[serde(default)]
    pub attempt: u32,
    pub outcome: String,
    #[serde(default)]
    pub repair_count: u32,
    #[serde(default)]
    pub repair_limit: u32,
    #[serde(default)]
    pub blocker: Option<String>,
    #[serde(default)]
    pub reviewed_commit: Option<String>,
    #[serde(default)]
    pub delivery_checkpoint: Option<String>,
    #[serde(default)]
    pub receipt_ref: Option<String>,
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "---\n\
plan_id: PLAN-042\n\
status: drafting\n\
feature_name: 测试特性\n\
author: [agent]\n\
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
## 1. 目标\n\n- 交付 A\n- 交付 B\n\n\
## 2. 架构方案\n\n模块化。\n\n\
## 3. 技术栈\n\nRust。\n\n\
## 4. 需求分析与背景调查\n\n背景。\n\n\
## 5. 详细设计\n\n核心设计文字。\n\n\
## 6. 测试设计\n\n单元 + 集成。\n\n\
## 7. 验收标准\n\n- [ ] AC-01 A 可验证\n- [ ] AC-02 B 可验证\n\n\
## 8. 执行步骤\n\n- [ ] T-01 第一步\n- [ ] T-02 第二步\n\n\
## 9. 复审记录\n\n（空）\n\n\
## 10. 待澄清事项\n\n（空）\n";

    fn sample() -> PlanContract {
        PlanContract::from_content(SAMPLE, Some("PLAN-042")).unwrap()
    }

    #[test]
    fn full_frontmatter_lists_survive() {
        let c = sample();
        assert_eq!(c.plan_id, "PLAN-042");
        assert_eq!(c.status, "drafting");
        assert_eq!(c.plan_revision, 1);
        assert_eq!(c.total_steps, 2);
        assert_eq!(c.supersedes_spec_components, vec!["docs/specs/modules/alpha.md"]);
        assert_eq!(c.new_spec_components, vec!["docs/specs/modules/beta.md"]);
        assert_eq!(c.touched_goals, vec!["goal-x"]);
        assert_eq!(c.sections.len(), 11);
    }

    #[test]
    fn malformed_frontmatter_is_rejected_not_degraded() {
        // 断 fence 后不是合法 YAML（控制字符）→ 拒绝。
        let bad = "---\nplan_id: [unclosed\n---\nbody";
        assert!(PlanContract::from_content(bad, None).is_err());
        // 无 frontmatter → 拒绝。
        assert!(PlanContract::from_content("# no fm\n", None).is_err());
        // 缺字段 → 拒绝且指名。
        let partial = "---\nplan_id: PLAN-001\n---\n";
        let err = PlanContract::from_content(partial, None).unwrap_err();
        assert!(err.contains("status"), "names the first missing field: {err}");
    }

    #[test]
    fn identity_mismatch_rejected() {
        let err = PlanContract::from_content(SAMPLE, Some("PLAN-999")).unwrap_err();
        assert!(err.contains("identity mismatch"), "{err}");
    }

    #[test]
    fn tasks_and_acceptance_parsed_with_ids() {
        let c = sample();
        assert_eq!(c.tasks.len(), 2);
        assert_eq!(c.tasks[0].id, "T-01");
        assert!(!c.tasks[0].done);
        assert_eq!(c.acceptance.len(), 2);
        assert_eq!(c.acceptance[1].id, "AC-02");
        assert!(c.validate_complete().is_ok());
    }

    #[test]
    fn completed_marker_forms_all_recognized() {
        let body = "- [x] T-01 done\n- [X] T-02 upper\n- [✅ 已完成] T-03 skill form\n- [ ] T-04 pending";
        let parsed = parse_checklist(body);
        assert_eq!(parsed.len(), 4);
        assert!(parsed[0].0 && parsed[1].0 && parsed[2].0 && !parsed[3].0);
        assert_eq!(parsed[2].1, "T-03 skill form");
    }

    #[test]
    fn validate_rejects_total_steps_mismatch() {
        let mut c = sample();
        c.total_steps = 5;
        assert!(c.validate_complete().is_err());
    }

    /// AC-06 正向：进度变化（勾选/证据/状态/时间戳/§9/§10）不改语义哈希。
    /// 勾选形态沿技能约定：`- [x] …`（checkbox 翻转）+ `证据：` 注记 +
    /// `[✅ 已完成]` 尾标——进度噪音全部在归一化规则内；任务行自由文本
    /// 之外的改写属语义变化（反向用例覆盖）。
    #[test]
    fn semantic_hash_ignores_progress() {
        let c0 = sample();
        let progressed = SAMPLE
            .replace(
                "- [ ] T-01 第一步",
                "- [x] T-01 第一步 [✅ 已完成]\n  证据：commit abc123；V01 绿",
            )
            .replace("status: drafting", "status: executing")
            .replace("current_step: 0", "current_step: 1")
            .replace("updated_at: 2026-10-01T10:00:00Z", "updated_at: 2026-10-02T12:00:00Z")
            .replace("## 9. 复审记录\n\n（空）", "## 9. 复审记录\n\nstage: work | outcome: pass | 证据链接")
            .replace("## 10. 待澄清事项\n\n（空）", "## 10. 待澄清事项\n\n- Q-01 新疑问");
        let c1 = PlanContract::from_content(&progressed, Some("PLAN-042")).unwrap();
        assert_eq!(c0.semantic_hash, c1.semantic_hash, "progress-only change must not change the semantic hash");
        assert_ne!(c0.contract_hash, c1.contract_hash, "exact bytes do change");
        // 任务勾选态解析跟随（进度事实进 tasks，不进语义哈希）。
        assert!(c1.tasks[0].done);
    }

    /// AC-06 反向：语义变更（目标/设计/AC/任务动作/规范增量）必须换哈希。
    #[test]
    fn semantic_hash_changes_on_semantic_edits() {
        let c0 = sample();
        let cases = [
            ("goal edit", SAMPLE.replace("交付 A", "交付 A2")),
            ("ac edit", SAMPLE.replace("AC-01 A 可验证", "AC-01 A2 可验证")),
            ("task edit", SAMPLE.replace("T-01 第一步", "T-01 第一步改道")),
            ("design edit", SAMPLE.replace("核心设计文字。", "核心设计文字（改）。")),
            ("spec list edit", SAMPLE.replace("- goal-x", "- goal-y")),
            ("new ac added", SAMPLE.replace("- [ ] AC-02 B 可验证", "- [ ] AC-02 B 可验证\n- [ ] AC-03 C 可验证")),
        ];
        for (name, edited) in cases {
            let c1 = PlanContract::from_content(&edited, Some("PLAN-042")).unwrap();
            assert_ne!(c0.semantic_hash, c1.semantic_hash, "{name} must change the semantic hash");
        }
    }

    #[test]
    fn snapshot_requires_all_four_skills() {
        let td = tempfile::tempdir().unwrap();
        // 缺源 → 硬失败（不静默降级）。
        let err = snapshot_plan_skills(td.path()).unwrap_err();
        assert!(err.contains("auto-plan-new"), "{err}");
        // 逐个补齐 → 成功且 hash 稳定。
        for name in PLAN_SKILLS {
            std::fs::create_dir_all(td.path().join(name)).unwrap();
            std::fs::write(td.path().join(name).join("SKILL.md"), format!("# {name} discipline"))
                .unwrap();
        }
        let snap = snapshot_plan_skills(td.path()).unwrap();
        assert_eq!(snap.skills.len(), 4);
        let h = snap.skills["auto-plan-work"].sha256.clone();
        assert_eq!(h, sha256_hex(b"# auto-plan-work discipline"));
        assert_eq!(snap.hashes()["auto-plan-work"], *h);
        // 空内容 → 拒绝。
        std::fs::write(td.path().join("auto-plan-review").join("SKILL.md"), "  \n").unwrap();
        assert!(snapshot_plan_skills(td.path()).is_err());
    }

    #[test]
    fn canonical_text_is_stable_and_ordered() {
        let c = sample();
        let t1 = semantic_canonical_text(&c);
        let t2 = semantic_canonical_text(&sample());
        assert_eq!(t1, t2);
        assert!(t1.starts_with("plan_id: PLAN-042"));
        assert!(t1.contains("supersedes: [docs/specs/modules/alpha.md]"));
        assert!(t1.contains("## 7 验收标准"));
        // 勾选与证据不进 canonical 文本。
        assert!(!t1.contains("证据："));
    }
}
