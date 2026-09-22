# VM 数据读侧契约（vm-data-semantics）

> PLAN-066 T-04~T-07 交付的双轨等价契约收口：VM 轨数据读侧与 web 轨同语义
> 的 enduring 规则与已知边界。

## typeof 属性抢占收窄（KD-057① 清偿）

`obj.type` 域读不再无条件抢占为编译期型名：typeof 语义仅原始接收者生效
（`StrFixed(0)` 动态推断哨兵排除），对象接收者落 GET_FIELD 通道与 web 轨
a2ts 纯属性访问同语义。musk ~90 处 `.type` 字段比较（forge_store ev.type
分派/questionnaire q.type 等）随之解锁。

## Regex.match JS web 语义（KD-057② 清偿）

三参契约 `[text, pattern, flags]`（自顶向下）：无 `'g'` 返回
`[全匹配, 组1, …]` 堆列表；含 `'g'` 返回全部匹配子串列表；空匹配空列表；
两参调用 codegen 编译期补 `flags=""`；`'i'` 支持。回归锁：musk_vm_track
p066_2（wl_probe18 全形态+组提取+583 元素存活锁）。

## str.includes 等实例方法（055-4⑥ 现代真身清偿）

合成 fn（handler/computed）内 `str.*` 未注册方法（includes/startsWith/
trim 族）路由引擎 CALL_SPEC str 臂（引擎已实现族随通）；`auto.str.includes`
另注册原生 shim（id2461，实例 CALL_NAT `[pat, receiver]`）。回归锁：语料
`test/ui/plan066_filter_projection/` PA（过滤投影命中 2/清空 3/miss 0）。

**Face A 已知边界（未修，另行立案候选）**：列表字段读改写
（`.messages = .messages + [..]`）在旧值陈旧/Nil 时静默失效；整写正常。
musk 生产消息列表全为后端快照整写，未踩。

## P536-D2 SET_FIELD 裁定

跨模块调用帧内 SET_FIELD 写共享根态——当前树实机语料裁定**可达**
（MarkDone→ClearWindow 自调链写落盘；624 闭包帧协议修复或更早已覆盖）。
musk `done` 臂「本臂自清」绕行不再必要但保留（防御性等价）。

## ThinkBlock chevron 读侧独立（KD-057③/T-13 清偿）

子件 computed 串等值（`isOpen => .expanded == .current_msg.id` 形态）+
子件 handler 直调 store msg 翻转，全链回归锁绿（语料 PC 三断言：翻转
expanded="m1"/展开 true/收起 false）。obj-prop 传递依赖逐帧烘焙（生产
每帧重建），纯静态 harness 不可测——行为面以实机验收。

## use 模块名纪律

`use <module>: <Symbol>` 的 module 名必须与文件名一致（resolve 按名找
文件，`use msg_bubble:` 配 bubble.at 永远解析失败且静默）。

## PLAN-081 增量：location.reload VM 语义 + 嵌套对象读/for 包列铁律（SD-02/SD-03）

- **`location.reload()` 在 VM 轨 = no-op（SD-02，AC-02）**：auto-lang
  `shim_dom_reload` 空实现是事实源。需要"整页刷新"语义的功能**禁止依赖
  reload**，必须显式走 store 级刷新链（见 ui-compositions.md PLAN-081
  增量的切换完成语义）。
- **嵌套对象字段读铁律（SD-03，AC-11）**：API 值树经 store 根态中转后，
  **渲染/计算上下文对嵌套对象（obj 字段链）的字段读不可靠（产出 "0"）**；
  字符串读全上下文可靠，native `JSON.stringify/parse` 任意上下文可靠，
  handler 上下文读可靠。**处方：ingest（handler 上下文）把后端 JSON
  载荷拍平为纯字符串字段 + 现算串**（`normalizeToolBlocks`/
  `summaryTextOf` 先例；args→args_json、summary 现算），模板与 computed
  只读字符串字段。他店（errands/relays/task_plans/specs 等）同症按同方
  处理。store 文件内联 fn 改名防 VM 扁平命名空间与 helpers 版撞名。
- **row 内 for 子树包装为列（SD-03，AC-11/13）**：VM 渲染器把 row 内
  for 循环子树包装为列（多段竖排），且容器 gap 对该列不生效——**行内
  多段必须预拼接为单串单 text 节点；块间距必须用块 wrapper margin**。
- **依据**：PLAN-081 r5 实机两实例对照定罪（vtree 真值 vs 像素全 "0"；
  BLKDBG pre/post 实证 handler 读真值；收据
  docs/reports/ui-parity/081-workspace-switch-ux.md r5 节；musk 0b70043）。

## PLAN-083 增量：数据链分页契约 + 异步两段式 + 归一化直出（SD-01）

- **会话详情分页契约（AC-01）**：`GET /api/chats/session/{id}/page?
  workspace=&limit=&before=`——`limit` 缺省 50、上界 500；`before=
  <message_id>` 取该 id 之前（不含）的最近 N 条（消息按存储序旧→新，
  页=窗口尾部）；响应 `{session, has_more, next_before,
  blocks_normalized:true}`，`next_before`=本页首条 id（下一页游标，
  无更多时空串）。**缺省全量的旧端点 `/session/{id}` 字节不变**——web
  旧消费零影响，web 轨跟进分页属后续排期。
- **归一化直出（AC-02）**：paged 端点由后端按前端 `normalizeToolBlocks`
  r5b 契约直出拍平块——tool 块纯字符串字段
  （tool_name/tool_id/tool_status/tool_result/tool_gate_id/
  tool_pending_cmd/tool_escape_paths_text/tool_args_json/summary 现算，
  summary 口径与 TS `summaryTextOf` 逐分支对齐含 60→57/80→77 截断）；
  thinking 块 `{kind,text,state:"done"}`。消费侧免本地归一循环。
  **`tool_args_json` 恒为合法 JSON**（消费侧 messageBlocks r5b 臂逐块
  JSON.parse 重建 arguments；截断臂超限→`{"_truncated":true}`、预算
  降桩臂→`"{}"`——非法串炸掉整条渲染 computed=AI 气泡整空，PLAN-083
  V-4 二分定罪）。
- **块级瘦身**：巨条（实测 552KB~938KB=84~160 个工具块）分页数限制不
  住——per-block 截断（tool_result 4K/thinking 4K/args 2K 文本类 16K，
  截断块携带 `truncated:true`）+ 每消息 48KB 序列化预算（尾部优先保真，
  超预算更早 tool 块降桩）+ 消息级 `tool_calls` 瘦身为 thin 数组
  （id/name/status；载荷只留 blocks 一份——652KB 条里重复数组独占
  225KB）。实测 938KB 会话首屏整页 106KB（8.8×）。
- **VM 数据加载模式=异步两段式（AC-03/04）**：**store handler 发起段禁
  长阻塞**——数据加载一律 `Http.get_msg(url, "Store.Handler")` 消息桥
  （fire-and-forget：入队派生线程即返回，无 task Waiting=无
  call_fn_by_name 忙等）；完成以 `{"ok","status","body"}` JSON 字符串
  实参回填 handler（渲染层订阅泵 19ms→on_with_input_for ␟s␟ 载荷）。
  **长冻结（handler 内同步 await #[api] 全量拉取+本地归一循环）=违反
  本契约**（082 §10-7 实测 20.9s 冻结的根因）。Web 轨同构（ts_adapter
  fire-and-forget IIFE + 同载荷协议；跨 store 派发 VM 专有）。
- **回填段重建铁律**：`JSON.parse` 产物**不得直接落 store 供渲染**——
  blocks 深度字段读在渲染/computed 上下文塌空（本节 PLAN-081 嵌套读
  铁律的深度表现）；回填 handler 必须经 `rebuildParsedMessages` 类
  扁平重建（handler 上下文字面量重建，分页载荷已是纯字符串块，重建=
  纯拷贝毫秒级）。
- **轮询回填窗口合并（073 链适配）**：流式轮询（PollStream→PollBackfill）
  以分页快照**按已加载窗口合并**（快照首条在现表有锚点→保锚点前缀换
  尾部；无锚点→整表换入）——用户已翻页加载的历史不被轮询覆盖；
  pending 快照/回合增长守卫/收束排空语义不变。
- **依据**：PLAN-083 T-01..T-05（auto-lang `auto-musk-dev` 2×commit：
  native 3148+队列+泵+语料探针 5/5、ts_adapter web 臂；musk
  `plan-083-dev` 5×commit：page 端点 8 测绿+938KB→106KB 实测、两段式
  迁移、V-1..V-4 实机矩阵全 PASS（0 不可响应轮次/首屏 414~754ms/
  翻页 264ms/巨条像素实证）——收据 tmp/v083-evidence/ + 本计划 §9）。

## PLAN-084 增量：会话归档与列表元数据（SD-05 + 分组元数据）

- **ChatSession.archived**（`#[serde(default, skip_serializing_if=not)]`——旧档
  JSON 零 diff/旧前端零感知；ag 镜像不承载=「镜像仅承载 parity 所需字段」
  哲学）：归档=列表级隐藏（list 滤除/list_archived 对偶面），不删消息、
  **不 bump updated_at**。
- 端点：`PATCH /api/chats/session/{id}/archive`（body {archived:bool}）、
  `GET /api/chats/sessions/archived`（chat_rename 同款补线先例）。
- **列表分组元数据后端现算**：summary 增 day_group(0=今天/1=昨天/2=更早)/
  time_text("HH:mm")/date_text(本年 MM-DD/跨年 YYYY-MM-DD)——本地时区 chrono
  现算（VM 无可用时钟：Date.now() 返负垃圾既有缺陷）；VM 轨日历运算收敛后端。
- **get_msg 桥 sessions 位重建漏斗**：rebuildSessions 逐字段重建（JSON.parse
  产物直接落 store 渲染上下文深读塌空=r5 家族 sessions 位显形；083 只给
  messages 配了漏斗）。
