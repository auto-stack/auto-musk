# UI 组合与壳导航规范（UI Compositions & Shell Navigation）

> 来源：PLAN-078 SD-03（2026-09-20）。沉淀导航壳、浮层弹窗及各个业务页面视图在
> Vue 与 VM/Iced 双端平权渲染与组合验证的契约规则。

## 1. 全局导航壳与侧边栏（NavSidebar）

1. **双态展示**：支持展开与折叠（rail 模式）两种形态切换。
2. **图标与文字分派**：折叠状态下隐藏文字标签，仅保留导航图标及活动指示器；
   展开状态下显示完整图标与对应名称。
3. **尺寸与布局**：支持外部传入 `width_class` 覆盖宽度；不依赖浏览器专有 CSS
   动画或无 VM 对应样式的伪类，在桌面各窗口尺寸下保持内容可见不溢出。

## 2. 浮层与弹窗契约（Popovers & Dialogs）

1. **工作区选择器（WorkspaceSelector）**：
   - 采用 Popover 模式锚定在导航侧边栏底部的工作区触发按钮旁。
   - 触发时展开工作区列表与切换/管理菜单；失焦或点击外部时自动收起。
2. **设置菜单（SettingsMenu）**：
   - 受控 Dialog 弹层，提供基本配置选项。
   - 打开时捕获焦点，支持 Esc 键以及显式关闭按钮退出。
3. **删除确认弹层（DeleteConfirmDialog）**：
   - 采用 Alert-dialog 模式承接高危操作二次确认。
   - 必须提供“取消”与“确认删除”两个明确动作，阻止对下层页面的非预期点击。

## 3. 业务页面组合契约（Business Views）

1. **FilesView**：
   - 左侧为目录树导航（`FileTree`），支持逐级展开/收起目录，根据文件后缀名映射树图标。
   - 右侧为只读预览面板，分流分型处理 Markdown、代码文件、多媒体或不支持格式提示。
2. **SpecsView**：
   - 左侧分类栏，右侧规格详情与 Markdown 渲染区。
   - 提供状态切换与编辑保存接口骨架，双端渲染无占位缺件。
3. **WhitelistView**：
   - 提供安全目录白名单的展示、添加与移除表单，校验规则两端一致。
4. **WikiView**：
   - 知识库层级树与文档浏览/编辑面板，支持富文本/Markdown 组合展示。
5. **PlansView**：
   - 计划路线图与阶段状态列表展示，支持筛选与详情展开。
6. **ChatsView**：
   - 消息流容器与底部 MentionInput 一体化输入区域的组合，保持滚动定位与输入协同。

## 4. 多 Store 与画廊隔离适配（Single-Source Multi-Store Boundary）

1. 在同源测试画廊（`examples/musk-widgets-gallery`）中，业务视图通过 AST 级别
   适配器（如 `materialize.mjs` 中的 in-memory `ForgeStore`）进行环境隔离。
2. 避免在没有完整运行时服务时调用真实网络端点，同时保证组件模板在编译和链接阶段
   具有完备的消息声明和类型注解，消除歧义性符号错误。

## PLAN-080 增量：settings 向弹层一律就近 popover（SD-03）

- **契约**：settings 类弹层（设置菜单及同族"就近触发、非阻断"的操作浮层）
  一律 **锚定 popover**：锚定触发件、非 modal（不遮罩不居中）、外点/ESC
  家族语义关闭（auto-lang Plan 422+ popover 原语；DSL `popover` +
  `popover-trigger(as_child)` + `popover-content` 嵌套形态，触发件无
  onclick 时解释器注入 toggle）。destructive 确认类（删除等）仍走
  alert-dialog modal（阻断语义正当）。
- **反例（已修）**：settings_menu 曾为 dialog 受控居中 modal——实机对拍
  双轨均判 viewport-centered 红（080-live-settings-popover 收据）。
- **依据**：PLAN-059 T9 弹层家族 + PLAN-080 用户裁定（"针对所有 settings
  类型的弹窗的通用修改"）；musk 1488482。

## PLAN-081 增量：WorkspaceSelector 显示名/切换完成语义 + 消息块与工具卡排布（SD-01/SD-04）

- **WorkspaceSelector 触发器显示（SD-01，AC-01）**：触发器一律显示**最终
  目录名**（name 优先/path 兜底/占位文案殿后），完整路径仅存悬浮 title
  位（EE03 label 组合的 tooltip 位）；弹层列表项显示工作区名。禁止回到
  080 T-02 的"触发器显示完整路径"旧裁定。
- **切换完成语义（SD-01，AC-02/03/04）**：web=页面 reload（主链保留）；
  **VM=store 级刷新链**（`platformRefreshAuth()` 先于 `SetWorkspace` →
  `SetWorkspace` 幂等回填+会话域清场+`LoadSessionList` →
  `PlansStore.Reload`；boot 链 refreshAuth 前置）。Choose 与 PickFolder
  两条路径共用同一接线顺序，不得只改其一。
- **消息块间距（SD-04，AC-13）**：聊天块（文本/思考/工具）垂直间距由
  **每块 wrapper 的 `mb-[12px]` 承担**；禁止依赖容器 `gap-*`——VM 渲染器
  把 for 子树包装为列、gap 只作用于直接子级（双轨等价要求 margin 方案）。
- **工具卡 header 排布（SD-04，AC-14）**：单行结构
  [图标][工具名][摘要串][状态][chevron]；工具名/摘要串 `shrink-0` 紧跟
  排布，**禁止 min-w-0/truncate 弹性盒**（VM 撑宽致文字盒内居中）；摘要
  为 ingest 预拼接单串（`getToolSummaryText`），色
  `text-muted-foreground`（暗于工具名 foreground）；类串禁混入
  `font-mono`（VM 解析阻断同串颜色类，等宽观感由 VM 默认字体/web scoped
  CSS 提供）。长参数截断由 web scoped CSS 兜底。
- **依据**：PLAN-081 r5/r6 实机定罪与像素实证（收据
  docs/reports/ui-parity/081-workspace-switch-ux.md r5/r6 节；musk
  0b70043/4945f61）。

## PLAN-084 增量：会话列表分组折叠与默认形态 + 聊天头部终态 + composer 合一（SD-01/02/06）

> UAT①-⑪ 实机迭代终态（2026-09-22 合入）。评审见 docs/plans/archived/084-*。

### 会话列表（SD-01，§1 NavSidebar list slot）

- 行=标题+相对时间+N 条；按 **今天/昨天/更早** 分组（后端 summary 现算
  day_group/time_text/date_text，本地时区；前端只做纯分组 sessionGroupsOf）。
- **"更早"默认折叠**：头行常驻（label+数量+lucide chevron），点击切换；
  折叠时会话行不生成（store 域 hideEarlier 参数）。
- **hover 工具组**：✎ 重命名 / 📥 归档（归档视图 📤 取消）/ × 删除（两步确认）；
  mouse-area 子树仅支持单臂 if（`<else>` S002），行内选中样式预计算进
  store（scls 字段），视图禁 if/else 对。
- **已归档过滤开关**：NavSidebar 头部 🗂（lucide archive），开启切
  archived_list 数据面。
- 会话行 hover 亮度减半（bg-accent/40；选中态 primary/20）。

### 一级导航栏（rail）

- **默认收起**（w-16 图标态，`rail_collapsed=true`），ToggleRail 随时展开；
  rail active 用 computed bool 绑定（表达式 prop `active: .x == "y"` VM 不消费）。

### 聊天头部（SD-02，§3 ContentHeader）

- 标题=会话名（text-lg、w-[250px] truncate、可编辑），无 agent 角色徽章
  （一会话未来可能多 agent，角色标识易误导——用户裁定）；流式状态点常驻。
- actions=✎ 重命名（标题行内编辑）+ 🔍 搜索（点击展开 middle 搜索框）+
  📥 归档三 icon（lucide）；SessionInfo（chat id/token 成本）退役，token 成本
  入口找回登记 KNOWN-DEBT。

### composer（SD-06）

- **合一容器**：textarea 无独立边框融入大圆角 composer-box——VM text_editor
  仅在 style 含文本色类时应用无边框样式臂（`text-foreground` 必挂）。
- **思考/审批菜单=popover 浮层**（popover/popover-trigger(as_child)/
  popover-content，settings 同款）：absolute 向上弹出 VM 不消费（菜单掉文档流
  被底缘裁剪）。菜单项 onclick 走 onpick/onpickapproval 落库链。
- AI 气泡身份头=名称+角色 badge（漏斗 aname/ainit 预计算字段直读；视图
  computed 内 fn 调用静默返空——坑①，AgentAvatar 组件内同踩）。

