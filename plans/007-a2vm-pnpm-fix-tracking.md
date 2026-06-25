# 001 — a2vm pnpm fix PR review/合并(E,优先级最低)

> **状态**: ✅ **已解决（2026-06-26 核实）** —— auto-lang master 已以**另案方案**修复并合并,本跟踪文件可关闭。
> **仓库**:auto-lang(`gitee.com:auto-stack/auto-lang`)。
> **分支**:`fix/vue-pnpm-workspace-single-package`(本仓原修复分支,本地已不可见)。
> **优先级**:5️⃣ 低 —— 不阻塞任何东西,挂着等 review。

## 已完成的修复

`ensure_pnpm_build_approvals()`(auto-man `vue.rs`)原来往 `output_dir`(Vue 包根目录)写 `pnpm-workspace.yaml`,但只含 build-approval 键、没有 `packages:` 字段。对单包项目,pnpm 把它当 workspace 根并报 "packages field missing or empty",install 失败 —— 这**阻塞了所有生成的 Vue 项目**(auto-musk、015-notes 等),且级联导致 shadcn 组件(select 等)没装上。

**修复**:`ensure_pnpm_build_approvals` 改为 no-op(build approvals 已在 `package.json` 的 `pnpm.onlyBuiltDependencies` 字段,workspace 文件冗余且有害)。

**验证**:修复后 `auto build` 在 auto-musk 上完整跑通(install + shadcn 组件自动装 + vite build)。

## ✅ 解决情况（2026-06-26 核实）

auto-lang 团队**未采用本计划提议的 no-op 方案,而是以另一套方案解决并合并到 master**。证据（auto-lang master 提交）:
- `dc0886c8 fix(auto-man): pnpm v11 build approvals — allowBuilds map + workspace root check`
- `51280936 fix(auto-man): pnpm build approvals via .npmrc, not workspace yaml`
- `fa6c93fa fix(pnpm): disable verify-deps-before-run to stop vite startup crash`
- `0eeae905 fix(pnpm): remove deprecated pnpm field from package.json + clean yaml`

`ensure_pnpm_build_approvals` 仍在 `crates/auto-man/src/vue.rs:557`(采用 allowBuilds map + workspace root check,而非本计划的 no-op)。**根因问题（pnpm install 失败阻塞生成的 Vue 项目）已消除**,目标达成。

- [x] review 分支 `fix/vue-pnpm-workspace-single-package`(auto-lang 另案处理)
- [x] 合并到 auto-lang master(以 `dc0886c8` 等系列提交)
- [ ] (可选)防回归单测——auto-lang master 现方案是否已有,未核实

## 影响

合并后,任何人 clone auto-musk + `auto build` 就能得到完整可运行前端(api.ts + 所有组件 + 可 build),不再需要手工 `npm install` 或复制 select 组件。

## 不在本计划范围
- a2vm 的其他改进(Skill/Flow 支持、`on` 块的异步处理等)—— 那些是 a2vm 功能扩展,与这个 install 修复无关。
