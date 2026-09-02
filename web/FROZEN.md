# web/ Track — FROZEN (2026-08-27, 永久冻结自 2026-09-01)

> **PLAN-041 T14/T15**: web/ 手写前端轨冻结。gen(Auto/vue)轨为生产真源。

## What changed

- **生产前端**: `server.rs` serve `gen/front/vue/dist`（不再是 `web/dist`）
- **回滚开关**: `MUSK_WEB_DIST=web/dist musk serve`（应急回滚指针,永久保留）
- **日常开发**: `start-musk-web.cmd` 已改启 gen dev :3334（T12,2026-08-27）

## Freeze terms

- 观察期 2026-08-27→09-01 零 P0/零回滚——**2026-09-01 提前收口转永久冻结**
  （PLAN-041 归档裁定;原时间门 2026-09-03,用户裁定提前收口）
- 永久冻结:web/ 完全停更,仅存历史参考 + 对拍基线
  （`git rm -r web/` 归档时机由用户另行裁定）
- `web/` 内的 vitest 套件已迁 `gen/front/vue/src/__tests__/`（T13）


## 冻结豁免记录

- **2026-09-02 PLAN-055 ⑤（用户明示豁免）**：`src/views/ChatsView.vue` 外科手术
  删除"⑂ 重试"按钮三处——模板按钮（原 :190-196）、`retryFrom` 函数（原
  :858-872）、`.retry-btn` 样式三条（原 :3018-3034）。范围仅限重试钮；
  :383-386 的 Regenerate 保留。单源（gen 轨）同日同步删除，对拍一致性目的。

## Status

| Item | Status |
|------|--------|
| 组件对拍 | 30/30 全等（2026-08-29 复审实跑;N1-N19 归一化） |
| 生产切换 | T11 已落地（gen dist + env 回滚）;观察期零回滚 |
| vitest | 23 passed + 1 skipped（i18n/frontmatter 迁毕） |
| deps-guard | 通过（gen 轨严格白名单;web 域 frozen 标注） |

## References

- [PLAN-041](../docs/plans/archived/041-web-track-retirement.md)
- [KNOWN-DEBT](../docs/plans/KNOWN-DEBT-AND-RISKS.md)
