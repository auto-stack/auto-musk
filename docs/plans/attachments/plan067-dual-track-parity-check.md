# PLAN-067 T-06 — 双轨对齐检查登记（VM 轨 / web 回退轨）

> 检查基线：PLAN-067 r2（2026-09-09），种子清单见 PLAN-067 §4.3/§4.4。
> 结论口径：✅ 无差异 / 🔄 行为差异（登记）/ ⚠ 待运行时复核。

## 种子清单复检（本会话 web 轨五项修复）

| # | 检查项 | web gen 轨（已修） | VM 轨（iced） | web/ 回退轨（冻结） |
|---|---|---|---|---|
| 1 | IME 组合串深色可见 | ✅ f0c77c9（组词期实绘前景色） | ✅ 无双层技术——iced text_editor 原生显字+原生 preedit，无此缺陷面 | ✅ 无双层技术——web/ ChatsView textarea 直接 `color: var(--af-fg)` 显字，backdrop 只画 mention 药丸（其自身文字 transparent 仅影响药丸内文字，不影响输入） |
| 2 | Ctrl+A 选区可见 | ✅ 00dee19（显式 ::selection 主题配色；仅 gen 轨 textarea.chats-input） | ✅ iced 原生 selection 渲染，无透明叠层 | ✅ 同上，原生路径；未加同款 ::selection（观感差异登记，冻结轨不修） |
| 3 | 输入链路 TypeError（mention_detect 契约） | ✅ 07b90a5（串入参回退 activeElement） | N/A——mention_helpers 为 use.web，VM 不编译该链；VM 轨 mentions 走原生 Highlighter（PLAN-493） | web/ 轨自带旧版 mention 检测（读事件对象，契约匹配其自身实现），无此缺陷 |
| 4 | 删除确认弹窗全件渲染 | ✅ 2ff3983（补 AlertDialogDescription 导入；gen 轨组件） | ✅ alert-dialog 为 VM 受控 open 模态（PLAN-058/059 T9 实机复验成立），无 web 组件导入问题 | web/ 轨 GateCard 独立实现，无 alert-dialog 依赖 |
| 5 | aaid 在线（对话可用） | 运维项：start-all.cmd 先启 aaid 再启 musk；musk 启动横幅有 unreachable 警告可判 | 同（daemon 与轨道无关） | 同 |

## PLAN-067 新增行为对齐

| 行为 | web gen 轨 | VM 轨 | web/ 回退轨 |
|---|---|---|---|
| 流式实时刷新（T-02） | ✅ SSE 主通道 + 叶链投影对齐 | 🔄 轮询为主通道；T-02 的 PollStream 叶同步（active_leaf 随回填刷新）预计**同时修复 VM 轨"回复了但界面不动"残留（KD 059-FU1 族）**——⚠ 待 VM 实机复核；SSE 健康门在 VM 恒开（OnStreamEvent 不触发，last_sse_at=0），轮询语义不变 | 🔄 web/ 轨自管流式（旧 forge_stream 形态），本修复不含；冻结轨差异登记 |
| gate 卡实时可见（T-04a） | ✅ relay_gate_waiting 镜像+桥接放行 | ⚠ VM 无 SSE，Run 卡门状态依赖轮询回填节奏——与 web 轨存在呈现延迟差异（登记）；门语义（暂停等决议）双轨一致（同一 store） | ⚠ 同左（冻结轨） |
| 审批模式（T-05） | ✅ composer 下拉 + PATCH 持久化 + driver 自动放行 | 🔄 字段/上下文/store 双轨一致（ChatSession.approval_mode ag 版同步）；**iced composer 无模式下拉（v1 web-only）**——可通过 API PATCH 设置，UI 选择器登记为 VM 待办 | web/ 轨无 relay spawn UI（冻结），N/A |
| gate 三态语义（T-04c） | ✅ 单测钉死（等待无超时/approve 续跑/reject=重做非中止） | ✅ 同一 store 语义，天然一致 | ✅ 同上 |

## 登记结论

- 冻结 web/ 轨：无需修复（无双层技术面），登记"观感差异：无自定义选区配色"。
- VM 轨：①PollStream 叶同步预计顺带修复 KD 059-FU1 残留（待实机复核后可关 KD）；②审批模式 UI 选择器缺位（API 可设）——两项均已登记，修复归 VM 轨后续计划，不在本计划内实施。
