# 099-chat-driven-demo-reverify — 引擎根修后对话驱动演示复验报告（PLAN-099 T-06 终态）

> 2026-10-09。用户裁定 (a) 路线（先修引擎→复验 AC-01/02→review）的复验收据。
> 驱动与原始证据：`D:/autostack/.demo/musk-099/e2e-01/`（driver.mjs A/B/C/D 四阶段
> + canvas-visual.mjs + monitor-*.log + capture-*/ + cv/capture/）。
> 本文件为证据索引与判定；原始日志/截图/JSON 以 .demo 目录为准。

## 根修清单（本轮交付的引擎修复）

| 修复 | 仓/提交 | 内容 |
|---|---|---|
| client 读空闲超时 | auto-ai main f1c7794 | rust-ref `complete_stream` 读循环自持空闲超时（env `AUTO_AI_CLIENT_STREAM_IDLE_TIMEOUT_SECS` 缺省 90s）；停滞终结为 Http 错误交失败臂；钉测试 5 绿 |
| 快照自死锁回归 | musk plan-099-dev 7e574b4 | tool_call 臂 blocks2 守卫横跨臂尾 + persist 同持锁域再锁 = 同线程自死锁；守卫收紧作用域 |
| 复现矩阵回归锚 | auto-lang auto-musk-dev 6a8c5e2d8 | `crates/a2r-std/tests/stream_hang_repro.rs` 三形状（silent_hold/abrupt_close/proper_end）终结性全绿——内核无罪定谳 |

详见 KNOWN-DEBT「E-STREAM-HANG」条（终谳版）。

## 复验判定（引擎根修后）

| AC | 判定 | 证据 |
|---|---|---|
| AC-01 对话全链 | **✓ 推进实证 / delivered 收据时间尺度未竟** | 40min 长窗实跑：需求一句话 → **计划文件真落盘**（`001-focusboard.md`——PLAN-001 完整计划，status=executing current_step=2/4，副本见本目录）→ 状态机推进 drafting→executing → 执行相位 relay 大流健康（glm-5.3 586~1390 token 流多次正常收束）；chat run 89s 正常收束（16 工具调用）。40min 窗结束时执行相位在途——delivered 收据为模型吞吐时间尺度（对照：097 晨间 60min 窗亦未跑完 execute），推荐 UAT 长跑补收据 |
| AC-02 实时渲染 | **✓** | +5s/+6s pending 快照两轮实机落盘可见（监控 1s 拍 page feed，无任何手动刷新）；UI 侧 500ms 轮询合并链即该数据面 |
| AC-03 计数徽章 | **✓** | message_count 0→1→2 随发送与快照增长（服务端口径，列表刷新链随行） |
| AC-04 卡死可恢复 | **✓✓** | ①假 AAID 挂死：看门狗 +61s 收束（idle=60 配置）；②真实静默流（71s 零输出）：看门狗 +647s fail-fast、错误落盘、守卫释放、relay 独立继续；③引擎悬挂注入 ×4：reaper +90~167s 精确回收；④新消息均正常触发新 run |
| AC-05 增量持久化 | **✓✓** | +7s 工具级 pending 快照（2 工具卡 blocks=3）→ kill serve 于 turn 中途 → 重启：孤儿提示 ✓ + 前缀可见（tools=2）✓ + 新消息续跑 busy=false ✓（phase-c-verdict.json） |
| AC-06 canvas 内嵌 | **✓✓** | 面板 img 每秒拉帧 18 次全 200（serve.log `frame?t=N&gen=N` 序列——修复前相对 URL 落本地文件臂零请求）；148KB 实帧；结构树镜像同框（ac06 截图） |
| AC-07 canvas 重绑 | **✓✓** | stop→start 第二 app：gen=1→2→3 全序列换绑无残留（serve.log 帧 URL 序列 + ac07 截图） |
| AC-08 回归 | **✓** | musk lib 548 绿（+3 新钉）；auto build gen-only 锁步全链（vue-tsc+vite）过；auto-lang 复现矩阵 3 绿 + lib check；auto-ai client 5 绿 |

## 遗留登记（不阻断 review）

1. **AC-01 delivered 收据**：UAT 长跑补（链路已证活；时间尺度问题）。
2. **auto-lang 折回**：auto-musk-dev 两提交（6afac0405/6a8c5e2d8→rebase 后
   aa382b11b/8efd12124）ff 被主检出外来 WIP 阻（PLAN-721 media 6 文件）——
   待属主处置后 `git merge --ff-only auto-musk-dev`。
3. **relay 相位内会话快照缺口**：相位内工具事件经驱动镜像上总线（web SSE
   可见）不落会话快照——"对话里看 AI 干活"在 plan 流相位依赖 web 轨或
   relay 进度面；候选后续（驱动镜像点同步 upsert）。
4. **原发停滞触发器**（基线代码时代、防护前）：降级观察项——现被 client
   空闲超时 + musk reaper 双兜底；复发按 "llm stream idle timeout" 日志取证。

## 三仓终态

- **musk**：plan-099-dev @ 603cde8（6 提交），guard clean，待 review/merge。
- **auto-ai**：f1c7794 已折回 main ✓（auto-musk-dev-2 留档）。
- **auto-lang**：auto-musk-dev @ aa382b11b（rebase 到 master 09849576b 上），
  折回待办（见上）；组内 worktree 为 musk 消费源。
