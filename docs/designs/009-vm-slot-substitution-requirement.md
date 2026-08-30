# 需求说明：VM 前端管线 slot（内容模板）替换机制

> 出处：auto-musk PLAN-050 T5 调查移交（2026-08-29）。携带本文档到 auto-lang 立项，
> 建议挂 KD-048 UPSTREAM① 的正式清偿批次。本文只写需求与勘验事实，实现方案由
> auto-lang 侧立项勘验裁定（文末附三个候选方向供参考）。

## 1. 背景与症状

musk（auto-musk）前端把二级导航收敛为 `NavSidebar` widget：widget 视图内声明
`slot(name: "list")` / `slot(name: "actions")` 插座，调用位（chats_view.at 等）
以 `slot(name: "list") { … }` 传入会话列表/操作按钮填充。

- **Vue 轨**：正常。ui_gen/vue.rs 的 codegen 完成了插座/填充对接。
- **VM(iced) 轨**：整个 slot 填充子树不渲染——会话列表、新建/删除按钮全部不可见，
  ChatsView 只剩内容区。已在 auto-musk KNOWN-DEBT-AND-RISKS.md 登记为
  **KD-048 UPSTREAM①**，本文是其清偿需求。

复现实测（2026-08-29，release auto.exe，`AUTO_BACKEND=http://127.0.0.1:8081
auto run --render=vm`，admin/admin 登录后）：chats 视图主区直接从 app rail 接内容
面板，无任何二级导航；`autoui_vtree` 导出中无 NavSidebar 的 slot 子树节点
（证据：auto-musk `tmp/plan050-survey/04-session-nav.vtree.txt`、
`01-rail.png`、`03-folder-picker.png`）。

## 2. 现状勘验（本次调查的代码事实）

`"slot"` 的处理点现状分布（auto-lang master 09e64c391）：

| 位置 | 现状 |
|---|---|
| `crates/auto-lang/src/aura/types.rs:223-237` | `slot_outlet_names()`：widget 可声明插座名；配套校验"填充给了无插座 widget"会 warn——**校验在，消费缺** |
| `crates/auto-lang/src/aura/schema.rs:2478` | `slot` ElementDef 已声明 |
| `crates/auto-lang/src/trans/rust.rs` | a2r 转译侧有 slot 特判（Rust 后端轨不受影响） |
| `crates/auto-lang/src/ui_gen/vue.rs` | vue codegen 侧完成对接（vue 轨正常的原因） |
| `crates/auto-lang/src/ui/aura_view_builder.rs` | **零 slot 处理**——VM 前端视树构建不识别 slot |
| `crates/auto-lang/src/ui/iced/renderer.rs` | **零 slot 处理** |

关键调用链缺陷点：`aura_view_builder.rs` 的 widget 实例化两个出口
（约 533 行与 783 行 `registry.get(name)` → `render_child_widget`/
`render_child_widget_tracked`）拿到调用位的 children/slot 填充后**直接 return
子 widget 自身视图**——填充从未传入。子构建器构造（2798 行起）：

```rust
let child_builder = AuraViewBuilder {
    bridge: self.bridge,
    widget_name: child_widget.name.clone(),
    override_state_obj_id: Some(child_state_id),  // ← 作用域已切到子 widget
    ...
};
child_builder.build(&child_widget.view_tree)
```

即：**VM 前端管线没有 slot 替换机制**。这不是某处 bug，是缺一整块
"跨作用域内容模板求值 + 槽位拼接"的特性。

## 3. 需求（功能语义）

1. **插座填充**：widget 调用位的 `slot(name: "X") { subtree }` 在 VM 轨渲染到
   子 widget 视图中对应 `slot(name: "X")` 插座位置。
2. **父作用域求值**：填充子树按**调用位（父）作用域**求值——父的 store 单例、
   `use` 导入、state 字段、computed 均按父解析（musk 场景：填充里的
   `.store.session_list` 是父 ChatsView 的 ForgeStore，不是 NavSidebar 的）。
3. **逐帧重求值**：父状态变化（如新会话入列表）后，下一渲染帧填充内容跟随更新。
4. **事件保留父绑定**：填充子树内 `onclick: .ParentFn($event)` / `onclick.stop`
   等路由到父作用域 handler（现 VM 事件路由的父侧语义不变）。
5. **默认插槽**：空名 `slot { … }`（default）与具名插槽并存。
6. **校验沿用**：填充给了无插座 widget → 沿用 `slot_outlet_names` 的 warn；
   name 不匹配的填充不渲染。
7. **范围外**：teleport、动态 slot 名（`slot(name: expr)`）、多层 widget 嵌套的
   slot 透传——后续批次。

## 4. 候选设计方向（供立项勘验，非定案）

- **方向 A｜父预求值 + 标记拼接**：父构建器在 widget-call 处先把各填充子树
  `build()` 成父作用域的 View 子树；子构建器把 slot outlet 渲染为唯一标记节点
  （debug_id/key）；构建完成后遍历子 View 树，把标记节点替换为父预求值 View。
  要点：View 树遍历/拼接工具、标记唯一性、替换后事件路由仍在父通道。
- **方向 B｜AuraView 数据级替换**：子 `view_tree` 的 slot outlet 节点直接替换为
  父的填充子树（AuraView 数据克隆），构建仍走子构建器；填充内部绑定的作用域
  需要 AuraViewBuilder 支持**作用域栈**（父 state 优先/子 state 兜底），改动面
  在状态解析层。
- **方向 C｜最小编号版**：先支持"纯展示填充 + 既有 emit 事件路由"（不做任意
  父作用域表达式），快速解 musk 界面④；完整语义留第二批。

## 5. 验收标准

1. musk 会话二级导航在 VM 可见：NavSidebar 头（"会话" + 新建/删除钮）+
   NavListItem 卡片列表（左对齐、两行、选中描边——类串已就位）。
2. 填充内事件生效：点击会话项切换选中、悬停删除钮删除会话（父 handler）。
3. 数据跟随：新建会话后列表出现新项（父状态逐帧重求值）。
4. auto-lang：新增单测（slot 替换树变换 + 事件路由 + 未匹配 name 不渲染）全绿；
   `cargo tf` 全量绿（含 ui-iced）。
5. musk 侧零源改动即可受益（.at 不动）；vue 轨门禁零回归
   （build strict / vitest 23+1 / 对拍 30/30 / vm-link-probe PASS / first-run alive）。

## 6. 证据与关联材料

- musk `tmp/plan050-survey/04-session-nav.vtree.txt`（VM 渲染树无 slot 子树的直接证据）
- musk `docs/plans/050-vm-vue-render-parity.md` 待澄清事项 4（PLAN-050 的裁定与影响面）
- auto-musk `KNOWN-DEBT-AND-RISKS.md` KD-048 行 UPSTREAM①（原始登记）
- musk `src/front/nav_sidebar.at` / `chats_view.at` / `nav_item.at`（现网用例：
  具名插座 + 填充内事件/父状态绑定，可直接作验收样例）

## 7. 补充需求：文本颜色继承语义（2026-08-29 追加，同批承接）

实测（auto-musk 主导航栏）：`.at` 里颜色声明在 **button** 上、内部 `text` 子节点
无颜色时，vue 轨靠 CSS 继承正常着色；VM 轨文字落到 **OnPrimary 暗色
rgb(15,23,42)**（压在 bg-secondary rgb(30,41,59) 上不可读）。renderer 的 Plan 409
§8 继承模拟（找按钮类里的 StyleClass::TextColor 注入子树）未覆盖该路径。

需求：**颜色（color）按 CSS 继承语义在容器/widget→text 子树间传播**——text 子
节点无显式颜色类时，取最近祖先的已解析颜色；无任何祖先声明时才落全局默认。
同族可顺带评估 font-weight 等少量可继承属性的语义对齐。

验收：musk rail 导航项（颜色在 button 类上）在 VM 深色下文字为
text-primary(激活)/text-muted-foreground(非激活) 的期望色。musk 侧已有显式
text 节点着色的声明与之兼容（显式值=继承值，不冲突）。

## 8. 对 PLAN-050 的影响

本批次落地前，PLAN-050 的界面④（会话二级导航）以 musk 侧降级形态兜底
（T11 时裁定：内联 NavSidebar 壳或维持不可见）；本批次合入后 musk 恢复
NavListItem 组件形态，降级补丁拆除。
