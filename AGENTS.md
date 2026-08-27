# AGENTS.md — 工作约定（对所有 agent 生效）

## Git worktree 规则（强制）

凡是会修改某个项目仓库的工作，一律在专用 git worktree 内进行，**不允许直接改
该项目的主检出（默认分支）**。

### 位置

统一放在所在项目仓库根目录下的 `.worktrees/` 子目录中（已在 `.gitignore` 忽略），
便于集中管理。不要散落在项目根、系统临时目录或其他位置。

### 命名

| 场景 | worktree / 分支名 | 示例 |
|:---|:---|:---|
| 本项目、有对应 plan | `plan-<NNN>-dev` | `plan-043-dev` |
| 本项目、无对应 plan | `<项目名>-dev-<n>`（n 从 1 起找空位递增） | `auto-down-dev-1` |
| 因本项目而改的外部依赖项目 | 在依赖项目里开，用本项目命名：`<本项目名>-dev` | 改 auto-lang 时：`auto-lang/.worktrees/auto-musk-dev` |

分支名与 worktree 目录同名。本仓库 `/auto-plan:*` 流程走第一行命名；其余场景
按第二行。第三行适用于任何"顺带要改依赖库"的任务——依赖项目内部的改动只发生
在它自己的 worktree 里。

### 收尾（合回 + 清理）

任务或计划完成后：

```bash
git merge <dev-branch>            # 把开发分支合回该项目主分支
git worktree remove .worktrees/<name>
git branch -d <name>              # 删掉对应的开发分支
```

- 依赖项目的 worktree 不等整体收尾——一旦本项目消费了改动（集成验证/锁文件更新
  通过），就尽快合回该依赖项目的主分支并清理，不留悬挂 worktree。
- 合回前 worktree 必须干净；有未提交改动时先向用户确认，不要静默丢弃。
- 离开 worktree 前，确认主分支上是已知良好的状态再继续后续工作。

### 与 plan 流程的分工

计划文档是共享流程状态，不走 worktree：`/auto-plan:new` 的草稿与各阶段的状态
标记（`[✅]`、frontmatter）都写在主检出的 `docs/plans/` 上。只有代码/产品改动
进 worktree。四阶段职责拆分见 `/auto-plan:{new,work,review,merge}` 技能说明：
work 建 `plan-<NNN>-dev` 并在其中干活 → review 在其中复验 → merge 先把分支
合回 main 并删掉 worktree + 分支，再做沉淀与归档。
