# AGENTS.md — 工作约定（对所有 agent 生效）

## Git worktree 规则（强制）

凡是会修改某个项目仓库的工作，一律在专用 git worktree 内进行，**不允许直接改
该项目的主检出（默认分支）**。

### 位置

统一放在按计划分组的平铺目录 **`D:\autostack\.wt\<组名>\<项目名>\`**（Plan 529 布局；
`.wt` 根不受任何仓库管理）。组名规则 `musk-<NNN>`（本项目 plan）或
`<任务名>`（无 plan 场景）。示例：`.wt/musk-060/auto-musk`。不要放项目根、
系统临时目录或项目内部（旧 `.worktrees/` 已因 2026-09-03 删除事故退役，仅剩
在途计划就地跑完）。

**红线**：worktree 内**禁止创建任何 junction/symlink**——`git worktree remove`
的递归删除会穿透链接删掉目标仓内容（事故实测复现）。跨仓依赖按解析序：
`$AUTO_LANG_ROOT/$AUTO_AI_ROOT 等 env 覆盖 → 组内 ../auto-lang 等兄弟 →
D:/autostack/<repo> 主检出`。

### 命名

| 场景 | worktree 组 / 分支名 | 示例 |
|:---|:---|:---|
| 本项目、有对应 plan | 组 `musk-<NNN>`，分支 `plan-<NNN>-dev` | `.wt/musk-060/auto-musk` |
| 本项目、无对应 plan | 组 `<任务名>`，分支 `<项目名>-dev-<n>` | `.wt/vendor-bump/auto-musk` |
| 因本项目而改的外部依赖项目 | 同组并排开依赖项目 worktree，分支 `<本项目名>-dev` | `.wt/musk-060/auto-lang`（分支 `auto-musk-dev`） |

分支名与 worktree 目录名解耦（目录名恒为项目名）。本仓库 `/auto-plan:*` 流程走
第一行命名；其余场景按第二行。第三行适用于任何"顺带要改依赖库"的任务——同组
并排使 `../auto-lang` 相对路径直接成立，**不再用 junction**。

### 收尾（合回 + 清理）

任务或计划完成后：

```bash
bash D:/autostack/wt-guard.sh D:/autostack/.wt/<组>/<项目>   # 必须输出 clean 才继续
git merge <dev-branch>            # 把开发分支合回该项目主分支
git worktree remove D:/autostack/.wt/<组>/<项目>
git branch -d <name>              # 删掉对应的开发分支
# 组内已无兄弟 worktree 时删除组目录：rmdir D:/autostack/.wt/<组>
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
