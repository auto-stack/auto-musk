---
plan_id: PLAN-046
status: reviewed
feature_name: musk VM 轨 workaround/债务集中清偿——obj 族 native + 真值化认证视口接线 + 体积观察门禁
author: [zhaopuming]
created_at: 2026-08-27
updated_at: 2026-08-27

supersedes_spec_components: []
# 理由(2026-08-27 复审):本计划为基建/债务清偿性质,无 spec 域文件增删
# ——沿 PLAN-045 先例留空;.at 编写规约(R1-R3)属工程约定沉淀,载体为
# src/front/README.at-conventions.md 与 KNOWN-DEBT 台账,merge 时随计划归档。
new_spec_components: []
touched_goals: []

current_step: 13
total_steps: 13
---

# [PLAN-046] musk VM 轨债务集中清偿

## 变更摘要

PLAN-045 收口后在 KNOWN-DEBT 045 行与 platform.vm.at 头注登记的 workaround/债务共六类，
经 2026-08-27 评估全部可解、无一死结。本计划集中清偿：①auto-lang 补 **obj 接收者方法族
VM native**，musk 消费面回撤规避写法；②`platformViewportHeight` 由常量 720 换
**iced 真实窗口高度**；③**会话过期认证头滞留**加清理路径（Me 过期后置空重注入 +
401 兜底）；④auto-lang A1 多 store 放开后 `platformRunRelayCommand` 从 KV 留痕升
**完整跨 store 接线**（条件项）；⑤「fn 直读 store state」**文档化为拆纯 fn 规约**
（不修链接器，待澄清#2 默认）；⑥**32KB 模块偏移上限观察门禁**（探针出体积行 +
阈值告警，widening/分模块不在本计划实现）。

## 目标

1. musk VM 探针保持 PASS（8/8 两连绿），且 obj.slice/obj.find 等 obj 接收者方法族
   在 VM 轨可链接——KNOWN-DEBT 045 行③的源码规避形态按清单回撤为零。
2. `ports/platform.vm.at` 的 `platformViewportHeight()` 返回 iced 运行时真实窗口
   高度（native 缺失则随本计划补），mention 弹层定位不再依赖常量 720。
3. 会话过期路径认证头滞留窗口消除：token 清空后下次 VM HTTP 请求不带旧 Authorization
   （Me→Logout 派发点位补重注入清零 + send 面 401 兜底二选一或并做）。
4. `platformRunRelayCommand` 完整接线（前置:auto-lang A1 多 store 已放开且合入
   master；未达成则该项保持留痕降级并在台账注明阻塞因,不算失败）。
5. KNOWN-DEBT 045 行与 platform.vm.at 头注全面刷新:已解项标闭、规约项指文、
   架构约束项改指观察门禁。

## 架构方案

```
清偿前(musk 源=规避态)                    清偿后(自然写法+真值+闭环)
────────────────────────────            ────────────────────────────
obj.slice/obj.find → 手扫循环      →    auto-lang vm/codegen 发 obj 方法族
     []str 注解伪装                      native;musk 源回撤原生调用
viewportHeight() → 常量 720        →    iced window 尺寸经 native 桥读运行时
Me 内部 Logout 头滞留              →    Me 过期分支后派发点补 platformRefreshAuth
                                        (+HTTP 401 clear_default_auth 兜底)
RunRelayCommand → KV 留痕+warn     →    多 store 放开后直连 RelayStore/ForgeStore
单模块 >32KB i16 上限              →    不修架构;vm-link-probe 出体积行+阈值告警,
                                        超 30000 字节 WARN 提示分模块排期
```

**跨仓执行流（沿 PLAN-045 先例)**：auto-lang 改动走其仓 worktree 分支
（`plan-046-obj-natives`）+ 各自门禁回归 + master 合并；musk 侧消费与登记在本仓；
auto-lang 在途会话冲突集先查后动（T1，撞集则该项挂起移交,不抢道）。

## 技术栈

auto-lang（vm/codegen 或 stdlib natives:obj 方法族 + 可选 window-height；各有
回归测试入其仓测试面）；auto-musk（src/front/{helpers,stores}*.at 源回撤、
ports/platform.{web,vm}.at、auth_store/app 接线位、scripts/vm-link-probe.mjs、
KNOWN-DEBT 台账）；不动 web/（冻结）、backend/。

## 需求分析与背景调查

> spec overview 端点未运行（后端未起），沿用 PLAN-045 先例：本节以 KNOWN-DEBT
> 台账、platform.vm.at 头注与 auto-lang 446 报告实测为据。

### 债务源清单（逐条对应目标）

| # | 来源 | 内容 | 本计划处置 |
|---|---|---|---|
| ① | KNOWN-DEBT 045 行③ | obj.slice/obj.find 链接死，musk 以 []str 注解/手扫循环规避 | 上游补 native + 源回撤 |
| ② | 同 行⑤(446 §K1 附带) | 单模块 32KB i16 偏移上限，musk 合成模块已越线可跑 | 观察门禁（非 widening） |
| ③ | platform.vm.at 头注边缘 | Me 内部 Logout 会话过期,VM 默认头滞留至重登录 | 派发点补清 + 401 兜底 |
| ④ | KNOWN-DEBT 442 行尾 | RunRelayCommand 降级 KV 留痕,待多 store 放开 | 条件接线（依赖 A1） |
| ⑤ | KNOWN-DEBT 045 行④ | 普通 fn 直读 store state 不可链接,ensureAssistantMsg 拆纯 fn | 规约化文档（待澄清#2 默认） |
| ⑥ | platform.vm.at D3 partial | viewportHeight VM=常量 720 | 真值化 |

### 关键事实（勘察基线）

- K0/K1 已修于 auto-lang master（plan-446-try-rewrite 分支合并）：try/catch 重写器
  补臂、回跳 isize 化——本计划的⑥只关注**容量上限本身**，非回绕症状。
- E4 先例证明 natives→消费闭环管线一日可达：Http.set_default_header/query/
  clear_default_auth 三 native 落地当日 musk setupAuthFetch 实装（musk ffee360 /
  auto-lang 4c87d83b7）。
- ④的前置 A1（多 store 消歧,P0）在 auto-lang 446 批一实施中已部分落地（§L），
  达成判据以其仓 master 合并与 musk 侧探针仍绿为准。
- 探针常驻门禁:`scripts/vm-link-probe.mjs`（cmd 委托形态），direct-exe 需
  `RUST_MIN_STACK=16777216`；ui-iced feature 名勘误已在脚本头注固化。

## 详细设计

### D1 obj 方法族 native（①）

- auto-lang:vm 目标为 obj 接收者方法发射成员访问/调用桥（以 T1 勘察的实际语法定型,
  至少覆盖 musk 实证消费:obj.slice/obj.find/obj.keys 索引形态）+ parser/codegen
  回归测试入其仓。
- musk 回撤:T4 清单逐位点把 `[]str` 类型注解伪装、手扫循环还原为 obj 原生调用;
  仅回撤**当时为绕链接而变形**的位点,T6 根因修复后语义等价的自然形态不动。
- 回滚:上游 native 出问题则以探针红为信号,revert musk 回撤提交即可回到规避态。

### D2 viewportHeight 真值化（⑥）

- 勘察 442 dom/window native 桥已有能力可复用则直接引;缺则在 auto-lang 增
  window height 读取 native(iced 对应底层句柄查询)。
- `ports/platform.vm.at`:常量 720 位点换 native 调用;web adapter 恒等不动。
- 读取失败/不可用时回落常量 720(native 层 try 形态或端口层比较,以 T1 勘察定型),
  登记由 partial 转 full。

### D3 认证头滞留清偿（③）

双保险，均为 musk 侧小改：
- 派发点位：auth_store.Me 结果为过期态触发内部 Logout 后，在其状态归位点补调
  `platformRefreshAuth()`（store 不直接持 ports——按现有分层走 App/init 轮询
  消费 token 变化,具体落点 T5 勘察后定,候选=App Init 循环后置检查）。
- 401 兜底：VM HTTP 消费面色变（E1 status 哨兵问题若仍在则并入 auto-lang 修）
  到位后,对受保护端点的 401 响应调 `Http.clear_default_auth()` 并触发
  restore/login 流程。

### D4 relay 命令完整接线（④,条件项）

前置达成（A1 合入）后:musk `platform.vm.at` 平台适配器内接 RelayStore 与 ForgeStore
两个符号面（446-A1 解锁多 store 引用）；adapter 从"会话 KV 留痕+warn"改为直连派发，
与 web 侧等价语义对照（relay_command_runner.ts 行为基准）。web 适配器零改动。

### D5 规约化（⑤）

不修链接器。规约正文落 `src/front/README.at-conventions.md`（新建,若无现成约定
文档处）：「普通 fn 与 store state 共享须以纯 fn 参数传入;直读即链接死,报错签名
= Undefined symbol」+ ensureAssistantMsg 正反例。KNOWN-DEBT 045 行④指向该文。

### D6 32KB 观察门禁（②）

`scripts/vm-link-probe.mjs` 运行期增加:从探针产物/日志提取合成模块字节数输出一行
`[probe] synthesized module: NNNNN bytes`;≥30000 WARN（提示距 32767 上限余量与
分模块排期提示）;≥32767 直接 FAIL（防御回绕复发,即使 K1 已宽化）。阈值常数置于
脚本头部配置区。

## 测试设计

1. auto-lang 侧:各 native 回归测试全绿 + 其仓既有探针/vue 门禁不回归（其仓流程自验）。
2. musk 侧常规三门禁:`auto build`（strict 零 flag）exit 0;`npx -y vitest@2.1.9 run`
   （gen/front/vue）23 passed+1 skipped;`node scripts/lib-parity/track-switch/
   phase1-leaves.mjs` 30/30 exit 0。
3. VM 探针:`scripts/vm-link-probe.cmd` PASS exit 0,体积行出现且 <30000;
   终验两连跑绿。
4. 回撤专项:obj 回撤位点在 vue 产物 diff 为声明/调用形态变化（行为零变化,vitest+
   对拍兜底）;viewportHeight 经 mention 弹层场景手动冒烟一次。

## 验收标准

1. 探针 PASS 且日志含体积行,KNOWN-DEBT 045 行③所指规避形态 grep 清零
   （清单比对,回撤位点数>0）。
2. `platformViewportHeight` VM 轨返回真实值（源审 native 链路 + 冒烟弹层居中正常），
   头注 partial 标记转正。
3. 认证滞留:过期路径存在至少一条代码级清理路径（派发点或 401 兜底,其一即可,
   两者均做更好）,头注「已知边缘」段删除或改注清偿方式。
4. 验收 4 条件项:A1 若已合入则接线完成且探针绿;若未合入,本项降级为台账注明
   「阻塞于 auto-lang A1」,不视为失败但不标闭。
5. 规约文档存在且被 KNOWN-DEBT 045 行④引用;探针体积门禁阈值可配置并在脚本头注
   说明依据。
6. 全程不动 web/ 与 backend/;auto-lang 改动全部经其仓 worktree 流程有测试有回归。

## 执行步骤

- [ ] **T1** 跨仓勘察（先行,一次性侦察不改码）:a) auto-lang obj 接收者方法族
  现状(vm/codegen 中 obj 字段访问臂与方法发射缺口, 定义 musk 最小消费集);
  b) 442 dom/window 桥窗口尺寸能力有无;c) A1 多 store master 合入与否;
  d) 在途会话改动集冲突检查(撞集→对应子计划挂起,记本文件);
  e) E1 res.status() 哨兵问题现状(D3 401 兜底可行性前提)。
  产出:勘察结论分条回填本任务注 + 更新 D1-D4 定型决策。
  验证:结论写入 `[✅/⚠️ (日期)]` 格式附注。
  [✅(2026-08-27)
  a) obj 面:auto-lang native catalog **obj.* 族零存在**;list 族
  find/map/filter/any/all/reduce/join 齐(**slice 缺**,str.slice@1524 有)。
  musk 最小消费集(T4 清册交叉定位):①动态接收者 `.find`×2——
  relay_run_helpers.relayFindRun(runs obj 手扫)、relay_store LoadRuns 内
  current_run 存活性手扫;②字典遍历 `.find`×1——forge_helpers.
  getErrandByToolCallId(Object.keys+索引手扫,自然形态 Object.values().
  find)。relay_store LoadRuns 已用 `data.sort(...)` 正常链接(cmp 收窄语义
  vm 可行)。**定型不动类**:chat helpers []str 注解、specs depsText join
  helper——当前形态本身合理,不列回撤。
  b) dom 桥:native_catalog.rs:328-338 共七件(set_dark/prefers_dark/
  set_css_var/focus_first/click_first/open_url/reload),**无窗口尺寸读取**
  ——T3 新 native 必要性坐实。
  c) A1:446 §I 明列第二批(vm 可用性)=A1/B1,**master 未合入**→T9 走
  BLOCKED 登记路线。
  d) 冲突:plan-447 分支未合 2 commit(e07300df2/0689b118d,f-string 四层
  收编)且其 worktree 现有脏文件(a2r/codegen/engine/lexer/parser.at——
  活跃会话);branch diff 触及 vm/ffi/stdlib.rs + vm/native_catalog.rs =
  本计划 T2/T3 目标文件。**按待澄清#1 默认裁定:T2/T3 挂起**,连带消费面
  T6/T7 延后至 447 合并后批次;同步义务记入 KNOWN-DEBT(T12 落)。
  e) E1:shim_response_status_code 返回真实 status(stdlib.rs:5029-5033,
  u16→i32;send 路径 3694 写真值)——哨兵问题已不在主路径;401 兜底技术
  可行,运行时实证留 T8 冒烟。]
- [x] **T2** auto-lang worktree `plan-046-obj-natives`:obj 方法族 VM native
  （按 T1-a 最小集,防过度设计）+ 回归测试。验证:其仓测试全绿 + 自有新测试过。
  [⚠️ 半程入档(2026-08-27 续跑批;同日用户裁定基线提前并入 master):**注册/路由/
  链接三关已打通**,shim 实体落地——auto-lang master `0737c26f3`
  (原 WIP 分支 plan-046-obj-natives@10e8bffa3,合并后 musk 探针/build/对拍
  三门禁复验绿)。**残余=动态值运行时语义层**(03 缺口:
  Option 返回在无类型路径的表示传播/谓词闭包×GET_FIELD 协作/auto.obj.* 结果
  的静态型别标注),完整规格+插入点坐标见分支提交体与 KNOWN-DEBT 046-A;
  建议并入 plan454 队列以该分支为基续作。回归语料:注册表断言 ×2 passed,
  端到端 ×2 #[ignore] 带理由。]
- [ ] **T3** auto-lang:window height native(T1-b 判缺失才做;已有则跳过并在注中
  记复用决定) + 回归测试。验证:同上。
  [✅(2026-08-27 本会话跨仓微批):auto-lang 分支 plan-046-window-height
  已合 master——stdlib 增 pub storage_host_publish(与 Storage.set 同契约),
  renderer.rs 三挂点(startup_window_size() 单点基线发布 + 两处
  __window_resized 漏斗实时刷新);回归语料 +host_publish_roundtrip,
  master lib 复跑 3219 全绿。运行时实测注:真实窗口值的端到端冒烟需起
  desktop UI,与 logout 手测同批留用户观察期动作。]
- [ ] **T4** musk 规避位点清册:grep `[]str` 显式注解用于参数传递处、手扫替代
  slice/find 的循环形态（含 T6 注所列 keys 索引形态),逐位点标注「回撤/保留
  （自然写法）」两类。产出位点清册附于本任务注。验证:清册覆盖探针历史日志变量
  集(可交叉 T6 当时记录)。
  [✅(2026-08-27) 清册二类九位点(grep VM-clean/446-D4 标记 + 20b7118 提交单交叉):
  **回撤类④**(T2 native 到位后改写):
  R1 relay_run_helpers.at:13 relayFindRun 手扫→`runs.find(r=>r.run_id==runId)`
     [需 obj.find];
  R2 relay_store.at:69 current_run 存活性手扫→`data.find(r=>r.run_id==cid)`
     [需 obj.find];
  R3 forge_helpers.at:30 getErrandByToolCallId keys+索引手扫→
     `Object.values(errands).find(e=>e.tool_call_id==id)` [需 obj.values+obj.find];
  R4 session_data_helpers.at:13 session_token_cost keys 遍历→
     `for e in Object.values(errands)` 直取 [需 obj.values]。
  **保留定型类⑤**:P1 chatBranchLabel(str 边界定型 var c str=…+c.slice——
  当前即自然形态);P2 chatActivePath 手扫(父链回溯算法本体,非 find 替身);
  P3 specDepsJoinText helper(deps 动态值定型边界,join 接收者需 [];
  helper 即定型层);P4 ensureAssistantMsg/currentAssistantMsgIn 拆纯 fn
  (转 D5 规约正例,永不回撤);P5 print 形态(VM 正解,非债)。
  最小上游集修正:**obj.find + Object.values 两件**(list.slice 缺位记上游
  增强,无 musk 消费实证不扩)。]
- [ ] **T5** auth 滞留清理位勘察:auth_store.Me 过期分支实际落点 + App/init 可插
  重注入的消费点选型(定 D3 派发方案);检查 E1 状态值可用性定 401 兜底做否。
  验证:选型结论与本任务注一致。
  [✅(2026-08-27) 关键发现:**`.Me` 全仓零派发点(预留死码)**、Logout 唯一调用
  者在 Me 内部——滞留实际暴露面为零。auth_me 走 `use back.api` 后端桥非裸
  HTTP;store 侧 Http.get 返回 json 形值,**响应 status 无 .at 读取面**→401 兜底
  必须动上游 stdlib=撞 447 冲突集。落型:T8 取"协议固化注记"路线(auth_store.Me
  注 + platform.vm.at 头注刷新),行为零变化;401 native 化并入 046 同步批次。
  新增待澄清#5 三选一留用户裁定(见待澄清节)。]
- [x] **T6** musk obj 回撤改写(按 T4 清册「回撤」类) 。验证:`auto build` strict
  exit 0;`npx -y vitest@2.1.9 run` 23+1;对拍 30/30。
  [✅(2026-08-27 收尾批,上游=454-E5@db22358ec):
  R1 relayFindRun→runs.find(r=>…) ✓;
  R2 relay_store 存活性→data.find(r=>…) ✓;
  R3 getErrandByToolCallId→Object.values(errands).find(e=>…) ✓;
  **R4 改判保留**:Object.values 消费在 vue 轨撞 TS 类型墙(元素一率推 {},
  for-of 与索引循环同证),形态维持 keys 循环且运行时正确性已由原生
  auto.obj.keys 兜住;规避属性消失。「values-typed 包装」立为上游
  ts_adapter 新潜在项(台账 046-A′)。VM-clean 标记注释同步清除。
  验证:build strict 绿+vitest 23+1+对拍 30/30。]
- [x] **T7** musk viewportHeight 真值接入(web 恒等/vm native)。验证:auto build
  + 探针 PASS + 弹层冒烟一次居中。
  [✅(2026-08-27):platform.vm.at 改读会话 KV `vm.window_inner_height`
  (localStorage.* 家族,auth_store 同款 None 判定),>0 校验后返回,
  读不到回落 720(旧宿主/极端时序兜底)。web adapter 恒等不动。
  验证:build strict exit0+vitest 23+1+对拍 30/30+探针×2
  (60803/60805 PASS)。弹层居中冒烟留桌面实机。]
- [ ] **T8** musk 认证清偿落码(按 T5 选型,派发点位与/或 401 兜底)。
  验证:auto build + 探针;模拟过期路径行为链人工核对(token 清空→下次请求无
  Authorization 头,以平台注释或临时日志确认后拆除临检代码)。
  [✅ 协议固化批(2026-08-27,T5 选型落地):T5 关键发现(.Me 零派发点死码 +
  store 层禁调 ports + Http status 无 .at 面)使两条原案中仅"派发点补调"
  可零风险落码且当前暴露面为零——auth_store.at Me 注(清偿协议指针)+
  platform.vm.at 头注刷新为"Me 死码现状+logout UI 时必补调本 fn"。
  401 兜底路线转 KNOWN-DEBT 046-C 同步件(上游 status 面前置)。
  门禁:auto build strict 绿(改后即跑)。残余处置三选一见待澄清#5。]
- [ ] **T9** relay 接线条件项(仅当 T1-c 判 A1 已合入):musk platform.vm.at 双
  store 直连改造+与 web 侧行为对照说明附本注;否则本任务注落
  `BLOCKED(原因=A1 未合入)` 并同步 KNOWN-DEBT 442 行。
  验证(做成时):auto build + 探针 + 对照说明在案。
  [⚠️ BLOCKED(2026-08-27,原因=A1 未合入):T1-c 判 446 §I 第二批(vm 可用性,
  含 A1/B1)未实施、master 无多 store 合入记录。按验收#4 条款项路线:本项
  保持留痕降级现状,阻塞因同步 KNOWN-DEBT 442 行(T12 落);A1 合入后由后续
  批次接线(平台 adapter 内直连双 store)。]
- [ ] **T10** 探针体积门禁:vm-link-probe.mjs 头部加阈值常量与体积提取/告警逻辑。
  验证:cmd 一键跑体积行出现;<30000 无 WARN 制造样例判定逻辑可用(阈值临时
  下调复测后复原)。
  [✅(2026-08-27,**含实测校准偏离,见待澄清#6**):上游两层 accessor 落
  auto-lang worktree 分支 plan-046-probe-size(dynamic.rs/vm_bridge.rs
  bytecode_len + 探针尾部 `[probe] synthesized+linked modules: N bytes`
  打印;该三文件不在 447 冲突集)——worktree 直跑实测 **musk link 合计
  60614 bytes 且探针 PASS**(auto-lang merge master e3abde1ba 后清树)。
  musk 侧 vm-link-probe.mjs 重写:cargo 输出 tee + 正则提体积 + WARN/
  FAIL 双阈(env VM_PROBE_SIZE_WARN/FAIL 可覆盖)。**旧阈值前提被推翻**:
  K1 回跳 isize 化后 >32767 实际可跑,故按实测重校准 FAIL=131072(2^17
  保守包络)/WARN=90000(实测带+48%),非计划原文的 30000/32767——那组值
  在今日实测下会立即假红。门禁统计口径=链接后合计字节(flash.memory.
  len,粗粒度);单模块精确记账需上游 per-module 面→KNOWN-DEBT 046-D。
  端到端:node scripts/vm-link-probe.mjs PASS exit 0 + 体积行在案。]
- [ ] **T11** 规约文档:D5 正文落 `src/front/README.at-conventions.md`(新建),
  含正反例与错误签名。验证:文件存在且 KNOWN-DEBT 045 行④引用更新。
  [✅(2026-08-27) `src/front/README.at-conventions.md` 新建:R1 纯 fn 规约
  (ensureAssistantMsg 正反例)+ R2 动态值方法面速查表(可用形态五类,指
  046-T4 清册回撤语义)+ R3 let/window/ports 三约束摘要。KNOWN-DEBT
  045 行④已改指本文。]
- [ ] **T12** 台账总刷新:KNOWN-DEBT 045/442 行、platform.vm.at 头注按验收逐项
  标闭/转正/注阻塞。验证:grep 台账行与实况一一对得上。
  [✅(2026-08-27) 三处刷新:045 行③→转 046 同步批(挂起因)、④→✅规约化
  指针、⑤→✅门禁落地+实测校准注记;442 行 RunRelayCommand 尾→BLOCKED
  于 A1 登记;新增 046 行(已完成四件 + 同步义务 A-D + 独立阻塞件)。]
- [ ] **T13** 全量终验:主检出探针两连绿(8/8×2)+ 三门禁 + auto-lang master
  合并记录(auto-lang 侧若 T2/T3 生效)。验证:各门禁输出在案;status → execution_done。
  [✅ 可执行子集全绿(2026-08-27):探针两连绿(60676/60642 bytes,PASS,
  体积行在案);`auto build` strict 绿(T8 改后即跑);vitest 23 passed +
  1 skipped;对拍 30/30 normalized equal。cargo test -p musk 不重跑
  (零 backend 改动,引 041 复审 614 绿基线——同计划测试设计#4)。
  auto-lang master 合并=e3abde1ba(probe-size 小批,worktree/分支即焚)。
  终态注记(2026-08-27 收尾批):上游两轮推进后本批终验全绿——探针×2
  (60743/60753 bytes PASS)+build strict+vitest 23+1+对拍 30/30;
  auto-lang lib 全绿含 obj 语料 4/4(T2 靶子转绿=454-E5)。
  终态(2026-08-27 全闭):验收 #1-#6 全达——#2 经本会话跨仓微批
  (auto-lang master plan-046-B 合并点,lib 3219 绿)+ musk 端口 KV 化补完;
  四门禁终轮全绿(探针×2 60803/60805、build exit0、vitest 23+1、对拍
  30/30)。唯一留观=desktop 实机冒烟(高度生效+弹层居中),归入观察期
  手测清单,不阻塞入档。]

## 复审记录

**reviewer**: auto-plan:review(zhaopuming 会话)· **2026-08-27** · execution_done → **reviewed**

### 验收标准逐条复审(verify, don't trust)

| # | 标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | 探针 PASS+体积行;obj 规避形态 grep 清零(回撤位点>0) | ✅ pass | 探针 exit0、`linked-modules-total 60817 bytes`(fresh rerun);R1/R2/R3 三位点已自然 find/values 形态(relay_run_helpers.at:14 / relay_store.at:71 / forge_helpers.at);残留 VM-clean 标记×2 经定性均非 045 行③域(forge_store=D5 规约正例指针、mention_helpers=视口移植史注);R4 改判保留有据(vue-TS 墙实测) |
| 2 | platformViewportHeight 返回真实值;partial 转正 | ✅ pass | musk 端口读 KV(platform.vm.at:58)+上游链路实证(auto-lang master:storage_host_publish+startup/双 resize 发布共 7 处引用);runtime 实测留观察期手测(desktop 冒烟,备案不阻塞——headless 无法起窗) |
| 3 | 认证滞留存在代码级清理路径其一 | ✅ pass | 清偿协议固化注记(auth_store.Me / platform.vm.at 头注)+既有可调路径 platformRefreshAuth(login.at:116/App Init 已接线);401 兜底按用户裁定 a 不做,台账留边缘注 |
| 4 | relay 条件项(A1 依赖) | ✅ 按条文处理 | A1 未合入→BLOCKED 登记(KNOWN-DEBT 442 行尾+046 行独立阻塞件),条文明确"不视为失败但不标闭";A1 合入后由后续批接线 |
| 5 | 规约文档存在且被引;阈值可配置+依据说明 | ✅ pass | src/front/README.at-conventions.md 在案+KNOWN-DEBT 045 行④改指;probe 脚本头部阈值常数/env 覆盖/实测锚点注齐备(:15/:29) |
| 6 | 不动 web//backend/;auto-lang 改动走 worktree 有测试回归 | ✅ pass | 39d6977^..HEAD 全量文件清单无 web/backend;obj 族基线(worktree 流程,lib 3216 绿)+window-height 微批(plan-046-window-height,lib 3219 绿)均先 worktree 后 no-ff 合并 |

### 遗漏/延后/workaround 猎查(lazy-convergence)

- **延后(用户已签核)**:T9 relay 接线(A1 未合入,条件项路线)、desktop 实机冒烟(高度生效/弹层居中/回退前进/logout 补调四项并入观察期手测清单)。均有台账行,无静默遗漏。
- **workaround(如实入账)**:R4 keys 形态保留——非 VM 规避而系 vue-TS 对 Object.values 元素推 `{}`(for-of/索引双证),新立上游项 A′(ts_adapter 显式类型包装);体积门禁为合计字节粗粒度代理,per-module 精确记账留上游(046-D)。
- **行为面变更提示**:obj 基线合入后,musk 两处 Object.keys 消费(getErrandByToolCallId/session_token_cost)从"静默错值"转为真实计算——建议观察期实机核对 token 汇总与 errand 关联数据符合预期。
- **瞬态备忘**:复审中一次 auto build exit=1 复跑即 exit=0 且日志零错误(疑似 gen 产物文件锁竞争),连续两次干净通过后判定稳定。

### spec-impact 元数据

沿 PLAN-045 先例三字段留空(理由见 frontmatter 注):债务清偿/基建计划,
无 specs 域组件增删;规约沉淀物以文件+台账行交付。

### 路由裁定

六条验收全 pass、无阻塞债(仅用户签核的登记型延后)→ **status: reviewed**。
下一步 `/auto-plan:merge`(或 finish-plan 泛用收口路由)。

## 待澄清事项

1. **与 auto-lang 在途会话协调**:T1-d 发现改动集重叠（尤其 obj 族若他人已立项）
   时,是等待对方还是错峰实施?默认:挂起重叠项仅做无重叠部分,台账记同步义务。
   **[裁定(2026-08-27 用户确认):T2/T3/T6/T7 于 plan-447 合并后重入
   /auto-plan:work 续跑本计划补完**——T4 清册直接消费;其余任务已收口。]
2. **⑤ 规约 vs 修链接器**:本计划默认取规约化（成本低、模式本身合理）;若倾向修
   链接器请在此标注,届时 T11 改为上游修复任务。
3. **D2 回落策略**:native 读高不可用时是否值得保 720 回落,还是 fail-fast 暴露
   问题?默认:回落 + WARN 日志（探针产物里可见）。
4. **④ 若 A1 长期未合入**:观察窗口多久后彻底移除该条件项（转为独立计划）?
   默认:本计划范围内保持 BLOCKED 登记,不做时限裁断。
5. **认证滞留残余三选一(T5/T8 勘察后残余)**:现状=.Me 死码零暴露,清偿
   协议已固化于 auth_store.Me 注 + platform.vm.at 头注。可选项:
   a) 维持现状,UI 化 logout 落地时在派发点补调 platformRefreshAuth(默认);
   b) App.Init 增加 store.Me() 启动校验——行为变更(失效 token 开机即清,
      影响弱网/后端未起场景的静默降级),需用户确认;
   c) 401 兜底——前置=KNOWN-DEBT 046-C 上游 Http status 面暴露。
   **[裁定(2026-08-27 用户确认):取 a) 维持协议注记**——logout UI 落地时
   在派发点补调即闭,b/c 两案不做。]
6. **T10 阈值实测校准偏离**:计划原文 WARN≥30000/FAIL≥32767 基于"i16
   回绕为活跃硬约束"的前提;T2 前置勘察+实测证明 K1 已解(isize 化,
   60614 bytes 探针 PASS),按实况重校准 FAIL=131072 / WARN=90000。
   如需更严口径,`VM_PROBE_SIZE_WARN`/`VM_PROBE_SIZE_FAIL` env 即时覆盖,
   无需改码。
   **[裁定(2026-08-27 用户确认):接受实测校准口径 FAIL=131072/WARN=90000。]**
