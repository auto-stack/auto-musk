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
