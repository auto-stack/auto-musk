独立复审已过（outcome: pass，next: merge）。请用 /auto-plan:merge 技能沉淀 PLAN-001：

1. 把 plan-001-dev（b864746）按快进方式合回本 demo 仓库 main（--ff-only，不产生 merge commit）。
2. 归档计划文件到 docs/plans/archived/，并把 docs/ 下的产物（plans 归档件、specs）全部纳入版本控制提交到 main。
3. 刷新工作区 Spec Ledger（.autoos/specs.json）：SD-01 → docs/specs/password-generator.md 的派生索引与历史。
4. 清理 D:/autostack/.wt/demo-plan-001 的 worktree 与 plan-001-dev 分支，给出四条收据（landed / ledger_refreshed / archived / cleaned）。
5. 备注：AC-02 的浏览器交互项（点击生成、滑块拖动、开关点击、复制反馈）按 D8 约定转为 merge 后用户首用验收义务，请在归档记录中保留该备注。
