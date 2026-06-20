# 006 — 更多技能移植(D,优先级中低)

> **状态**:设计 + 实施计划。
> **仓库**:auto-musk(`skills/` + 用户目录 `~/.config/autoos/skills/`)。
> **优先级**:4️⃣ 中低 —— 三步曲(最核心)已有,这些是"锦上添花"。

## 目标

从 superpowers 移植**对编程 agent 最有价值的几个技能**,扩充 musk 的技能库。当前有 3 个(brainstorming/writing-plans/executing-plans),补到 ~8 个。

## 候选技能(按价值排序,从中选 4-5 个)

### 高价值(建议都做)

1. **`test-driven-development`** —— 写测试先于实现。executing-plans 的天然补充(每个 task 的"step 1 常是失败测试")。
2. **`systematic-debugging`** —— 遇 bug 时系统化排查(复现→定位→假设→验证),而非瞎改。agent 调试能力的核心。
3. **`requesting-code-review`** —— 完成后请求 review(自检 checklist)。对应三步曲的 review 阶段。
4. **`verification-before-completion`** —— 声称"完成"前必须跑验证、拿证据。防止 agent 谎报完成。

### 中价值(选做)

5. **`using-git-worktrees`** —— 隔离工作区。但 musk 是 CLI agent,git worktree 价值不如 IDE agent,可选。
6. **`finishing-a-development-branch`** —— 分支收尾(merge/PR/cleanup)。依赖 git worktree,配套做。
7. **`skill-creator`** —— 帮用户创建新技能(元技能)。让技能库可扩展。

### 低价值/排除

- `subagent-driven-development` —— 依赖子 agent 调度(Forge 的 dispatch),musk 无此能力,跳过。
- `dispatching-parallel-agents` —— 同上,依赖子 agent。
- `writing-skills`(superpowers 的元技能)—— 我们已有 skill-creator 候选,二选一。

## 建议首批(4 个)

`test-driven-development` + `systematic-debugging` + `requesting-code-review` + `verification-before-completion`。这 4 个直接强化三步曲的 execute/review 阶段。

## 改编原则(每个技能)

- **保留核心方法论**,删 superpowers 里的平台特定内容(Claude Code / Copilot / Gemini 的工具名差异)→ 统一用 musk 的工具名(`edit_file`/`search`/`run_command`/`skill`)。
- **frontmatter 只留 name + description**(description 是触发器,只说"何时用",不剧透流程 —— 这是 superpowers writing-skills 的明确要求)。
- **每个技能一个目录 `skills/<name>/SKILL.md`**,简洁(<200 行)。
- **交叉引用用名字**(`REQUIRED SUB-SKILL: writing-plans`),不用会强制加载的语法。

## Tasks

### Task 1-4: 各写一个技能 SKILL.md
每个:
- [ ] 读 superpowers 原文,提取核心方法论
- [ ] 改编为 musk 上下文(工具名、流程适配 CLI agent)
- [ ] 写 `skills/<name>/SKILL.md` + 装到 `~/.config/autoos/skills/<name>/`
- [ ] commit

### Task 5: 验证 + 提交
- [ ] `musk chat` 确认新技能出现在 `<available_skills>` 且模型能调用
- [ ] push

## 验收
- 技能库从 3 → 7(+4 个核心)。
- 每个技能模型可自主调用(`musk chat` 触发对应场景)。

## 注意
- 技能多了 `<available_skills>` 块会变长 —— 控制每个 description 简短(<1024 字符,superpowers 规范),且数量不超过 ~10 个以免稀释模型注意力。
