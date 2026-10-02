# PLAN-098 V03 随仓 fixture（脱敏样本）

来源与脱敏口径（PLAN-098 §5.5：fixture 随仓持久，真实工件只在现场消费）：

- **run-fixture-098**：relay run 样本，turn 行形状逐字段对齐真实产物
  `tmp/demo/.autoos/conversations/run-094-humanarm-8975/turns.jsonl`
  （main@da748ac 仍在，PLAN-094 重跑产物）；时间基线 1790655343 取自该真实
  run。内容全部替换为无信息 stub（计划正文/工具结果/复审意见）；相位结构
  刻意构造为**签名阳性样本**：K5（plan 相位无 create_plan）、K3（同参
  run_command ×4）、K1（收束请求 out=4096 / stop=max_tokens）、K4（gate
  waiting 与 Step started 同秒）。
- **chat-fixture-098**：chat 会话样本，对齐 2026-09-29 UAT 会话产物形状；
  含一轮**旧格式** assistant 轮（无 telemetry 字段——AC-04 兼容面）与
  流式 in=0 / 非流式 in=123 两种 usage 形态。
- **aaid-fixture.log**：daemon 日志片段，行格式取自 auto-ai@5a50a55
  `auto-ai-daemon/src/server.rs` 实测格式并保留 ANSI 序列（脚本必须能去
  ANSI 解析）；`chat stream start` 行携带 provider（`chat stream done` 不
  携带——provider join 依赖 start 行）。
- **expect-*.json**：V03 `--expect` 断言集（确定性检查；--expect 不符退出 3，
  基线契约 0=产出 / 2=输入缺失）。

aaid 聚合计数（chat_req 等）是**整份日志级**事实（日志跨会话共享）；
逐请求归因（provider/confidence）才是按 telemetry 行时间窗 join 的结果。
真实工件现场分析时同理：日志计数≠本 run 计数。

生成器为一次性脚本（tmp/gen-098-fixtures.mjs，不入库）；本目录为静态
产物，改动走重生成后人工核对。
