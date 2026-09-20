# ui-default-styles — 默认样式三方对账与主题字体收敛（PLAN-075 SD-01）

> 来源：PLAN-075 r1（基于 bde98f1 开发）。  
> 核心交付：Design 22 / Musk Web-Only CSS / VM 默认样式三方零漏项对账、`pac.at` 品牌主题声明、离线字体栈收敛、标题 `tracking-tight` 补齐、引擎规约明确移交 PLAN-076。

## 契约

1. **三方合同对账与零漏项原则**：
   - Design 22 默认规范（§2 排版、§3 表单控件、§4 容器与兜底边框共 24 条）必须与前端注入 CSS（`inject_styles.web-only.ts` 15 条规则）及 AutoUI VM 运行时引擎达成逐行映射。
   - 所有条目必须清晰归属，无任何悬挂或未分配条目（详见 `docs/reports/ui-parity/075-default-style-map.md`）。
   - 严禁在 Musk Web-Only CSS 中擅自注入与 AutoUI 基础组件冲突的全局覆盖。

2. **品牌主题双端声明规范**：
   - 全局品牌色通过项目根及 Gallery 的 `pac.at` 中的 `theme: {}` 块统一定义，双端同源消费：
     ```auto
     theme: {
         extends: "scaffold",
         colors: {
             primary: "238 55% 58%",
             primary-foreground: "0 0% 100%",
             ring: "238 55% 58%",
         },
     }
     ```
   - AutoUI 编译器（Vue 生成器与 AutoVM 运行时）统一解析该块。VM 模式启动时激活 `scaffold` 主题调色板，Button 与 Input 组件自动获得该强调色（HSL 238 55% 58% -> Hex `#5963cf` / RGB 89, 99, 207）。

3. **字体安全与离线收敛规范**：
   - 前端严禁通过 `@import url('https://fonts.googleapis.com/...')` 等在线链接动态拉取外网字体。
   - 全站统一收敛至离线系统无衬线字体栈：
     ```css
     font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
     ```
   - 杜绝因外网拉取超时、DNS 阻断或沙箱断网导致的排版抖动（FOUT/FOIT）或渲染异常。

4. **基础属性与排版对齐**：
   - 标题组件：`h1` 与 `h2` 默认预设统一包含 `tracking-tight`（负字距紧凑排版），与 AutoDown `autodown_heading_style` 保持视觉一致。
   - 裸控件兜底：对于未显式声明 class 的控件，双端必须遵循 Design 22 §3 约定的默认盒模型与圆角/内边距标准。

5. **AutoDown 引擎规约移交边界**：
   - 涉及 Markdown 块间节奏（12px 间距）、暗色 token 映射、中性灰调色板、五色强调色、行内排版分档（25.3px）、双档高亮及编辑器编辑壳交互共 10 条规约，正式界定为 AutoDown 引擎职责。
   - 上述 10 条规约全量移交 **PLAN-076** 在 `autodown-engine` 中实现，Musk 侧不做前端补丁兜底。

6. **双端证据与测试门禁**：
   - 针对基础控件样式与状态（`style-controls-login`、`style-badge-status`、`style-button-dialog`），必须在 `tests/ui-parity/cases.json` 注册独立用例与确定性 fixture。
   - 门禁指标：
     - 静态检查：`node scripts/ui-parity.mjs check` 必须 0 error。
     - VM 模式：`node scripts/ui-parity.mjs run --plan 075 --mode vm` 必须达到 `snapshot-ok`，Reset 按钮交互 spy PASS，基线截图成功留存。
     - Vue 模式：`node scripts/ui-parity.mjs run --plan 075 --mode vue` 必须启动 Vite 与 AutoVM 后端并返回 `http-ok`。

## 关联实现与证据

- 样式映射全表：`docs/reports/ui-parity/075-default-style-map.md`。
- 双端验证证据收据：`docs/reports/ui-parity/075-evidence.md`、`tmp/ui-parity/PLAN-075/*.json`。
- 主题与样式配置：`pac.at`、`examples/musk-widgets-gallery/pac.at`、`src/front/inject_styles.web-only.ts`。
- 上游框架修复：`auto-lang` commit `3edcf5fcf` (`aura_view_builder.rs`, `ui_gen/rust.rs`)。
- 截图资产：`examples/musk-widgets-gallery/src/front/tests/screenshots/plan075-*.png`。

## PLAN-080 增量：VM 裸 button 默认 chromeless（SD-01）

- **契约**：未指定 `variant`/`size` 的裸 button 一律 **chromeless**（preflight
  等价：透明底、无边框、内容贴边）——三端同步（VM 解释器臂 / Rust transpile
  臂 / Vue cva，后者 `defaultVariants` 已移除）。外观只来自显式 variant 或
  显式 class。
- **显式档**：`variant="default"` 保留 PLAN-571 UA 预填等价基线（muted 填充
  + 发丝描边）；`size="default"` 显式得 h-10 px-4。缺省/未知 size 无预设。
- **事实源**：auto-lang `crates/auto-lang/src/ui/style/variants.rs`（单一
  事实源，三表互锁 `plan571_*` 测试锚定）。
- **VM `__json_object` 字段语义**（SET 入册，PLAN-080 T-06）：**GET 缺键返
  null（安全），SET 缺键硬崩**（engine.rs PLAN-044 分支）。消费后端稀疏
  JSON（持久化消息块等）时：读一律 `x.f ?? 兜底`；补写必须**构造新对象
  整体替换**（容器键存在时 SET 合法），严禁 `b.f = v` 原地补缺失键。
- **依据**：2026-09-20 桌面实机验收五项裁定（用户：统一重置而非逐按钮
  覆盖）；auto-lang 0cbf5a031；musk 1488482。
