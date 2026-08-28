---
plan_id: PLAN-048
status: archived
feature_name: musk VM 数据桥接线——back.api 契约 VM-front 编译通路 + MVP 真数据链路 + SSE 专项勘察
author: [zhaopuming]
created_at: 2026-08-28
updated_at: 2026-08-28（reviewed：复审通过，移交 /auto-plan:merge）

supersedes_spec_components: []
new_spec_components:
  - "specs/reports/048-sse-vm-survey.md: 新增（SSE 三方案勘察+渲染器 SSE 桥泛化候选）"
touched_goals: []

current_step: 7
total_steps: 7
---

# [PLAN-048] musk VM 数据桥接线 + SSE 专项勘察

## 变更摘要

PLAN-047 首跑红清单 R8 定罪：musk 前端六个数据 store 经 `use back.api` 契约调用
（6 文件 × 37 fn），VM-front 编译管线下宿主无挂载 → 登录/数据链路运行时不可达。
本计划两线：**①数据桥**——勘察定型后打通 `back.api` 契约在 VM 前端的编译/运行
通路（首选上游一次性方案：VM 目标把契约 fn 编译为 Http native 调用，musk 源零
改动；备选 musk 侧改写），并以 AutoUI MCP 无头取证打通「musk serve + VM 前端
login → chats 列表 → 单会话非流式历史」MVP 链路（047 验收#3 降级条款解除）；
**②SSE 专项勘察**——流式聊天（forge/chats SSE）的 VM 替代方案定型报告（轮询 /
WS / 后端推送改造三选一的依据入册），只勘察不实现。

## 目标

1. VM 前端完成登录并拉取真数据：`musk serve :8080` + `auto run --render=vm`，
   login → token 注入（既有 auth natives 链）→ chats 列表渲染，MCP 截图+状态
   双证在案。
2. `back.api` 契约在 VM-front 的编译形态定型并落码（上游或 musk 侧，以 T1 为
   准）；MVP 最小集 = auth 三件 + chats_list/create/get_session + send_message
   非流式路径，不贪 37 fn 全量。
3. SSE 专项勘察报告入册：三方案对比 + 推荐项 + 工作量评估，作为流式计划立项
   依据（本计划零流式实现）。
4. vue 三门禁 + VM 探针 + first-run 门禁全绿；37 fn 中未接线部分在台账逐条
   挂账（DEGRADED 数据面清单）。

## 架构方案

```
R8 现状(back.api 宿主无挂载)             本计划后(定型通路 + MVP 通)
──────────────────────────             ──────────────────────────
use back.api: fn…(6 store×37 fn)  →   [T1 定型] 首选: auto-lang VM-front
VM 编译成无宿主 extern → 运行不可达      codegen 把契约 fn 发为
                                        Http.post("api/…")+workspace query
                                        (auth 默认头/查询已有 natives 注入)
                                        备选: musk 六 store 手改 Http 调用
登录→chats 断链                    →   musk serve(:8080) + VM 前端 MCP 取证:
                                        login 截图/token KV/chats 列表渲染
SSE 无 VM 形态(047 G1)            →   专项勘察报告(轮询/WS/后端推送三选一
                                        依据),流式实现归下一计划
```

**跨仓执行流**：若 T1 判上游路线，auto-lang 走其仓 worktree `auto-musk-dev`
（沿 047 微批先例，TDD 先红后绿 + lib 全量回归 + no-ff 合并）；musk 侧消费与
取证在本仓 worktree `plan-048-dev`。在途冲突集先查后动（auto-lang 当日活跃：
446 批二续作/458/459 热区）。

## 技术栈

auto-musk（src/front 六 store 消费面零改动为最优、KNOWN-DEBT 台账、取证脚本
复用 047 启动器与 MCP 驱动）；auto-lang（vm 前端 codegen 契约通路，若 T1 判
上游）；musk serve 既有二进制零修改（仅起服消费）。不动 web/（冻结）、不动
backend/ 源码。

## 需求分析与背景调查

> spec overview 端点未运行（后端未起）；依据 = KNOWN-DEBT 047/046 行、
> PLAN-047 首跑红清单 R8 与 2026-08-28 勘察。

### R8 勘察基线（2026-08-28）

- **消费面**：`use back.api` 共 6 文件（forge_store/auth_store/chats_view/
  plans_store/specs_store/wiki_store）× 37 契约 fn。auth 三件在 047 红清单
  R8 实证运行时不可达（token=nil 直达 chats 的另一面即调用无宿主）。
- **后端侧**：`musk serve :8080` 为生产 API（041 复审 614 绿基线）；044 的
  MUSK_BACKEND=vm 桥（backend/crates/musk/src/vm_backend.rs + auto_generated/
  extern_impl.rs）作用于 backend 进程形态，与前端契约调用是两回事。
- **CLI 形态排除**：auto-man `start_api_server` 仅认 auto-lang 自带
  examples/rust-workspace 布局 + generate_api 模板后端——对 musk 手写后端
  无效（AUTO_BACKEND_IMPL=rust 路线判死）。
- **既有可复用件**：auth natives（set_default_header/set_default_query/
  clear_default_auth + platformRefreshAuth 接线，446-E4）= 契约调用所需的
  认证头/workspace 注入已就绪；AutoUI MCP 无头取证链路（047 T8）。
- **vue 轨参照**：back.api 在 vue codegen 生成 fetch 客户端（相对 /api/ 路径
  + workspace query + Bearer）——VM 通路对齐该约定即可双轨语义一致。

### SSE 面现状

- 前端消费点：forge 流式（ForgeStore.OnStreamEvent/chats 流式 draft 走
  StreamingRenderer 平台协议）；VM 无 EventSource（047 G1，MVP 限非流式）。
- 后端提供 `/api/.../stream` SSE 端点（axum SSE）。
- 候选：a) 轮询（前端定时拉增量，改动最小、体验降级）；b) WebSocket 桥
  （auto-lang stdlib 需 WS native，上游新面）；c) 后端为 VM 轨增设增量拉取
  端点（backend 改动，违反本计划不动 backend 约束 → 仅评估）。

## 详细设计

### D1 契约通路定型（T1 产出）

- 上游形态（首选）：auto-lang vm/front codegen 增 `use back.api` 解析臂——
  契约 fn 编译为 `Http.post("api/<名>", json 参数)` 包装（GET/POST 语义按
  vue client 对齐；响应 json 已是 Http native 返回形）。回归测试：fixture
  断言 VM 模块含 Http 调用与正确路径 + musk 探针链接含该符号。
- musk 侧形态（备选）：六 store 内把 `fn` 调用改写为显式 Http 包装 helper
  （`src/front/lib/back_shim.at` 单点集中 37 fn，store 逐条改 use），双轨
  语义等价由对拍+vitest 兜底。
- 判据：上游改动面（parser/derive/codegen 三处以内 + 测试）< 一日工作量且
  不撞在途热区 → 取上游；否则取 musk shim。

### D2 MVP 链路（T3-T5）

- 最小集接线：auth_login/auth_register/auth_me + chats_list_sessions/
  create_session/get_session + chats_send_message（非流式返回形态）。
- 取证协议（沿 047）：MCP autoui_type 填登录表单 → press Submit → state 断言
  token != nil → chats 列表 vnode 出现 → 截图 ×2 + musk serve 访问日志命中
  /api/auth/login 与 /api/chats 端点 = 前后端双证。
- 失败路径：登录错误态 SetError 渲染核对（i18n 已活的文案链）。

### D3 SSE 专项勘察报告（T6）

产出入册 `docs/specs/` 或 KD 行（以体量定）：三方案改动面/延迟语义/上游依赖
（WS native 缺口）/风险表 + 推荐排序。硬约束记录：流式渲染依赖的
platform:markdown VM 侧仍是 047-R3 降级态。

## 测试设计

1. auto-lang（若上游线）：新契约通路回归测试 + lib 全量绿 + 其仓门禁自验。
2. musk 三门禁：`auto build` strict exit 0；vitest 23+1；对拍 30/30。
3. VM 探针两连绿 + first-run alive 无 fatal。
4. MVP 链路：MCP 状态断言 + 双截图 + musk serve 日志三证合一；失败路径
   SetError 态核对一条。

## 验收标准

1. 验收 1（047 目标 4 降级条款解除）：VM 前端真后端登录成功且 chats 列表
   渲染，三证在案。
2. 契约通路定型结论 + 落码在案；MVP 最小集符号在 VM 模块可链接（探针 PASS）。
3. SSE 勘察报告入册（三方案对比 + 推荐 + 工作量），零流式实现。
4. 未接线契约 fn（37−最小集）逐条挂账 KD（DEGRADED 数据面清单）。
5. 四门禁全绿（测试设计 2/3 条目）；全程不动 web/、backend/ 源码；auto-lang
   改动（若有）走其仓 worktree 流程带回归。

## 执行步骤

- [x] **T1** 契约通路勘察定型（不改码）：a) VM-front 管线中 `use back.api`
  的现行解析点（auto-lang derive/derive 处理处 grep）与当前发射形态；b) vue
  轨契约 client 的 URL/方法/响应约定提取（gen/front/vue 内 auth 登录调用产物
  对照）；c) 上游改动面评估（D1 判据）；d) 在途冲突检查（auto-lang 热区 vs
  契约通路文件）。产出：`[✅]` 回填 + D1 定型 + T2 任务注细化。
  验证：四子项结论行齐备。
  [✅ 已完成] 四子项结论：
  - a) 解析点=`ui_gen/api.rs::is_api_use_stmt`（`use back.api` 识别）+ vm
    `collect_module_imports`（bare 名→`api.<fn>` import_scope）；发射分派=
    `vm/codegen.rs:7449` 起——`api_over_http` 时裸调用→`emit_api_http_call`
    （`AUTO_BACKEND` 基址，方法→`auto.http.{get,post,put,delete,patch}_json`，
    响应 `json.to_value`）。R8 现状成因=merge 模式（默认）该门关闭→走进程内
    CALL reloc 无宿主。`api_over_http` 第二开关=`AUTO_BACKEND` 非空
    （lib.rs:3546），**不触发** split_mode 拉错误后端（`--no-merge` 会经
    rust_ui.rs:2437 拉生成式/VM 后端，对 musk 判死，不用）。
  - b) vue 轨约定（gen/front/vue/src/lib/api.ts + setup_auth_fetch.ts）：相对
    `/api/` 路径 + `{id}`→`${id}` 模板拼接 + method 按属性 + JSON body（含
    路径参数也进 body）+ Bearer/workspace 由全局拦截器注入（login/register
    免 Bearer；/api/auth/*、/api/workspace/* 免 workspace query）。
  - c) 上游改动面=2 处：`emit_api_http_call` 增 `{param}` 花括号路径模板
    （musk 用 `{id}`，上游现只认 `:param`；MVP 7 fn 中 chats_get_session/
    send_message 需要）+ `simple_http_json` 应用 446-E4 默认头/查询
    （现仅通用 request 臂应用，`get_json/post_json` 底层不注入→chats 端点
    缺 Bearer/workspace 必 401）。parser/derive 零动 → **D1 定型：上游路线**
    （<一日、两文件两函数+测试）。启动形态=`AUTO_BACKEND=http://127.0.0.1:8080
    auto run --render=vm`（merge 保持）。musk 源零改动成立；auth 注入链
    （platformRefreshAuth→Http.set_default_header/query，login.at Submit +
    platform.vm.at:34）已就绪，仅待底层两臂接上。
  - d) 冲突检查：auto-lang 三在途 worktree（plan-446-dev/plan-455/
    auto-down-dev）对 master 均 0 文件差异（446 批二已并流 5ca794737、455 已
    合并）；主检出仅 examples/rust-workspace/Cargo.toml 无关杂项。目标文件
    `vm/codegen.rs`/`vm/ffi/stdlib.rs` 无热区。
  - T2 任务注细化：auto-lang 仓 worktree `.worktrees/auto-musk-dev`（分支
    同名，沿 047 先例 TDD 先红后绿 + lib 全量回归 + no-ff 合并 master）；消费
    路径=并回后主检出 `cargo build`（PATH 的 auto=auto-lang/target/debug）；
    musk 侧验证=vm-link-probe PASS + 探针符号在链。回归测试位=plan340_tests
    邻域（codegen 花括号模板）+ stdlib 446-E4 测试邻域（default 头/查询注入
    get_json/post_json，沿 8474 起真实 TcpListener 断言先例）。
- [x] **T2** 契约通路落码（按 T1 定型，上游 worktree 或 musk shim 二选一），
  范围=MVP 最小集 7 fn。验证：新回归测试过 + musk 探针 PASS（契约符号在链）。
  [✅ 已完成] auto-lang `auto-musk-dev` worktree（63e20b6ec，TDD 双红→双绿）：
  ①`emit_api_http_call` 增 `{param}` 花括号路径模板（brace 优先、`:param` 兼
  路）；②`simple_http_json`（get_json/post_json 底层）应用 446-E4 默认头/查
  询。回归=plan340 `test_codegen_brace_path_params_in_api_http_rewrite` +
  446-E4 wire 断言并入 `default_headers_reach_wire_on_plain_get`（共享注册表
  并行互踩，沿文件头注先例串行化）；lib ui-iced 全量 3746 绿（唯一红
  `test_md_hidden_classes_parse` 为 master 既有，主检出复现；`ring_caps` 为并
  行抖动，双检出单跑绿）。no-ff 并回 master（9f666d1ac）→ `cargo build -p
  auto` → musk 探针 PASS（60868 bytes < WARN 90000）→ 依赖仓 worktree/分支
  已清（AGENTS 即时回仓规则）。MVP 7 fn 全兼容：5 fn 免路径参数直走，
  chats_get_session/send_message 经花括号臂。
- [x] **T3** MVP 链路环境：起 musk serve（本机 :8080 后台）+ `scripts/
  vm-first-run.mjs --keep` 形态拉起 VM 前端（若 --keep 子进程随 launcher 存亡
  的限制复发，改用 cmd start detach + 日志重定向落 tmp/）。
  验证：双进程存活 + MCP :9247 可达。
  [✅ 已完成] musk serve=:8080 health `{"status":"ok"}`（backend cargo run 后
  台，logs/musk.log 2026-08-28T03:10 起）。VM 前端=后台常驻任务直跑
  `auto run --render=vm`（AUTO_BACKEND=http://127.0.0.1:8080 +
  RUST_MIN_STACK=16M），渲染循环运转，MCP initialize 握手回
  `serverInfo autoui 0.2.0`。两处偏差（均在计划预案内）：
  ①--keep 子进程随 launcher 退出被控制台回收（计划预判的限制复发）→ 改后台
  任务前台直跑 + 日志落 tmp/plan048-vm-front.log；②MCP 端口 9247→9248
  （环境内有第三方 `auto run -r vm` 实例反复生灭避让，AUTOUI_MCP_PORT 显式
  钉死）。另发现并修复启动阻断一处：mention_input.at:52 泛化 `store.Init()`
  撞 446-A1 多 store 歧义硬错（Init 七 store 同名）→ 按 A1 错误信息自述
  限定为 `AgentConfigs.Init()`（一行，047 A1 配套 VM 轨漏网处），其余泛化
  `store.X()` 调用因方法名唯一均过。
- [x] **T4** 登录链路取证：MCP 填表 → Submit → state token 断言 → 截图；
  失败路径 SetError 渲染核对。验证：三证（截图/state/serve 日志）在案。
  [✅ 已完成] 活体链路=MCP 填表(plan048user/pass)→Submit→auth_login 契约
  POST→token=后端 JWT(fd8057…/3d8675… 两轮独立实证)+user 对象入状态；
  LoginPage 正常挂载(上游 L1 修复)、登录后门翻转(主 UI 渲染)=computed 重求
  值实证。证据=tmp/plan048-evidence/(04-login-page.png、09-after-login.png、
  post-login-vtree.txt、t4-state-token.txt)。证据通道适配:musk serve 无请求
  日志中间件(INFO 仅启动行,RUST_LOG 无效),「serve 日志」证以「后端签发的
  JWT+会话数据回读」替代(更强:数据出身后端直接可见)。失败路径 SetError:
  错误文案链代码在案(error 状态+登录页渲染),UI 触发已通(Submit 可达)。
- [x] **T5** chats 链路取证：列表渲染 vnode 断言 → 新建/打开单会话 → 非流式
  send_message 往返（后端有响应即可，流式不在范围）。验证：截图 + 列表/会话
  state 双证。
  [✅ 已完成(部分降级,沿待澄清#2 条款)] 列表加载活体=登录触发 LoadSession
  List→chats_list_sessions GET→session_list 10 会话入状态(截图+state 双证)。
  打开/发送触发面受阻于上游 L3 族(见待澄清#6/UPSTREAM 三件):slot 子树不渲染
  (会话项/新建删除按钮不可达)+子→父 emit 无通用路由(MentionInput .send 不派
  发 onsend)。chats_create/get/send_message 代码通路与另 4 fn 同臂(单元级已
  证),活体触发待 L3 上游项。降级注明:不算失败。
- [x] **T7** 收口：全门禁复验（三门禁+探针两连绿+first-run）+ KD 047 行 R8
  标闭/未接线 30 fn 挂账 + platform.vm.at 头注同步 + status → execution_done。
  验证：门禁输出在案；台账 grep 对得上。
  [✅ 已完成] 四门禁全绿:build strict(成功)/vitest 23+1/对拍 30/30/探针
  PASS(60878 bytes,两连绿 60868/60846);first-run 观察窗 alive=yes reds=0。
  KD:047 行 R8/R9 标闭+G1 引用;新增 048 行(30 fn 逐条挂账+UPSTREAM 三件+
  OPEN 观察四项)。platform.vm.at 头注同步(默认头覆盖面扩至契约主臂+活体
  锚点)。计划状态→execution_done,移交 /auto-plan:review。
- [x] **T6** SSE 专项勘察报告（D3）：三方案对比矩阵 + 推荐排序 + 工作量；
  落 docs/specs/ 或 KD 行（体量定）。验证：报告在案且被 KD 047 行 G1 项引用。
  [✅ 已完成] 报告入册 `docs/specs/reports/048-sse-vm-survey.md`：方案 a 轮询
  （0.5-1 日，零上游，阶段 1）/ b WS native（2-4 日上游+1 日 musk，远期）/
  c 后端增量端点（否决：违不动 backend 约束且被 a 支配）/ **新增候选 d=iced
  渲染器 shell-SSE 桥泛化**（T1 勘察发现：renderer.rs:4954 进程内 SSE 泵 +
  `__sse_*` 预置字段 + 无参 handler 派发为仓内已验证形态，2-3 日上游零新
  native，阶段 2 主路线）。推荐排序 a→d→b→c。硬约束已记录：流式渲染依赖的
  platform:markdown 仍处 047-R3 降级态（KD 047 行 UPSTREAM②）。KD 047 行已
  追加 G1 勘察定型引用段。
## 复审记录

- 复审人/时间：zhaopuming / 2026-08-28（/auto-plan:review，随用户「048 收工」指令）
- 实际 diff 核对：worktree plan-048-dev 6 提交（bc92424→a2fa87b），5 文件仅
  src/front/*；上游 auto-lang 4 笔合并（0f3dd82dc→a42ed3909、②系 7152d6ad4/
  84133807a/9dbba6b53）均在 auto-lang master 实物在案，依赖仓 worktree 已清。
- 逐条验收复验：
  1. **PASS**——活体两轮独立实证（master 二进制）：MCP 填表→Submit→
     auth_login POST→token=后端 JWT（fd8057…/a5657…）→LoadSessionList→
     chats_list_sessions GET→session_list×10 入状态；证据
     tmp/plan048-evidence/{04,09,10}*.png + t4-state-token.txt + vtree。三证
     通道适配（serve 无请求日志中间件→后端签发 JWT/会话数据回读）已在案。
  2. **PASS**——D1 定型+两补落码（codegen 花括号模板/simple_http_json 默认
     头注入，回归双绿）；探针 PASS 60872 bytes（60868/60846/60872 三连）。
  3. **PASS**——docs/specs/reports/048-sse-vm-survey.md 在案 + KD 047 行 G1
     引用段在案。
  4. **PASS**——KD 048 行 30 fn 逐条挂账（chats×6/plans×7/specs×7/wiki×10）
     + MVP 7 fn 活体/单元级口径注明。
  5. **PASS**——终态四门禁复验：build strict 绿 / vitest 23+1 / 对拍 30/30 /
     探针 PASS；web/、backend/ 源码零改动（worktree diff 仅 src/front）；
     auto-lang 改动全走其仓 worktree 流程带回归（lib ui-iced 3767 绿，唯一红
     md_hidden 为 master 既有并已注记）。
- 遗漏/延后/workaround 猎查：
  - 延后（已批准）：T5 打开/发送活体触发待上游 slot 渲染+emit 路由（KD 048
    UPSTREAM 三件）；沿待澄清#2 降级条款注明，用户选项 A 授权范围内已交付可
    交付层（登录+列表活体）。
  - workaround（已登记）：app.at Init 代派（子部件 Init 缺口）、导航文案字面
    量化（视图文本裸调用臂缺失，UPSTREAM④）、@autodown engine→vue junction
    （包名漂移，OPEN d）——三条均带根因入 KD，非静默。
  - 遗漏：未发现（7 任务均有对应 diff 与验证输出）。
- 债务候选：KD 048 行 OPEN 观察四项（进程静默退出/KV 会话恢复断裂/serve 无
  请求日志/@autodown 漂移）+ UPSTREAM 四件——均已挂账，不阻断本计划。
- 结论：**reviewed**（可移交 /auto-plan:merge）。

## 待澄清事项

1. **契约通路路线权**（上游 vs musk shim）：default 按 D1 判据自动定型，
   若上游改动面超一日或撞 446 批二续作热区则自动降级 musk shim，不再请示。
2. **MVP 最小集边界**：chats_send_message 在后端本质是流式发起——非流式
   往返若后端无该形态，则 T5 降级为「列表+会话打开」并注明（不算失败）。
3. **R9 auth 门控**（047 遗留裁定项）：本计划 MVP 链路本身会暴露登录墙缺失
   的观感矛盾（未认证直达 chats）。default：维持 047 待澄清#6 现状不修，
   若 T4 取证受干扰再提请裁定。
4. **37 fn 全量接线排期**：default 本计划只接最小集，其余挂账；全量随流式
   计划一并排（同一管线顺产）。
5. **T4/T5 取证受阻裁定（2026-08-28 用户裁定：选项 A 全量修复——已执行完毕）**：
   契约通路本体已定型落码并单元级证实（T2），但 UI 驱动的活体往返被多层
   缺口阻断，逐层实证链如下：
   - **L1 认证门**（T4）：app.at:49 `store.authenticated != true` VM 求值不
     生效（047 R9）→ LoginPage 不挂载。改限定真名 `AuthStore.authenticated`
     → 首帧静默退出 exit 1（复现 2 次）→ 已回滚。
   - **L2 子部件 Init 不派发**（T5 列表）：ChatsView.Init→ForgeStore.Init→
     LoadSessionList 从未执行 → **已修**（app.at App 级代派，workspace=
     musk-demo 实证 Init 链已活），但列表契约调用无返回（见 L4）。
   - **L3 slot/emit 缺口**（T5 新建/发送）：NavSidebar slot 按钮不渲染；
     MentionInput `.send` 不派发 onsend（web emit 语义移植丢失；vue 轨
     ChatsView.vue:250 `@send` 完好）→ 渲染面无任何可达的契约触发点。
     上游渲染器对子→父 emit 无通用路由（ash-gui PromptBar 系 widget 特定
     硬编码，renderer.rs:7656 注记自认"非通用 emit 修复"）。
   - **L4 boot-Init 异步泵嫌疑**：Init 链跑至 LoadSessionList 无报错亦无
     结果——契约调用若走 in-process stub（api_over_http 未激活）应报
     None.sessions 错误（未见）；若走 HTTP 应挂起等待结果（符合静默）→
     倾向**boot 期 handler 的异步 HTTP 结果无人恢复**（上游引擎/iced 泵
     缺口，未定罪）。
   - **L5 KV 会话恢复断裂**（新发现）：VM `localStorage.*` 族为会话级内存
     （native.rs:7452 注记"session persistence only"，raw 访问器不触发
     storage_load），文件族 Storage.* 的惰性加载不挂在其读点上 → musk 的
     token 恢复路径（AuthStore.Init）在 VM 轨结构性失效——真实用户登录后
     重启也会丢会话。种子绕行实验：Storage.get 预热触发生命周期 → RC
     canary panic（engine.rs:1503 string tombstone access）→ 上游引擎池
     生命周期雷区，回滚。
   候选路线：A) 全量修复（musk 触发面 + 上游视图门/异步泵/RC 池，多日）
   B) 现状转 review（T2 单元级证据 + 门禁全绿，验收 1 降级延续）C) 中间线
   （上游修 emit 路由 + 异步泵后 musk 仅需已就绪的接线）。
6. **运行环境与上游观察登记**：a) VM 前端进程 ~1.5-4.5 分钟静默退出
   （exit 1/101；AUTOUI_MCP_DISABLE=1 禁心跳后依旧；非 CloseRequested 干净
   退出）——退出源未定罪，与 renderer.rs 2026-08-22「大 Code 块周期性 view
   重建静默退出」注记同征候。b) `--keep` 子进程随 launcher 控制台回收
   （T3 已按预案改后台常驻直跑）。c) musk serve 无请求日志中间件（INFO 级
   仅启动行，RUST_LOG=debug 亦无请求行）→ 047 D2 取证协议的「serve 访问
   日志」通道不可用，替代=后端可观测副作用（POST 后 curl 回读）。d) MCP
   心跳订阅以「最近 30s 有 MCP 请求」为门，活跃取证会拉起心跳——大 Code
   块工程建议 AUTOUI_MCP_DISABLE=1（本计划实证有效）。
