---
plan_id: PLAN-073
status: execution_done
feature_name: chat 运行挂死修复（看门狗命令盲区 / daemon 静默吞参 / 超时收束丢证据）
author: zhaop / zcode
created_at: 2026-09-18T22:00:00+08:00
updated_at: 2026-09-18T23:10:00+08:00
plan_revision: 2
current_step: 6
total_steps: 6
touched_repos: [auto-ai@630a98d, auto-musk@ae86fb0]
---

# PLAN-073 — chat 运行挂死修复

会话 `90ed3ae00e4d6a6321f7af76`（auto-edit，"@plan/001 实施这个计划"，21:28:59 →
21:34:19 空闲超时中止）实测暴露三缺陷。诊断已实证（诊断记录见 PLAN-072 §4 同期
会话），本计划实施修复。

## 0. 根因（已实证）

1. **看门狗命令盲区**：空闲看门狗只被 agent StreamEvent 喂（extern_impl.rs:2071）；
   run_command 执行期不产生 agent 事件（ToolStart 已发、Tool 结果等命令退出），
   100ms 输出进度走 tool_update 桥到 UI 但**不喂狗** → 长命令执行 ≥300s 必被误杀。
2. **双 300s 竞速**：命令默认超时 300s（tools.rs:369，AUTO_CMD_TIMEOUT_SECS）==
   看门狗窗口 300s（server.rs:61，AUTO_RUN_IDLE_TIMEOUT_SECS），同时到期看门狗先胜。
3. **daemon 静默吞参**：流式工具参数为空/JSON 解析失败时，openai.rs:348-370 与
   anthropic.rs:405-418 均静默替换为 `{}`（仅 stdout warn）→ UI 出现"无参数"工具卡，
   模型拿到 missing-args 错误盲目重试。
4. **超时收束丢证据**：空闲超时路径只持久化超时消息，已积累的
   thinking/tool_calls/blocks 全部丢弃（对比成功路径 extern_impl.rs:2327+）。

## 1. 实施

- [x] **T-01** musk：tool_update 桥喂看门狗——bridge 任务收到
  tool_update/tool_gate_waiting/relay_gate_waiting 事件时 `wd_tx.send(now)`。
- [x] **T-02** musk：命令默认超时 300→120s（AUTO_CMD_TIMEOUT_SECS 仍可覆盖），
  令命令自然上限远小于看门狗窗口，长命令超时走自身收束而非看门狗误杀。
- [x] **T-03** auto-ai：吞参显式化——`StreamDelta::Warning(String)` 变体；
  openai/anthropic provider 在空参/解析失败替换 `{}` 时 emit warning（含工具名、
  原因、原始长度/头部）；daemon server.rs 增 `{"type":"warning","text"}` 帧分支；
  agent `forward_sse_delta` 映射 warning 帧 → `StreamEvent::Warning`
  （warning 帧带 text，必须先于通用 text 路径分流，否则告警文本被拼进正文）。
  转换逻辑提取 `tool_calls_from_accum`/`tool_calls_from_blocks` 以可测。
- [x] **T-04** musk：超时收束持久化——空闲超时分支按成功路径同款组装已积累
  thinking/tool_calls/blocks，超时通知作尾块追加后落盘（单消息 + turns 镜像）；
  另：on_event 事件分派增加 `warning` 臂，⚠️ 文本块入时间线（用户可见）。
- [x] **T-05** 验证：auto-ai daemon 64 + agent 133 测试全绿（含新增双 provider
  降级行为单测）；musk 437 测试全绿（含新增 `default_cmd_timeout_resolves_to_
  120s`；既有 `chat_idle_watchdog_times_out_hung_run` 等看门狗用例不回归）；
  musk worktree 经路径依赖直接消费兄弟 auto-ai worktree 编译链接成功（组布局
  即验证）。live 复现指引（GLM key 在用户环境）见 §4——参数丢失属模型侧还是
  分片侧的最终定性留待该复现，provider 层行为已单测钉死。
- [x] **T-06** 收尾：auto-ai 合回 main（**630a98d**，worktree/分支/auto-lang
  兄弟快照清理）；musk 合回 main（**ae86fb0**，worktree/分支/组目录清理；
  wt-guard 三 worktree 全 clean）。

## 2. 验收标准

- [x] **AC-01** 长命令不再被看门狗误杀：命令执行期 tool_update 事件刷新空闲窗；
  静默命令 120s 由命令超时自身收束并返回结果事件（模型可见），看门狗仅对
  真停顿（LLM/管道级）兜底。
- [x] **AC-02** 吞参可见：畸形/空参时 UI 收到 warning 帧，消息时间线出现
  ⚠️ 块；daemon 不再无痕迹替换（双 provider 单测锚定 warn 触发与内容）。
- [x] **AC-03** 超时会话保留现场：空闲超时落盘的消息包含中止前的
  thinking/工具卡/时间线，超时通知在尾部。
- [x] **AC-04** 双仓既有测试全绿 + 新增行为测试（warning 帧/映射/默认超时值）。

## 3. 影响面与兼容

- warning 帧为新事件类型：老前端/CLI 未知类型忽略（既有行为），向后兼容。
- 命令默认超时变更只影响未显式传 timeout 的调用；模型显式传值不裁剪（文档化：
  显式值 > 看门狗窗口时仍可能被看门狗中止——进度流动则不受影响）。
- 已知残留（登记不阻塞）：human 审批门（gate）等待 1800s 上限 > 看门狗窗口，
  等待期无心跳仍会被中止——需门工具心跳或看门狗门状态感知，另立计划。

## 4. live 复现指引（参数丢失根因钉死，用户环境执行）

```bash
# 终端 A：debug 日志起独立 aaid（不同端口，避免动正在跑的 17654）
RUST_LOG=debug ZHIPU_API_KEY=<key> aaid --listen 127.0.0.1:17655 2>&1 | tee aaid.log
# 终端 B：独立 musk 指向它 + 缩短窗口
AAID_URL=http://127.0.0.1:17655 AUTO_RUN_IDLE_TIMEOUT_SECS=60 AUTO_CMD_TIMEOUT_SECS=30 \
  musk serve --addr 127.0.0.1:8095 2>&1 | tee musk.log
# 终端 C：对 8095 发同型消息（@plan/001 实施这个计划），观察
# aaid.log 的 `streaming tool_call: name=... args_len=` 与
# `streaming: tool_call ... empty/malformed arguments` warn 的先后关系
```

## 5. 记录

**实施完成（2026-09-18 23:10，r2）**：T-01..T-06 全勾，AC-01..04 全过。
auto-ai 分支提交 630a98d → main merge；musk 分支提交 ae86fb0 → main merge
（fast-forward）。收据：wt-guard 三 worktree clean；测试对 daemon 64 +
agent 133 + musk 437。**生效前提**：需重编译部署 musk.exe 与 aaid.exe
（本轮已在两仓 worktree 构建通过；用户运行态的 8090/17654 为旧二进制，
重启/替换后生效）。同会话遗留观察：human 审批门 1800s > 看门狗窗口，
gate 等待期无心跳仍会被中止（§3 已登记，另立计划）。
