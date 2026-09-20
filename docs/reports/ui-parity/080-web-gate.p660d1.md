# PLAN-080 web 门收据（F-R2 修复轮，P660-D1 收口后全 pipeline）

- 日期：2026-09-21
- 被验提交：auto-musk `plan-080-dev` @ `9b0e857`（F-R1 check 门修复）；
  auto-lang `auto-musk-dev` @ `7d6888076`（P660-D1 收口，release 二进制重建自该提交）
- 门命令：`auto build`（musk worktree 根）＝ codegen + scaffold + `pnpm run build`（**vue-tsc && vite build**）

## 修前状态（2026-09-20 复审定罪，见 PLAN-080 §9 r2 F-R2）

干净再生的 `gen/front/vue` 上 vue-tsc 双红：

```
src/auto-select/overlay.ts(6,30): error TS2307: Cannot find module '../auto-sources' or its corresponding type declarations.
src/main.ts(32,17): error TS2339: Property 'env' does not exist on type 'ImportMeta'.
[ELIFECYCLE] Command failed with exit code 2.
```

- TS2307 根因：`auto-sources.ts` 仅 `auto run`/incremental 路径发射，`auto build` 不发射，而 build 产出的 overlay.ts 无条件导入它。
- TS2339 根因：P660-D1（auto-lang KNOWN-DEBT-AND-RISKS 预存债，444b87fec 已在 080 基线内，非本计划回归）。
- 绕行验证：`pnpm exec vite build` 直跑绿（14.31s）——vite 本体不查类型。

## 修后状态（P660-D1 收口，auto-lang 7d6888076）

scaffold 三处发射 `src/vite-env.d.ts`（create-vue 惯例 `/// <reference types="vite/client" />`，
不设 tsconfig types 以免窄化 @types 自动包含）+ `prepare_vue_sources` 尾部补发射
`auto-sources.ts` + run 路径缺文件自愈。

```
$ vue-tsc && vite build
✓ built in 14.70s
═════════════════════════════════
  Vue project built successfully!
═════════════════════════════════
```

- musk `auto build`：**全 pipeline 绿**（codegen → scaffold → install → vue-tsc → vite）。
- examples/ui/014-weather `auto build`（P660-D1 原始复现点）：**全 pipeline 绿**（built 1.59s）。
- 发射物实证：`gen/front/vue/src/vite-env.d.ts`（38B）+ `gen/front/vue/src/auto-sources.ts`（773KB dev-only 源映射）。
- auto-lang `cargo tf`：3649/3652 绿，3 败=已定罪前置红（mouse_area / autodown_panel_heading /
  shell_pack_vocabulary），零新增。

## 基线有效性

scaffold 变更为纯类型面（vite-env.d.ts）与 dev-only 模块（auto-sources.ts，产物 tree-shake），
不改 Vue 运行时渲染输出与 VM 面——既有 078/079 物化基线与 080 live 收据保持有效。

## 关联

- PLAN-080 §9 r2 复审 F-R1/F-R2/F-R4；T-01/T-06 复审重开收窄项本轮收口。
- ui-parity check 门：离线 PASS（live 3/4 ok，settings-rows 红=T-05 字体度量族在途的正确红）；
  `check --live` 硬红语义经 F-R1（musk 9b0e857）兑现。
- F-R4：PLAN-078 收据目录 2 份已被 079 目录补跑取代的陈旧红收据已清理（2026-09-21）。
