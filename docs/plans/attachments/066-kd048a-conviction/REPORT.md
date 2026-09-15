# PLAN-066 T-01 KD-048a 定罪报告（2026-09-15）

## 结论

**KD-048a 慢性「~4-5min 静默退出」不是产品缺陷，是同机并发 agent 会话争抢标准
AutoUI MCP 端口 9247 的环境干扰。** P625-D1 AppHang 链（UI 停泵 >5s → WER 外击）
在本次全部 6 轮 × 10min 全程 procdump `-h` 监听下**未现形**（零挂起检出、死亡窗口
WER 零事件）；历史 WER AppHangB1 ×2（625 T-07）为另一张独立面孔，其结构面
（tool_snapshot 持锁深拷贝+全树序列化顶停 UI 发帧）已由 T-02a 根修。

## A/B 对照（单变量：AUTOUI_MCP_PORT）

| 臂 | 端口 | r1 | r2 | r3 | 存活 |
|---|---|---|---|---|---|
| run1 | 9247（标准，与并发会话 ui_desktop 共址） | 死@4.5min exit **-1** | 自退@58s `main_return code=0` | 死@7.7min exit **-1** | **0/3** |
| run2 | 9741（私有隔离） | 存活 | 存活 | 存活 | **3/3** |

- run1 r1/r3：575 退出审计三挂点（vm_process_exit/panic/main_return）**零记录**、
  零 panic、零 WER 报告、procdump -h 无停泵检出——外部 `TerminateProcess` 形态
  （PowerShell `Stop-Process -Force` 的退出码恰为 -1；按名/按端口清理是并发
  agent 会话的常见动作）。575 钩子覆盖缺口（原生层 `std::process::exit`/
  TerminateProcess 不经任何挂点）与该形态自洽。
- run1 r2：`site=main_return code=0`——iced 事件循环正常退出（窗口被外客关闭/
  exit 请求），吻合外方 MCP 客户端误触（端口共址下打错应用）。
- 并发会话（auto-os 工作区）活动时间线与死亡窗口高度共存：12:24 起 ui_desktop
  占 9247 → 我方 14:07 清理 → 14:08 其重启（9532 显式端口）→ 14:14/14:20/14:37
  我方三轮死亡 → 14:39 其再起 ui_desktop **抢占 9247**。动机闭环：其工具链需要
  9247，占用者成为清理靶子。

## 证据文件（本目录）

- `soak-summary-run1-9247.json` / `soak-summary-run2-9741-isolated.json`：两臂
  逐轮 exitCode/verdict/auditLines（run1 r2 的 soak verdict「alive」为 harness
  旧判读缺陷——vm-first-run 对子进程干净自退上报进程码 0；已改按 summary 行
  `alive=` 判读，scripts/vm-first-run-soak.mjs 同批修正）。
- `hangwatch-run1.log` / `hangwatch-run2.log`：procdump -h 逐 PID 挂接与退出
  时序（"Dump count not reached" = 监听至进程终止、全程无窗口挂起）。
- `exit-audit-run1-r2-mainreturn.log`：run1 r2 唯一审计行（逐字）；
  run1 r1/r3 审计文件为零字节（缺席即证据，无文件）。

## 环境备注

- run1 启动前清理了两个遗留进程：ui_desktop pid 42944（12:24 起，占 9247）与
  孤儿 auto.exe pid 48072（主检出 debug 树，13:54 起，父已亡）——均为测试残留。
- run1/2 全程同步存在并发会话（auto-os）的 ui_desktop 与截图探测活动。
- procdump64 v12.01（`D:/autostack/tools/`，仓外）挂接工具：
  `scripts/vm-hangwatch.mjs`（随本批入库）。

## 对 T-02 的指向（已落地，auto-lang `8d03dc1a8`）

1. **T-02a 快照锁持有面收紧**（结构面孔根修）：`SharedState.styled_vtree` 改
   Arc 发布；tool_snapshot 与 autoui_wait 扫描深序列化移出锁外。
2. **T-02b 端口绑定回退链**（干扰面加固）：同端口重试后跨 `9247..+10` 逐档
   探测并播报实际端口；共址双方皆可存活，拔除「清理靶子」动机。
3. **日志限频候选**：本次取证无日志洪水证据，不动（裁定记录）。
