//! PLAN-087 T-08: 生成侧模板池 + coding 模式生成指导注入。
//!
//! 内嵌两份最小模板（counter/hello，自 auto-lang examples/ui/{002,001} 精简
//! 拷贝，剔除 .am/、.auto/ 缓存与生成目录——VM 轨对 .at 直接解释，不需要
//! npm 工具链）。原稿的 examples/ui 扩展池运行时消费因沙箱而缓行：agent 的
//! read_file 被多根沙箱限制在工作区内，读 D:/autostack/auto-lang/examples
//! 需要 white-list 授权，M1 不把"读仓库外部"写进生成路径（登记 M3 再评估）。

/// counter 模板 pac.at（002-counter 精简）。
pub const COUNTER_PAC: &str = r#"name: "counter"
version: "1.0.0"
scene: "ui"
render: "vue"
title: "Counter"
title_zh: "计数器"
// 窗口随内容自然尺寸收缩。
window: "fit"
"#;

/// counter 模板 app.at（002-counter 主体；显式 msg/on 变体保留为注释参考）。
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

/// 内嵌模板清单（名字 → (pac.at, app.at)）。实例化 = 目录拷贝 +
/// pac.at `name/title` 参数化（调用方替换首部两行值）。
pub const TEMPLATES: &[(&str, &str, &str)] = &[
    ("counter", COUNTER_PAC, COUNTER_APP),
    ("hello", HELLO_PAC, HELLO_APP),
];

/// coding 模式生成指导（追加进系统上下文）。策略：模板优先 → 写文件 →
/// canvas_run 实况验收；附已知坑节选（视图侧实证清单）。
pub const GENERATION_PROMPT: &str = r#"## Live canvas: generating Auto apps

You can build runnable Auto UI apps in this workspace and preview them LIVE on the
canvas panel (the app runs in an isolated VM window; screenshots stream to the user).

Preferred flow (template-first):
1. Pick the closest embedded template and write it into a new app directory
   (e.g. `counter-app/pac.at` + `counter-app/src/front/app.at`), then adapt it.
2. Templates — write these files exactly, then modify:
   --- counter (pac.at) ---
   {counter_pac}
   --- counter (src/front/app.at) ---
   {counter_app}
   --- hello (pac.at) ---
   {hello_pac}
   --- hello (src/front/app.at) ---
   {hello_app}
3. Run canvas_run { "app_path": "counter-app" } — the canvas panel opens with a
   live window (first frame within ~10s).
4. Verify like an engineer: canvas_snapshot to SEE the UI, canvas_act to drive it
   (e.g. press the "+" button), canvas_state to ASSERT state (e.g. count == 1).
5. Iterate by editing app.at — the VM hot-reloads file changes (≤2s), the canvas
   refreshes by itself. canvas_stop when done.

Widget vocabulary (safe subset): col, row, text (style: tailwind-ish classes),
button (onclick), input (value/oninput), icon (name/size), image/img, span,
checkbox, toggle, select, text_editor. Compose layout with style classes
("flex items-center gap-4 p-6"), state in `model { var x int = ... }`.

Known pitfalls (do not fight these):
- Text interpolation: `Counter: ${.count}` in template strings (backtick).
- onclick inline lambdas: `onclick: () => {.count += 1}`.
- Keep names simple: no underscores in msg names; widget App is the entry.
- Don't add build tooling or package.json — VM track interprets .at directly.
- After canvas_run, ALWAYS verify with canvas_snapshot/canvas_state before
  telling the user it works.
"#;

/// 渲染最终指导文本（模板内容内插）。
pub fn generation_prompt() -> String {
    GENERATION_PROMPT
        .replace("{counter_pac}", COUNTER_PAC.trim_end())
        .replace("{counter_app}", COUNTER_APP.trim_end())
        .replace("{hello_pac}", HELLO_PAC.trim_end())
        .replace("{hello_app}", HELLO_APP.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_prompt_interpolates_templates() {
        let p = generation_prompt();
        assert!(p.contains("name: \"counter\""));
        assert!(p.contains("widget App"));
        assert!(!p.contains("{counter_pac}"));
    }

    #[test]
    fn templates_cover_two_entries() {
        assert_eq!(TEMPLATES.len(), 2);
        assert_eq!(TEMPLATES[0].0, "counter");
        assert_eq!(TEMPLATES[1].0, "hello");
    }
}
