# workspace-ui — 五栏目工作区界面契约

> 来源：PLAN-085。适用于会话、计划、规范、知识库、文件五个主栏目。
> `src/front/*.at` 是界面实现真源；生成的 Vue 与 VM 必须从同一份 Auto 源产生。

## 信息架构

1. 工作区采用三级布局：64px 主图标轨、240px 栏目上下文侧栏、可伸缩内容区。
2. 内容区顶栏高 48px，标题使用 16px semibold；标题允许截断，操作区必须始终可见。
3. 主图标轨默认折叠。当前栏目只用柔和主色底与主色图标表示，避免大面积强调色。
4. 会话正文与输入框共享最大 960px 阅读容器，并在窄窗口保留 16px 水平边距；消息气泡继续遵守 70% 最大宽度约束。

## 栏目职责

- **会话**：侧栏组织会话历史，内容区承载消息、思考、工具、审批和输入。用户消息用柔和强调色，AI 内容保持开放阅读面。
- **计划**：侧栏负责选择与创建计划，内容区展示计划正文、状态和操作。空列表必须提供“新建计划”，加载失败必须提供错误说明和重试入口。状态徽标使用本地化文案（中英资源随语言切换，未知状态回退原始值）；进入栏目时选中计划的详情正文与磁盘内容同步（PLAN-091）。
- **规范**：概览解释规范体系并提供模块文件入口；文件模式显示规范树和正文。规范树为嵌套可折叠树（目录展开/收起，顶层目录默认展开）；md 文件正文经 Markdown 渲染，非 md 文件纯文本降级；正文加载失败为独立可见状态（独立于树的加载状态）。加载中、加载失败、无内容是三个独立状态（PLAN-091）。
- **知识库**：侧栏提供搜索、树和新建入口；无页面时必须解释用途并允许新建页面。
- **文件**：侧栏显示工作区文件树，内容顶栏显示当前路径；未选择文件时说明可预览的内容类型。

## 规范与索引的权威边界

1. `docs/specs/` 中的模块规范是当前系统知识的权威来源，描述现行行为与约束。
2. 六区 ledger 及其他索引只保存条目定位、关系、状态和历史视图，必须从模块规范或交付记录派生。
3. 相同知识不得在规范与 ledger 中分别维护完整正文。发生差异时以模块规范为准，并重新生成或修正索引。

## 状态与交互

- 所有数据视图必须区分加载、空、失败和有内容状态，禁止用“加载中”长期代替空态。
- 空态先说明当前栏目用途，再给出一个主要动作；按钮沿用既有业务消息，不改变后端状态值。
- 新增用户可见文本必须同时提供中文和英文资源。
- 1024px 和 1280px 宽度不得出现页面级水平滚动，内容区可独立滚动。
- **会话 id 信息入口（PLAN-092 T-01）**：聊天头部 actions 槽常驻 i 信息钮
  （Info 图标，展开态反色），点击展开/收起弱色单行信息条——`chat.chatId`
  标签 + 当前会话 id（`font-mono truncate`，空态显示 `—`）+ 复制钮
  （Copy→CopyCheck 成功态常驻至下次复制；`.at` 不表达 setTimeout 自动
  复位）。会话 id 是调试/汇报锚点，不得只存在于存储层。

## 双端与主题边界

1. 布局优先使用 AutoUI 的 `row`、`col`、`text`、`button` 与已有组件，不依赖只在浏览器生效的 DOM 结构。
2. 品牌 `primary`、`primary-foreground` 和 `ring` 由 `pac.at` 的 `theme` 共享给 Vue 与 VM。
3. 当前 AutoUI `theme.colors` 只有跨模式的单份局部映射；Web 的明暗 surface token 暂留在 `inject_styles.web-only.ts`。AutoUI 支持分模式主题后，应迁回共享主题声明。
4. Web 专用样式不得改变信息结构或核心操作；VM 不支持的视觉细节应自然降级。

## 已知边界

- **KNOWN-DEBT 092-D1（PLAN-092）**：VM 轨 `dom.copy_text` native 静默
  失效——会话 id 信息条的复制半臂在 web 轨工作（`navigator.clipboard.
  writeText` 对拍实证），VM 轨点击后无剪贴板写入（`let` 绑定形态 handler
  中止、裸调用形态空转；接收者字段读正常）。根因在 auto-lang 2926
  native 于视图 handler 上下文的分派，随上游收敛；收敛前 VM 轨会话 id
  以可见性为限（`chat_message.at` 消息复制同 native 同病）。

## 验证

- Auto 生成和 Vue 生产构建必须通过。
- `node scripts/ui-parity.mjs check` 必须无新增静态错误。
- 五栏目需在 1024px 与 1280px 检查导航、标题、空态和水平溢出。
- VM 运行证据应绑定当前修订；若被已登记的引擎基线缺陷阻断，必须保留复现命令、错误和受影响范围，不得把 Web 结果表述为双端通过。

## 关联实现

- `src/front/app.at`
- `src/front/nav_sidebar.at`
- `src/front/content_header.at`
- `src/front/chats_view.at`、`chat_message.at`、`mention_input.at`
- `src/front/plan_progress.at`、`plan_progress_store.at`（PLAN-097）
- `src/front/plans_view.at`、`specs_view.at`、`wiki_view.at`、`files_view.at`
- `src/front/inject_styles.web-only.ts`

## 应用设计模式（PLAN-093）

- 会话栏目增加「应用设计」模式：进入暂收会话列表（进出保存/恢复折叠
  偏好）；工作台三段布局（结构/画布/对话右置）与窄层规则见
  `modules/app-studio-ui.md`（本模块不复制其契约）。
- 消息链与 composer 单实例：模式切换不重挂——草稿、展开态、审批卡保持；
  退出模式恢复原侧栏折叠态与会话内容。

## 选择器当前项标记（PLAN-101）

- workspace 选择器弹层中当前 workspace 行必须以可见标记呈现（✓；背景
  高亮受 VM 动态 style 字段引用引擎限制，恢复后补，登记 PLAN-101 §10）。
- VM 轨实现约束：模板 `if` 条件不得直读 store/computed 或循环变量成员
  访问（断点族）——当前项标记经 handler 域预构建行数据（`ws_tag_current`
  纯 fn：JSON 克隆 + 动态键写 `mark`，静态名 SET_FIELD 在 VM handler
  续体 RuntimeError）+ 模板文本位成员读（`text .w.mark`）实现；web 轨
  同函数同语义（单源双轨）。
- `Init` 与 workspace 解析链存在时序竞争（Init 先 resume 时 store 仍
  空）：`app.at` 解析完成后定向 emit `WorkspaceSelector.WsResolved(id)`
  补拍重标；`Choose`/`PickFolder` 成功链内即时重标（VM 下 reload 为
  no-op，弹层保持开启）；`UpdateQuery` 过滤保留行数据已带标记。

## 计划进度呈现（PLAN-097）

- **PlanProgressCard（`src/front/plan_progress.at`）**挂在 chat_message
  的 spawn_relay 卡位（RelayRunBox 上方，run_id 判定与 PLAN-100 双通道
  同源），自持 Attach/Detach 生命周期，消费独立
  `plan_progress_store.at`（单飞 1Hz 轮询 + 代次拒收 + 终态停拍）：
  - 相位轨（规划→实施→复审→沉淀→已交付）+ 当前相位高亮；
  - plan_id · r{revision} · phase · attempt · 修复 c/limit 摘要行
    （handler 域预构建标签，VM/Vue 同义）；
  - 交付检查点行（prepared/landed/archived/cleaned
    冻结序，✓/○）；
  - blocker 横幅（blocked/needs_replan 事实可见）；
  - 断连陈旧横幅（最后成功快照 + 明确陈旧标记）与
    contract_version 不兼容横幅（未知版本显式呈现，不默认成功）；
  - 可展开阶段记录（stage/attempt/outcome + reviewed_commit + receipt）；
  - 终态显式标「已完成阶段记录」，不伪装正在运行。
- **不另建主栏目**：不打断 composer 草稿、审批与 Canvas 状态；五栏目
  布局保留；中英 i18n 键齐备（planProgress.*）。
- 只有 A 控制器 delivered 事实才显示已交付；检查点缺件不绿。

## PLAN-106：交付进度卡的文档源策略

交付检查点固定为 prepared/landed/archived/cleaned；新 run 不等待 ledger_refreshed。旧收据里的 ledger_refreshed 可作历史信息，不能成为缺件提示或替代 landing。应用自动索引状态不影响 delivered，源文件与已落地提交仍须核验。
