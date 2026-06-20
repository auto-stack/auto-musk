# 001 — a2vm pnpm fix PR review/合并(E,优先级最低)

> **状态**:修复**已完成并推送**,待 review/合并。本文件是跟踪记录。
> **仓库**:auto-lang(`gitee.com:auto-stack/auto-lang`)。
> **分支**:`fix/vue-pnpm-workspace-single-package`(已推送)。
> **优先级**:5️⃣ 低 —— 不阻塞任何东西,挂着等 review。

## 已完成的修复

`ensure_pnpm_build_approvals()`(auto-man `vue.rs`)原来往 `output_dir`(Vue 包根目录)写 `pnpm-workspace.yaml`,但只含 build-approval 键、没有 `packages:` 字段。对单包项目,pnpm 把它当 workspace 根并报 "packages field missing or empty",install 失败 —— 这**阻塞了所有生成的 Vue 项目**(auto-musk、015-notes 等),且级联导致 shadcn 组件(select 等)没装上。

**修复**:`ensure_pnpm_build_approvals` 改为 no-op(build approvals 已在 `package.json` 的 `pnpm.onlyBuiltDependencies` 字段,workspace 文件冗余且有害)。

**验证**:修复后 `auto build` 在 auto-musk 上完整跑通(install + shadcn 组件自动装 + vite build)。

## 待办

- [ ] review 分支 `fix/vue-pnpm-workspace-single-package`
- [ ] 合并到 auto-lang master
- [ ] (可选)考虑:是否在生成器里加个单测,防回归(写 pnpm-workspace.yaml 的逻辑不再触发)

## 影响

合并后,任何人 clone auto-musk + `auto build` 就能得到完整可运行前端(api.ts + 所有组件 + 可 build),不再需要手工 `npm install` 或复制 select 组件。

## 不在本计划范围
- a2vm 的其他改进(Skill/Flow 支持、`on` 块的异步处理等)—— 那些是 a2vm 功能扩展,与这个 install 修复无关。
