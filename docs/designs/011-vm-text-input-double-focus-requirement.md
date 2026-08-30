# 011 — VM text_input 双焦点/键盘双投递 上游修复需求

> 来源：auto-musk PLAN-050（VM/Vue 渲染一致性第一批）用户验收发现。诊断链已在
> auto-musk 侧完成并实证，缺陷定位在 auto-lang 的 iced 渲染层（widget 焦点生命周期），
> 与 KD-048 族（VM 静默退出、debug RC canary）同属 VM widget 层债。本文为携往
> auto-lang 立项的完整问题说明与方案要求。

## 一、症状（用户实测 + 截图证据）

musk 登录页形态：根 widget 里 `if !authenticated { LoginPage }` 条件渲染子 widget，
子 widget 视图含两个 input（`value: .username` / `.password`，各挂 oninput handler）。
`auto run --render=vm` 真键盘实测：

1. **双焦点**：点击 password 后，username 与 password **同时**显示焦点环、光标双闪；
2. **键盘双投递**：在 password 输入 "admin"（5 键），username 框同步追加同样 5 键，
   变成 "adminadmin"；password 自身正常显示（掩码）"admin"；
3. **状态双污染**（AutoUI state 实证）：`username="adminadmin"`、`password="admin"`
   ——两个框各自触发自己的 on_input：UsernameChanged 收到 "adminadmin"、
   PasswordChanged 收到 "admin"。即键盘事件被**两个焦点框各自独立消费**。

## 二、已排除（诊断链，均以探针/对照实证）

| 层 | 结论 | 证据 |
|---|---|---|
| 视图层消息归属 | ✅ 正确 | build 后遍历 View 树断言：username 节点 on_change=UsernameChanged、password 节点 on_change=PasswordChanged（各自携带，无串） |
| 派发回写层 | ✅ 正确 | `on_with_input_for("LoginPage","UsernameChanged",AAA)` / `("PasswordChanged",BBB)` 各写各字段（临时探针，已撤，模式见提交 ff7eb126 邻域） |
| 对照组 | ✅ 正常 | 003-converter（根级双输入、无条件、无子 widget）同构建真键盘联动正常（用户实测） |
| 输入管线回归 | ❌ 不存在 | 二分对照：8bc51ed4b（T3+476）与后续各构建 converter 行为一致 |

**缺陷限定形态**：条件渲染的**子 widget** 内的多 input。根级双输入不受影响。

## 三、已试无效的修复

- 为每个 text_input 派生稳定唯一 Id（`placeholder+width+password`，auto-lang
  master@1a8516b5b，`build_input_shape`）——双焦点依旧。说明不是"缺 Id 导致状态
  合流"这么简单：要么 Id 未真正参与该路径（子 widget/条件包装下 diff/焦点路由），
  要么失焦广播机制在该形态下失效（见根因方向 1）。

## 三点五、追加（2026-08-29 用户实测，Plan 483 落地后）

- **Tab 焦点遍历缺失**：username 框按 Tab 不跳 password。DOM 免费提供 Tab 焦点
  环；iced 需渲染器显式实现（按树序 Tab/Shift+Tab 在可聚焦 widget 间移动）。
  Plan 483 的「唯一稳定 Id + 渲染期登记表 + 聚焦点改址」基建正好是它的承载面——
  在登记表上实现焦点环遍历即可。
- （Enter 提交已在 musk 源级修：login.at password 框 `onenter: .Submit`，
  plan-050-dev@d4814df——单行声明，不属上游缺陷。）

> **2026-08-30 结案追记**：
> ① Tab 流——P491 机制级交付后用户侧观感仍是「password→user 可跳、反之不行」，
>   根因=运行时二进制停在 P487 时代（旧 483 回落语义=Tab 恒聚焦登记表首项，
>   恰好伪造成"反向可跳"）。auto-lang master(P491) 与 plan051 专修分支
>   （auto-musk-dev）合流重建后，真键盘双向 Tab 循环用户实测通过，本条关闭。
> ② Enter 提交升级为 form 级机制：button 新增 `variant: "submit"` 行为语义
>   （auto-lang extract 层 wire_form_submit——widget 视图内任意未声明 onenter
>   的 input 自动接线提交钮 onclick，两轨同源；vue 轨生成 @keyup.enter，
>   VM 轨走 on_submit 既有通道）。login.at 已迁移（移除 password 独享 onenter）。
> ③ 顺带补齐 KD-048「KV 会话恢复断裂」：VM 轨 raw localStorage 家族原先
>   纯内存不落盘，现与 Storage.* 同 load-once + write-through 契约——登录页
>   「记住上次登录用户名/密码」（login.at .Init 预填 + auth_store.at 成功
>   登录时写入 musk_login_username/password）依赖此修复跨重启生效。

## 四、次要伴生缺陷（同批顺修）

MCP `autoui_type` 的 vnode id→action 归因错位：对第二个 input 的 vnode id 执行
type，实际派发**第一个 input 的 handler**（仪器化实证：password id →
on_with_input_for 收到 event=UsernameChanged）。错位发生在 mcp_server 持有的
UiNode 快照树（snapshot_builder::traverse_view + DebugIdMap 的 path 对齐）或
find_node 归属——与主缺陷无关但同在 id/快照层，建议同批加 path 对齐回归测试顺修。

## 五、复现样点（最小化，立项第一步）

新增 example（如 `examples/ui/0XX-two-inputs-child`）：

```
根 widget：
    if !.authed { LoginChild } else { text "in" }
子 widget LoginChild：
    model { var user str = ""; var pass str = "" }
    view {
        input { value: .user, oninput: .UserChanged, placeholder: "user" }
        input { value: .pass, oninput: .PassChanged, placeholder: "pass" }
    }
```

- 预期：真键盘点击 pass 框 → 仅 pass 有焦点环；user 值与焦点不变。
- 现状（复现）：双焦点环 + 击键双投递（user 框同步追加 pass 的击键）。
- musk 真实样点：`src/front/login.at`（子 widget）+ `src/front/app.at`（条件装配）；
  auto-musk worktree `plan-050-dev` 可对照（掩码修复在 4e184b9）。

## 六、根因方向（定位起点，按序排查）

1. **焦点生命周期**：点击第二个 input 时，第一个 input 的 `State.is_focused` 未被
   清除。查 iced 0.14 `shell.request_focus` → 旧焦点框失焦广播在「render_child_widget
   构建的子 widget View 子树 + 条件包装」里是否失效。对照差异点=根级双输入正常。
2. **键盘投递**：iced 向 widget update 递全部键盘事件、text_input 按
   `state.is_focused()` 过滤——双框都通过过滤即双焦点本身；修好 1 即连带修好 2。
3. **Id 传导核实**：1a8516b5b 的派生 Id 是否真正到达子 widget 路径的 widget 实例
   （tracked/untracked 双构建路径、条件重渲染时 Tree diff 的节点复用是否稳定）。

## 七、方案要求与验收

- 修在 auto-lang iced 层（`crates/auto-lang/src/ui/iced/renderer.rs` /
  `aura_view_builder.rs`）；**不动 .at 单一真源契约、不动 musk 侧**。
- TDD：最小复现 example + 回归测试先红后绿。断言：聚焦第二个 input 后第一个失焦；
  键盘文本只进聚焦框（example 级探针或 widget 单测，沿 plan050 期间
  tests/plan050_probe_login*.rs 的遍历断言模式，可从提交史找回）。
- 验收清单：
  - [ ] 最小 example：单焦点、无键盘双投递
  - [ ] musk 登录页实测：双框独立、admin/admin 登录可完成
  - [ ] 003-converter 双向联动不回归
  - [ ] `cargo tf` 全绿 + `cargo test -p auto-lang --lib --features ui-iced` 全绿
  - [ ] （顺修）autoui_type 对第二个 input 派发正确 handler + path 对齐回归测试

> 2026-08-30 注记（auto-lang Plan 491 T7）：P483-3 真人清单追加「musk 登录页
> Tab 流」——Tab/Shift+Tab 焦点环遍历已由 auto-lang Plan 491 机制级交付
> （iced_test 七测），真键盘 username→password Tab 切框复验并入该清单。

## 八、流程

按 AGENTS worktree 规则执行（`plan-<NNN>-dev`），每能力项独立提交，no-ff 合并。
参照先例：009-vm-slot-substitution-requirement → Plan 476（slot 替换机制）。
