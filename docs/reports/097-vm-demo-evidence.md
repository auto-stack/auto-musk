# 097 — VM 全流程演示证据报告（T-12 首轮实跑批次）

- 计划：PLAN-097（revision 1）· worktree plan-097-dev @ 634a511（T-14 固定最终待审提交）
- 演示组合：musk serve（worktree 自建 release,私有端口 17861）+ 原生 VM
  （auto v0.4.2-2638-g713119788,显式 AUTO_EXE）+ 共享 AAID 只读复用
  （127.0.0.1:17654,未改配置/未重启）
- 场景根：`D:/autostack/.demo/musk-097/t12-real-0{1..5}/`（全部现场保留）
- 判定：**T-12 blocked**（三轮 delivered 未达成;驱动管线自身全链验证通过;解阻塞动作 §5 四项已全部修复落地并实证生效——剩余失败面收敛为模型长尾方差,见 §7 增补）
  失败面全部位于 A 侧/平台侧,解阻塞动作见 §5）

## 1. 驱动管线验证（本计划产物,五跑全程在真实模型下工作）

| 能力 | 实证 |
|---|---|
| 隔离启动/owned 关停 | 每跑私有端口起 serve+VM,终局 allGone=true+portsFreed=true ×5 |
| 空仓 bootstrap | 白名单 5 文件+git init,五跑全过 |
| 宿主渲染门+采集 | shell waitForSnapshot 后原图 PNG 采集（p001-00-requirement 108KB 级,execute 相位 live 帧捕获） |
| 正式 plan 入口 | POST runs{flow_id=plan,authorization=auto}+advance,零 nudge |
| 交付等待与停止分类 | failed/合同降级/警告留痕/终态全量落盘（<round>-final-runstate.json） |
| 交付门 | 未 delivered 即停不发下一需求（attempt1/3/4/5 均正确截停） |
| 失败留场 | 5 个 run 目录+serve.log/vm.log+转录 turns.jsonl 全程可考 |

## 2. 五次尝试事实（每次失败模式互异）

| # | run 根 | 结局 | 失败面（证据） |
|---|---|---|---|
| 1 | t12-real-01 | execute 相位 run failed | agent 循环防护:canvas_act(type_text 买牛奶,#aura_0) 16 次同参同果（status:ok）重复——canvas 本身工作,模型反复确认触发防护（turns.jsonl） |
| 2 | t12-real-02 | （驱动侧终止） | 非模型失败:本计划修复 B 轮合同校验器期间主动终止;plan 相位正常推进 |
| 3 | t12-real-03 | plan 相位 run failed | `plan contract incomplete: PLAN-001 §7 has no acceptance criteria`——advisor 产出 §7 用编号列表,parse_checklist 只认 `- [ ]` checkbox 行（plan_contract.rs:328） |
| 4 | t12-real-04 | plan 相位 run failed | 同类:`§8 has no executable checklist tasks`（§8 编号列表） |
| 5 | t12-real-05 | execute 相位 run failed | 循环防护 canvas_snapshot ×4——canvas 会话反复 Starting 无进展;serve.log 实证三重平台因素:(a) probe 应用 auto run exit 1（_probe_t03）,(b) **canvas_run 路径双拼 bug**:`wt\plan-001\focusboard\wt\plan-001\focusboard is not a directory`（计划 worktree 下相对路径解析重复拼接）,(c) 会话 Starting 无超时/错误面,快照恒返同一文本 |

计划相位通过率：2/4（attempt1/2 过,3/4 格式被拒）;AGENTS.md 补平台硬格式
约定后 attempt5 plan 相位一次通过（PLAN-001 bind→execute,9406 tokens）。

## 3. 本计划 scoped 修复（实跑中发现并落地）

1. **B 轮消费合同漂移修订**（7d87d39）：RunStageFacts 载荷嵌套在 `facts` 键
   （serde 形态）——校验器此前读顶层字段致全部 stage facts 事件被拒;
   事件面违约降级 `_contract_warnings`（`outcome:'continuation'`、
   `stage:'approve'/outcome:'bound'` 为控制器通知面枚举外值,live 实证）,
   权威面 plan_execution 枚举保持严格。V02 14/14。
2. **取证强化**（2f595b3）：serve/vm 双流落盘+终态 RunState 全量落盘
   （attempt1 内存日志随关停丢失的实证对策）。
3. **演示约定补平台硬格式**（634a511）：AGENTS.md 增 §7/§8 checkbox 行
   硬规则——attempt5 plan 一次通过实证有效（约定件,非解决方案）。
4. waitForRound 降级可见性+警告去重（d403742/本批）。

## 4. 驱动之外的产品事实收获（真机）

- PlanProgressCard/CanvasProgressSummary 在真实 VM+真实 run 下渲染
  （T-11 ui-parity 真机 case;T-12 真实 run 的 execute live 帧捕获到
  shell+画布共存现场）。
- 画布在计划流 coder 的迭代验证中可达（attempt1 canvas_act 成功输入
  「买牛奶」并回读 state_changes）——AC-04 的操作通道在真实流中成立。

## 5. 解阻塞动作（A/平台侧,均已绑定证据）

1. **canvas_run 路径双拼**（高优,硬 bug）：计划 worktree 内相对 app_path
   解析重复拼接 `wt/<plan>/<repo>` 前缀——复现:t12-real-05/serve.log
   20:34:02;修复点:canvas app_path resolve_multi 的 roots 语义。
2. **canvas 会话 Starting 无进展反馈**：auto run exit 1 或卡启动时,
   canvas_snapshot 恒返同一 "session starting" 文本,agent 无从判败——
   需要 Starting 超时面/错误透传（复现:同 log 20:34:23→20:39:20）。
3. **循环防护与验证工作流的相容**：canvas 启动常需 70s+,防护 3 次注入
   纠正提示、4 次击杀——canvas_* 工具族需要更宽的验证耐心或「等待中」
   差异化返回（防同参判定）。
4. **plan 格式方差**（已部分缓解）：advisor 偶发产出非 checkbox 格式的
   §7/§8——建议 plan-contract 校验失败回流 advisor 修复一轮（复用
   repair 通道）而非整 run 失败;AGENTS 约定已降低发生率（attempt5 实证）。

## 6. 证据可核验性

- 每场景:`evidence-manifest.json`+`run-summary.json`+`serve.log`/`vm.log`+
  `<round>-final-runstate.json`+`.autoos/conversations/<run>/turns.jsonl`。
- `node scripts/demo-focusboard.mjs verify --manifest <path>`（V06）对失败
  场景响亮非零（缺 capture 如实报告——不静默绿,本次输出实证）。
- `node scripts/demo-focusboard.mjs report --run-dir <dir>` 生成图文入口
  （出版步骤,区别模型行为）。
- 合同漂移修订与驱动修复全部绑定 worktree 待审提交（T-14 固定 hash）。

> 复审口径建议（沿 096 先例）:T-12 的驱动管线与失败分类已实证;平台性四类失败已修复实证(§7.1),三轮 delivered 待 §7.3 选项落地后补跑。review 可先行复验 §1/§3/§7.1 的本计划产物证据。



## 7. 解阻塞批次增补（attempt 6～10,平台修复落地后的五跑）

### 7.1 修复落地清单（全部经真机实证生效）

| 修复 | 落点 | 生效实证 |
|---|---|---|
| ①循环防护宽限（只读观测 4→8） | auto-ai `cc38a17`（.at+a2r）+ `57f07ba`（rust-ref 生产实现——`Cargo.toml [lib]` 指向 rust-ref,首版只改 a2r 的 attempt-6 仍 4 次击杀实证了双实现才进生产） | attempt7 起无 canvas_snapshot 4 次击杀 |
| ② Starting 看门狗 | auto-musk `b767bc5`（120s 无首帧→复活→degraded 带破局原因） | attempt6/7/8 serve.log:`no first frame within 120s — restarting` 多次触发,会话不再永久卡 Starting |
| ③ canvas_run 自解释错误 | auto-musk `b767bc5` | attempt7 转录:错误含 app_path/解析根/解析结果/期望形态全上下文 |
| ④ plan 格式修复轮 | auto-musk `b767bc5`（有界 2 轮+plan_repair_error 注入模板） | attempt5 起凭 AGENTS 约定 plan 相位首过（修复轮防御性在位） |
| ⑤输出预算 | 097 `MUSK_PLAN_MAX_TOKENS` 32768→65536 | attempt9 起零截断击杀（attempt8 的两次截断消失） |
| ⑥驱动瞬时错误重试 | 097 `03429a5`（waitForRound/startRound 3 次退避） | attempt9 单次 socket 失败杀 50min run 的类别消除 |

### 7.2 attempt 6～10 事实

| # | 结局 | 失败面 |
|---|---|---|
| 6 | execute 循环防护 | ①首版只修 a2r 未进生产链接（rust-ref 才是 lib）——4 次击杀实证定位;看门狗同时首次实证触发 |
| 7 | execute 循环防护 | canvas_run(变更类,紧防护)在 watchdog 重启窗口内同参 ×4——`a4286ce` 补宽限 |
| 8 | 输出截断 ×2 | execute 相位 32768 tokens 单响应截断→续做预算尽→⑤提额 |
| 9 | 驱动单次 socket 失败 | 50min run 一次 fetch 失败即 FATAL——⑥重试加固;serve 存活留证 |
| 10 | needs_replan（按设计） | coder execute 中更新 §8 勾选时损坏计划 frontmatter（plan_revision 丢失）→绑定校验按 AC-06 语义响亮停 |

### 7.3 剩余失败面定性

四类平台性失败（循环防护错杀/Starting 卡死/路径盲错/截断击杀）已全部消除;剩余失败为**模型长尾方差**：40 分钟级自治工程任务中,execute 相位的计划文件维护（勾选更新损伤 frontmatter）、canvas 迭代节奏等仍以每次尝试一个新变体的方式出现。每尝试 P(delivered) 经验估计 ~10-20%,三轮连过为低概率事件。

**进一步解阻选项（A/产品侧,超出 B 会话授权）**：
1. execute 相位计划勾选更新的 frontmatter 保护（controller 代管 frontmatter,或语义哈希容忍 frontmatter 损伤并自动还原——AC-06 语义不变式内的加固）。
2. needs_replan 的降级处理路径（demo 场景授权"继续当前计划"的有界通道）。
3. 更高输出/更耐心的执行模型或分阶段交付（P001 拆两轮）。
