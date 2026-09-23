//! PLAN-087 T-08 + PLAN-090 T-03/T-06: 生成侧模板池 + 三层生成流指导 v2 注入。
//!
//! 内嵌两份最小模板（counter/hello），结合动态探测的词汇表（vocabulary）、
//! Blueprint 目录摘要（bp_tools）、L1>L2>L3 复用序、层归属三问及 ≤3 轮验收循环。
//! 注入段受 ≤8KB 经验预算约束（AC-01）。

use super::bp_tools::get_blueprint_catalog_summary;
use super::vocabulary::get_vocabulary_summary;

/// counter 模板 pac.at（002-counter 精简）。
pub const COUNTER_PAC: &str = r#"name: "counter"
version: "1.0.0"
scene: "ui"
render: "vue"
title: "Counter"
title_zh: "计数器"
window: "fit"
"#;

/// counter 模板 app.at（002-counter 主体）。
pub const COUNTER_APP: &str = r#"widget App {
    model {
        var count int = 0
    }

    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            style: "items-center gap-4 p-6"
        }
    }
}
"#;

/// hello 模板 pac.at（001-helloworld 精简）。
pub const HELLO_PAC: &str = r#"name: "hello-world"
version: "1.0.0"
scene: "ui"
render: "vue"
title: "Hello World"
title_zh: "你好世界"
window: "fit"
"#;

/// hello 模板 app.at（001-helloworld 精简）。
pub const HELLO_APP: &str = r#"widget App {
    view {
        col {
            text "Hello, World!" {
                style: "text-4xl font-bold text-primary",
                selectable: true
            }
            style: "p-8 items-center"
        }
    }
}
"#;

/// 内嵌模板清单（名字 → (pac.at, app.at)）。
pub const TEMPLATES: &[(&str, &str, &str)] = &[
    ("counter", COUNTER_PAC, COUNTER_APP),
    ("hello", HELLO_PAC, HELLO_APP),
];

/// coding 模式三层生成流指导 v2 骨干模板。
pub const GENERATION_PROMPT: &str = r#"## Live canvas: three-tier generation flow (M3)

You can build runnable Auto UI apps in this workspace and preview them LIVE on the
canvas panel (isolated VM window + screenshot stream to the user).

### 1. Reuse ladder (L1 > L2 > L3 priority)
- L2 Copy & Adapt (Primary): If an existing blueprint matches your need (e.g. note-list, login, filetree),
  inspect it via `bp_show` and copy its reference implementation, then adapt for the current app.
- L1 Declarative Bind: If a package dependency on blueprints is configured in pac.at, use `use bps.kind.name`.
- L3 Freeform Generation (Fallback): If no blueprint matches, compose stdlib widgets from scratch or adapt
  the embedded counter/hello templates.

### 2. Three questions on layer attribution (before modifying code)
1. Is this a one-off app tweak? -> Edit instance props/slots in the view.
2. Is this a reusable pattern change? -> Consider elevating into a shared blueprint spec.
3. Does this require a new primitive capability? -> Scaffold a new widget or binding.

### 3. Blueprint Catalog
{blueprint_catalog}

### 4. Widget Vocabulary
{vocabulary_summary}

### 5. Verification loop (≤3 rounds of iterative repair)
1. Write the target .at code into the workspace app directory.
2. Run `ui_lint { "path": "<file>" }` as an immediate advisory check to catch known pitfalls.
3. If implementing or adapting a blueprint, run `bp_check { "path": "<file>", "spec": "<kind>/<name>" }`.
4. Launch in canvas: `canvas_run { "app_path": "<dir>" }`.
5. Verify live state: `canvas_snapshot` to see visual output, `canvas_act` to drive buttons/inputs, `canvas_state` to assert model values.
6. Stop if needed: `canvas_stop`. Complete repairs within ≤3 rounds; report any remaining blockers.

### 6. Templates
--- counter (pac.at) ---
{counter_pac}
--- counter (src/front/app.at) ---
{counter_app}
--- hello (pac.at) ---
{hello_pac}
--- hello (src/front/app.at) ---
{hello_app}

### 7. Known pitfalls (do not fight these)
- Text interpolation: `Counter: ${.count}` in template strings (backtick).
- Inline click handlers: `onclick: () => {.count += 1}`.
- Inline clickable elements: use explicit `button`, avoid hanging `onclick` on plain `span` with child tags.
- No underscores in msg or handler names; widget `App` is the entry widget.
- Do NOT add build tooling, package.json, or node_modules — VM track interprets .at directly.
- After canvas_run, ALWAYS verify with canvas_snapshot / canvas_state before concluding.
"#;

/// 渲染最终指导文本（内插模板、词汇表、Blueprint 目录）。
pub fn generation_prompt() -> String {
    let bp_summary = get_blueprint_catalog_summary();
    let vocab_summary = get_vocabulary_summary();

    GENERATION_PROMPT
        .replace("{blueprint_catalog}", bp_summary.trim_end())
        .replace("{vocabulary_summary}", vocab_summary.trim_end())
        .replace("{counter_pac}", COUNTER_PAC.trim_end())
        .replace("{counter_app}", COUNTER_APP.trim_end())
        .replace("{hello_pac}", HELLO_PAC.trim_end())
        .replace("{hello_app}", HELLO_APP.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_prompt_interpolates_all_placeholders() {
        let p = generation_prompt();
        assert!(p.contains("name: \"counter\""));
        assert!(p.contains("widget App"));
        assert!(!p.contains("{counter_pac}"));
        assert!(!p.contains("{counter_app}"));
        assert!(!p.contains("{hello_pac}"));
        assert!(!p.contains("{hello_app}"));
        assert!(!p.contains("{blueprint_catalog}"));
        assert!(!p.contains("{vocabulary_summary}"));
    }

    #[test]
    fn generation_prompt_satisfies_ac01_budget() {
        let p = generation_prompt();
        assert!(
            p.len() <= 8192,
            "prompt length must be within 8KB budget (actual: {} bytes)",
            p.len()
        );
        assert!(
            p.len() > 2000,
            "prompt should contain comprehensive instructions (actual: {} bytes)",
            p.len()
        );
    }

    #[test]
    fn generation_prompt_contains_m3_keywords() {
        let p = generation_prompt();
        assert!(p.contains("Reuse ladder"), "must contain reuse ladder");
        assert!(p.contains("Three questions"), "must contain three questions");
        assert!(p.contains("≤3 rounds"), "must contain <=3 rounds loop bound");
        assert!(p.contains("Blueprint Catalog"), "must contain blueprint catalog");
        assert!(p.contains("Widget Vocabulary"), "must contain widget vocabulary");
        assert!(p.contains("ui_lint"), "must mention ui_lint");
        assert!(p.contains("bp_check"), "must mention bp_check");
    }

    #[test]
    fn templates_cover_two_entries() {
        assert_eq!(TEMPLATES.len(), 2);
        assert_eq!(TEMPLATES[0].0, "counter");
        assert_eq!(TEMPLATES[1].0, "hello");
    }
}
