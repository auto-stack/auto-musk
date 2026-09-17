# VM 进程稳定性（vm-process-stability）

> PLAN-066 T-01/T-02/T-03 交付的常驻知识：VM UI 进程退出/挂起的审计消费口径、
> 定罪决策记录、以及作为可复跑验证资产的长跑 harness 与进程普查工具。

## 退出审计消费口径（PLAN-575 三挂点）

`AUTO_DESKTOP_EXIT_LOG` 环境变量指定审计文件（空串视为未设；缺省
`%LOCALAPPDATA%/auto-desktop/exit-audit.log`）。三挂点：

1. `vm_process_exit`——Auto 层 `Process.exit`（VM shim 前落笔）；
2. `panic`——全局 panic hook（code=101，携带 msg+位置）；
3. `main_return`——iced 事件循环正常返回（关窗/exit 请求/电源键确认退出）。

**覆盖缺口（by design）**：原生层 `std::process::exit`/`TerminateProcess`
不经任何挂点 → 审计零记录 + 非 0 退出码 = 外部终止实锤（PowerShell
`Stop-Process -Force` 退出码恰为 -1）。判读矩阵见
`scripts/vm-first-run-soak.mjs`（alive/product-defect/external-kill-or-apphang/
harness-red/infra）。

## KD-048a 定罪翻案（2026-09-15，PLAN-066 T-01）

慢性「~4-5min 静默退出」**不是产品缺陷**：A/B 对照（单变量
`AUTOUI_MCP_PORT`）——9247 标准端口与并发会话共址 0/3 存活（-1 静默死×2+
main_return 自退×1）vs 私有端口隔离 3/3 存活；procdump `-h` 六轮全程零挂起
检出、死亡窗口 WER 零事件。**定罪=同机并发 agent 会话争抢标准 AutoUI MCP
端口的清理式干扰**。运行守则：稳定性 soak 先查同机并发会话，并以
`AUTOUI_MCP_PORT` 私有端口隔离。

P625-D1 AppHang 结构面孔（tool_snapshot 持锁深拷贝+全树序列化顶停 UI 发帧）
独立根修：`SharedState.styled_vtree` 改 Arc 发布，深序列化全部移出锁外
（8d03dc1a8）。

## 端口绑定回退链

AutoUI MCP server 绑定：同端口 3 次重试（Plan 065）后跨
`AUTOUI_MCP_PORT..+10` 逐档探测，回退时播报实际端口；全链失败 FATAL 行附
行动指引。共址双方皆可存活，拔除「清理靶子」动机。

## outproc 子进程收割（KD-062 清偿）

桌面 outproc 进程模型（Plan 508）`spawn_outproc_child` re-exec 的子 auto.exe
登记于 `DesktopSession.outproc_children`——生产路径曾零 kill/wait（Rust
`Child` Drop 不杀），会话结束即滞留（~66MB 档）＝KD-062「子进程不退」真身；
~43MB 瞬态=broker attach 失败自退同族。根修=`DesktopSession::drop` 统一
收割（drain+kill+wait，f11cd5df1）。回归锁：`session_drop_reaps_outproc_children`。

## 常驻验证资产

- `scripts/vm-first-run.mjs` / `.cmd`——首跑启动器（observe 后 taskkill，
  summary 行 alive/reds 判读）。
- `scripts/vm-first-run-soak.mjs`——N 轮×observe 长跑（verdict 以 firstrun
  summary 行 `alive=` 判定；注入逐轮 `AUTO_DESKTOP_EXIT_LOG`）。
- `scripts/vm-hangwatch.mjs`——procdump `-h` 挂起期转储伴跑（PROCDUMP env
  可覆写仓外工具路径）。
- `scripts/vm-mcp-census.mjs`——MCP 会话前后 auto.exe 全量普查（滞留新增
  exit 1）。

证据归档：`docs/plans/attachments/066-kd048a-conviction/`（定罪报告+A/B
soak summary+hangwatch 时序+审计行样本）。
