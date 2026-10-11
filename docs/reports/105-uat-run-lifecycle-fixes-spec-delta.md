# PLAN-105 Spec Delta 对照 — SD-01/02/03

与 `docs/specs/modules/` 三个模块规范的 before/after 对照（计划 §5 规范增量
表的落地证据）。规范文本本体在 merge 阶段随 K2 行改写一并入 live
specs（本文件是 delta 对照工件，不是 live 规范）。

## SD-01 — chat-run-policy.md（AC-01/02/03）

| 规则 | before | after（本计划落地） | 证据 |
|---|---|---|---|
| 收束时限 | run_stream 返回即收束（隐含）；send 悬死靠 300s 看门狗死等 | ①流完成=run 收束秒级（形态 A/B 钉测 0.07s/0.06s）；②下一轮请求 send 阶段有界（client 60s 窗+单次重试，env `AUTO_AI_CLIENT_SEND_TIMEOUT_SECS`），异常终结为 Http 错误走既有失败臂 | T-01/T-02；`chat_run_llm_send_stall_converges_bounded` 红 10.15s→绿 2.11s |
| 在途消息 | guard 占用 → ⚠ 提示+丢弃（web 前端 queued 重试单轨） | 服务端待发队列：guard 占用入 `chat_pending`，收束后按序自动触发新一轮（人工边界=新消息 id+注入额度回复）；前端重试语义保留 | T-04；`chat_pending_message_replayed_after_run` |
| 自动续跑 | 无（UAT-K2 登记为设计现状） | 异常收束且有界自动续跑一次：env `AUTO_RUN_AUTO_CONTINUE`（缺省 1=开）、注入「继续：从中断处推进，不重做已完成部分」、注入轮自身异常不再链式、transient `auto_continue` 事件不落中间消息 | T-03；续跑/防链式/关闭路径三测 |
| 看门狗文案 | 固定「⏱ 运行空闲超时…建议把任务拆小后重发」（归因错误、无可操作指引） | 分型：停滞型（在途工具点名+拆小建议+「继续」指引）/挂起型（部分产物保留+「继续」指引+反复出现再拆小） | T-03；分型两测 |
| 配套 | — | agent 入口孤儿 tool_use 修补（drop-中止后复跑请求线合法化） | auto-ai `run_repairs_orphan_tool_uses_before_new_request` |

## SD-02 — workspace-sandbox.md（AC-04/05/08）

| 规则 | before | after | 证据 |
|---|---|---|---|
| run_command 引号 | 未约定（实际 `.arg(cmd)` 经 Rust argv 引用+cmd /C 剥引号损坏——UAT 四类命令实锤） | Windows `cmd /S /C` + 命令串整体 raw_arg（外层包引号=剥最外层原样执行，等价交互式 cmd 输入）；Unix `sh -c` 不变 | T-05；引号矩阵四测（commit -m 回读一致/-C 空格路径/curl -o 带空格输出名/findstr 多词） |
| 安全层词法 | `split_whitespace()`（引号盲——引号内越界路径以「首残片恰在根内」形态漏判） | `tokenize_command` 引号感知切词（双引号段不切分/引号剥离/相邻拼接/未闭合吞尾保守），confine×4 面共用；与执行层 cmd 语义对齐 | T-05；tokenizer 单测+漏判补红两例+根内带空格不误伤 |
| 进程收割 | run 结束无收割（`start` 分离孙锁 worktree） | spawn 凭证即时上报（Windows kill-on-close Job Object——分离孙唯一可靠面；Unix pid 进程组）→ run 级注册表 → chat_run_owner 全部收束出口+panic 守卫统一收割 ≤5s；仅 run_command 面 | T-08；runner 级+run 级端到端双测 |
| 邻接修复 | 超时臂读任务裸 await（管道被存活孙持有→悬死） | 读任务收尾 2s 有界 | T-08（15 分钟悬死实测复现后修复） |

## SD-03 — specs-ledger.md（AC-06）

| 规则 | before | after | 证据 |
|---|---|---|---|
| 状态串解析 | `from_str_lossy` 未知串兜底 Draft（帕斯卡 `Approved`→Draft） | API 边界 `parse_exact`：未知/大小写不符 → 400 带该区合法值清单+原串；lossy 仅留内部宽容面 | T-06；`pascal_status_rejected_with_reason_not_fake_ok` |
| 错误体 | 恒 `"transition failed"` 常串 | `transition_item` Err 原文透传（`from -> to` 详情/未找到原因） | `illegal_transition_400_carries_real_reason` |
| 响应回显 | `new_status`=请求串（假成功时纹丝不动也回显「成功」） | `applied_status`=服务端应用后真值；new_status 留作对照 | `legal_transition_applies_and_echoes_applied_status` |
| 大小写对称 | GET 帕斯卡（serde 变体名）、POST 只认 snake | 线格式统一 snake_case（Serialize rename_all）；读侧手动 Deserialize 帕斯卡归一兼容（`UnderReview`→枚举，未知串报错） | GET/POST 对称回写闭环测；前端本就 snake 约定（specs_helpers.at 全 snake） |
| upsert 直写 | 绕状态机可直写任意状态（无标注） | 直写保留（Q-2 默认）+ `bypassed_transition` 标注（写入状态≠现状且非合法转移时 true；响应顶层，磁盘 doc 不落） | `upsert_bypassed_transition_annotation` |

## 与 PLAN-104 SD-02（specs-ledger）的合并时序

两 delta 同目标文件、无任务重叠（104=spec 工具注册/HTTP DTO/技能文本；
105=状态机 API 诚实性）。合并时序按计划合入先后，后到者 rebase 适配；
规范终态以两计划 AC 并集为准（104 先执行中——本计划合入时按需适配）。

## 未尽事项（merge 阶段收口）

- KNOWN-DEBT K2 行改写（自动续跑缺省值随 K2 一并回用户确认——Q-1）。
- Q-2 upsert 直写权限的最终裁决（本计划按默认方案落地并标注）。
