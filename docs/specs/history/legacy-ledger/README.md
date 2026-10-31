# PLAN-106 旧 Spec 账本迁移对账

原账本 SHA256：`4d73a7aadbf4e3a2c83942d024f0921e5099763b94af4172e9430463f9561367`；37 条记录（18 designs / 1 tests / 9 reviews / 9 reports）及 envelope 全字段保全。原 JSON 未改，dry-run、apply、重复 apply 成功。

导入内容仅为历史。当前知识仍在模块规范；不因保全记录而重新批准旧状态、配额或交付流程。备份与 journal 仅保留本地。

| 旧条目 | 当前知识/历史去向 | 保全记录 |
|---|---|---|
| designs/PLAN-096-plan-execution-contract | [plan-execution-contract](../../modules/plan-execution-contract.md) | [原记录](4d73a7aadbf4e3a2/2-0.md) |
| designs/PLAN-096-plan-flow | [plan-flow](../../modules/plan-flow.md) | [原记录](4d73a7aadbf4e3a2/2-1.md) |
| designs/PLAN-096-specs-ledger | [specs-ledger](../../modules/specs-ledger.md)（中央写入/配额条款由 PLAN-106 明确退休） | [原记录](4d73a7aadbf4e3a2/2-2.md) |
| designs/PLAN-096-worktree-scope | [workspace-sandbox](../../modules/workspace-sandbox.md) | [原记录](4d73a7aadbf4e3a2/2-3.md) |
| designs/PLAN-098-run-telemetry | [run-telemetry](../../modules/run-telemetry.md) | [原记录](4d73a7aadbf4e3a2/2-4.md) |
| designs/PLAN-099-canvas-embed | [canvas-embed](../../modules/canvas-embed.md) | [原记录](4d73a7aadbf4e3a2/2-5.md) |
| designs/PLAN-099-chat-run-liveness | [chat-run-policy](../../modules/chat-run-policy.md) | [原记录](4d73a7aadbf4e3a2/2-6.md) |
| designs/PLAN-100-plan-flow-nudge | [plan-flow](../../modules/plan-flow.md) | [原记录](4d73a7aadbf4e3a2/2-7.md) |
| designs/PLAN-100-relay-runbox | [chat-streaming](../../modules/chat-streaming.md) | [原记录](4d73a7aadbf4e3a2/2-8.md) |
| designs/PLAN-101-vm-composer-send | [app-studio-ui](../../modules/app-studio-ui.md)、[workspace-ui](../../modules/workspace-ui.md) | [原记录](4d73a7aadbf4e3a2/2-9.md) |
| designs/PLAN-103-chat-gate-visibility | [workspace-sandbox](../../modules/workspace-sandbox.md)、[chat-run-policy](../../modules/chat-run-policy.md)、[ui-parity](../../modules/ui-parity.md) | [原记录](4d73a7aadbf4e3a2/2-10.md) |
| designs/PLAN-102-canvas-embed-workbench | [canvas-embed](../../modules/canvas-embed.md)、[app-canvas](../../modules/app-canvas.md)、[app-studio-ui](../../modules/app-studio-ui.md)、[autoui-canvas-runtime](../../modules/autoui-canvas-runtime.md) | [原记录](4d73a7aadbf4e3a2/2-11.md) |
| designs/PLAN-105-run-lifecycle-and-tool-layer | [chat-run-policy](../../modules/chat-run-policy.md)、[workspace-sandbox](../../modules/workspace-sandbox.md) | [原记录](4d73a7aadbf4e3a2/2-12.md) |
| designs/PLAN-104-vm-process-stability | [vm-process-stability](../../modules/vm-process-stability.md) | [原记录](4d73a7aadbf4e3a2/2-13.md) |
| designs/PLAN-097-demo-evidence | [demo-evidence](../../modules/demo-evidence.md) | [原记录](4d73a7aadbf4e3a2/2-14.md) |
| designs/PLAN-097-canvas-progress-vm | [app-studio-ui](../../modules/app-studio-ui.md) | [原记录](4d73a7aadbf4e3a2/2-15.md) |
| designs/PLAN-097-plan-progress-card | [workspace-ui](../../modules/workspace-ui.md) | [原记录](4d73a7aadbf4e3a2/2-16.md) |
| designs/PLAN-097-ui-parity-consumable | [ui-parity](../../modules/ui-parity.md) | [原记录](4d73a7aadbf4e3a2/2-17.md) |
| tests/PLAN-097-demo-suite | [demo-evidence](../../modules/demo-evidence.md)；测试入口/历史原文保全 | [原记录](4d73a7aadbf4e3a2/3-0.md) |
| reviews/PLAN-098-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-0.md) |
| reviews/PLAN-099-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-1.md) |
| reviews/PLAN-100-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-2.md) |
| reviews/PLAN-101-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-3.md) |
| reviews/PLAN-103-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-4.md) |
| reviews/PLAN-102-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-5.md) |
| reviews/PLAN-105-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-6.md) |
| reviews/PLAN-104-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-7.md) |
| reviews/PLAN-097-review | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/4-8.md) |
| reports/PLAN-096-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-0.md) |
| reports/PLAN-099-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-1.md) |
| reports/PLAN-100-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-2.md) |
| reports/PLAN-101-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-3.md) |
| reports/PLAN-103-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-4.md) |
| reports/PLAN-102-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-5.md) |
| reports/PLAN-105-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-6.md) |
| reports/PLAN-104-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-7.md) |
| reports/PLAN-097-delivery | 既有归档 Plan 及本行历史保全；不合成为新的当前要求 | [原记录](4d73a7aadbf4e3a2/5-8.md) |

18 条设计摘要逐项核对：当前文档承载其长期合同；关于旧五检查点、账本三通道/配额的说明仅为历史，其退休规则见 PLAN-106 新规范。所有未知/兼容字段留在每条 Original record，区级/顶层元数据留在 envelope。

[Envelope](4d73a7aadbf4e3a2/envelope.md)。本地 journal：D:/autostack/plan-artifacts/plan-106/migration-manifest.json；备份同目录 migration-manifest.legacy-backup。

<!-- PLAN-107 migration reconciliation -->

## PLAN-107：auto-musk-backend

旧源：`auto-musk/backend/.autoos/specs.json`。

原字节 SHA256：`9cf74525a18f6985d384bc5121db7479c7455e4ecb9ed740b7dc5843fa75261d`；本批 245 条 + 1 envelope，全部原记录可从 AutoDown Markdown JSON 围栏回读。原 JSON 不改写、不删除。

提交归属：auto-musk PLAN-107；执行基线 `3b291c91d9c1668396b192665245494fc7d2a28d`。最终迁移提交和默认分支落地映射记录在该计划 §9；本报告与保全文档由同一仓库迁移提交管理。

本批核对每条记录的来源、归属、当前/历史边界及保全内容；现有 canonical 文档优先，历史执行数字不重新认证，也不将旧状态改成 approved。无正文占位和外部历史收据有显式处理结果。

- [auto-musk-backend section 0：38 条](plan-107-auto-musk-backend-section-0.md)
- [auto-musk-backend section 1：36 条](plan-107-auto-musk-backend-section-1.md)
- [auto-musk-backend section 2：35 条](plan-107-auto-musk-backend-section-2.md)
- [auto-musk-backend section 3：36 条](plan-107-auto-musk-backend-section-3.md)
- [auto-musk-backend section 4：61 条](plan-107-auto-musk-backend-section-4.md)
- [auto-musk-backend section 5：39 条](plan-107-auto-musk-backend-section-5.md)

### 恢复与回滚

未合入时使用同一 manifest 的 rollback，仅删除迁移自有且 after hash 未变的导入文件；修改过的文件明确拒绝。报告、规范提炼和规则变更在完整 Git 写集合中审阅。合入后从专用回滚工作树对本批提交作新的 `git revert`，无关后续修改保留，重叠修改先解决冲突；不 reset 默认分支。批次状态按项目记录，局部失败不伪称跨仓原子成功。

原字节备份/journal/逐文件指纹在本地批次目录，遵守当前证据保留规则；长期回滚依据各仓 Git 提交、本文及原值保全文档，不能仅依赖临时附件。
