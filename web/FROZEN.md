# web/ Track — FROZEN (2026-08-27)

> **PLAN-041 T14**: web/ 手写前端轨冻结。gen(Auto/vue)轨为生产真源。

## What changed

- **生产前端**: `server.rs` serve `gen/front/vue/dist`（不再是 `web/dist`）
- **回滚开关**: `MUSK_WEB_DIST=web/dist musk serve`（观察期内有效）
- **日常开发**: `start-musk-web.cmd` 待 T12 更新指 gen dev server

## Freeze terms

- **只收 P0 bugfix** 到观察期结束（默认 7 天）
- 观察期后完全停更（`git rm -r web/` 归档时机由用户裁定）
- `web/` 内的 vitest 套件已迁 `gen/front/vue/src/__tests__/`（T13）

## Status

| Item | Status |
|------|--------|
| 组件对拍 | 16/17 全等（N1-N12 白名单，actions styling 差异已知） |
| 生产切换 | T11 已落地（gen dist + env 回滚） |
| vitest | 23 passed + 1 skipped（i18n/frontmatter 迁毕） |
| deps-guard | 通过（gen 轨严格白名单） |

## References

- [PLAN-041](../docs/plans/041-web-track-retirement.md)
- [KNOWN-DEBT](../docs/plans/KNOWN-DEBT-AND-RISKS.md)
