---
plan_id: PLAN-065
status: archived
feature_name: local-debt-cleanup-batch
author: [zhaop]
created_at: 2026-09-07T11:00:00+08:00
updated_at: 2026-09-07T05:55:47Z

supersedes_spec_components:
  - "P042-3/P042-4（architecture/designs）: tool_result SSE 语义修订——status 恒 success 作废，改服务端单点真字段（tool_result_status 两前缀判定），前端去嗅探"
  - "P033-3/P033-4（architecture/designs）: 归档路径语义显式化——move_to_archived 裸写改调 force_transition 系统迁移面（行为零变化）"
new_spec_components:
  - "P065-1: 变更摘要（SSE status 真字段/静态服务 no-cache/TraceLayer/fixture pre→div/force_transition/六件冒烟并账） → reports"
  - "P065-2: 目标（027 债闭环/部署缓存债闭环/服务端可观测/033 W1 显式建模/六件冒烟清账） → goals"
  - "P065-3: 架构方案（错误标记唯一产源单点检测/SetResponseHeaderLayer/TraceLayer 外层挂装/force_transition 系统迁移旁路面） → architecture"
  - "P065-4: 详细设计（tool_result_status 契约注释钉死/持久化同源取值/ServiceBuilder 包 ServeDir 链/pre→div+pre-wrap/force_transition 不移位） → designs"
  - "P065-5: 测试设计（status 真字段四断言/620 全量/gen 36+1skip/curl 双实证/冒烟六格） → tests"
  - "P065-6: 验收标准（七项全过，见复审记录） → reviews"
  - "P065-7: 复审记录（2026-09-07 复验通过，含 F1-F6 新发现登债与 merge 折叠注意） → reviews"
touched_goals:
  - "goal-agent: SSE tool_result.status 真字段化——工具事件错误语义从双轨前端嗅探改服务端单点判定，流式/回放两轨一致（持久化同源）"
  - "goal-frontend-parity: gen 轨去嗅探仅消费 ev.status + specs_detail fixture pre→div（pre-wrap）对齐已验行为；六件历史冒烟并账执行，F1-F6 gen/web 缺口全量登债在案"
  - "goal-spec-knowledge: plans 归档状态机显式化（force_transition）+ musk serve 可观测（TraceLayer 请求日志）与部署缓存债闭环（静态服务 no-cache）——auto-plan 流程与规范服务工程面收口"

current_step: 10
total_steps: 10
---

# [PLAN-065] musk 本地收尾小批——DEBT 清偿（SSE status 真字段/缓存/日志/pre 缩进/归档状态机）+ 六件人工冒烟合并

## 变更摘要

集中清偿 KNOWN-DEBT 中**纯 musk 本地、无上游依赖**的敞口，并把散落六个计划的"待用户 5 分钟"人工冒烟合并成一场执行：

1. **SSE `tool_result.status` 真字段化**（027 债，2026-08-24 登记）：服务端单点检测 auto-ai 错误标记 → `status: "error"`，gen 前端去前缀嗅探。
2. **静态服务 Cache-Control**（056 部署缓存债）：dist 响应统一 `no-cache`，重新部署不再吐旧 JS。
3. **musk serve 请求日志**（048-c）：加 TraceLayer，INFO 级行（方法/路径/状态/耗时）。
4. **specs_detail fixture pre→div**（056 pre 缩进伪影，规范页批次）：套用 056 T3-T5 已实证的修复模式。
5. **plans.rs 归档系统迁移显式建模**（033 W1）：`force_transition` 独立语义面，不再裸写绕过状态机。
6. **六件人工冒烟合并执行**：033 W2 MetaBlock 渲染目验 / 040-1 流式 E2E / 042 T10 LLM 会话冒烟 / 493 @mention 着色像素终验 / 050 rail 像素目验 / 061 D29 IAB 语言切换复验。
7. **048-d（@autodown 包名漂移）复核销号**：非代码任务——已由 056 T7 的 `gen/front/vue/package.json` file: 双链接（vue + engine 0.5.0 真身 vendor）实质闭合，会话级 junction 早已退役，随复审在 DEBT 表销号并注记。

## 目标

- 027 债闭环：错误语义从"前端嗅探 result 前缀"改为"服务端真字段"，gen 轨消费 `ev.status`（`forge_store.at:457` 已预留该判定）。
- 部署缓存债闭环：浏览器强刷不再是看到新前端的必要条件。
- 服务端可观测：契约调用有请求日志（配合副作用回读调试）。
- 033 W1 闭环：归档路径的状态写入有显式系统迁移语义，未来 `transition()` 增加副作用时终态路径不会静默漏触发。
- 六件历史冒烟一次清账，全部记录在案（通过或注明残留）。

## 架构方案

全部改动落在 musk 主仓，**零 auto-lang / auto-ai 代码改动**：

- **SSE status**：错误标记的唯一产源是 auto-ai `exec_or_msg`（`auto-ai/crates/auto-ai-agent/rust/src/tool.rs:152-158`，仅两种格式：`[security denied ...]` / `[tool error: ...]`）。musk 在 `stream_event_to_json` 边界单点检测这两种前缀（契约注释钉死来源），不再让两端前端各自嗅探。事件结构不变，仅 `status` 取值真实化。
- **缓存**：tower-http `SetResponseHeaderLayer` 包静态服务（`server.rs:106-111` 的 `static_service`），index.html 与 assets 统一 `no-cache`（每次 revalidate）。产物 hash 化涉及 auto-lang vite 模板（`auto-man/src/vue.rs` entryFileNames），不在本计划。
- **日志**：tower-http `TraceLayer::new_for_http`，INFO 级 span（方法/路径/状态/耗时），挂 router 与静态服务之前。
- **归档状态机**：`PlanStore` 新增 `force_transition`（系统迁移语义，不经 `can_transition`），`move_to_archived` 改为调用它——行为零变化，语义显式化。
- **冒烟**：清单文档 + 一场执行，浏览器面（:9247 或 8580）与 VM 面（iced 实机）分开列步骤。

## 技术栈

Rust（axum + tower-http 0.6，需追加 `set-header` / `trace` feature）/ Auto（`src/front/forge_store.at`、`src/front/specs_detail.at`）/ vitest + vue-tsc（gen 轨门禁）。

## 需求分析与背景调查

specs 账本相关项：
- **P042-2**（ToolOutput 迁移——details 双通道产出 UI 价值）：details 已透传，status 真字段是同族收尾；`server.rs:451-459` 注释自认"status 真字段仍缺失"。
- **P040-2**（run_command/ToolUpdate SSE 实时进度）：SSE 事件契约的面。
- **P041-2**（gen 轨转正生产前端）：部署缓存债与请求日志都落在 gen 轨生产面。
- **P056-2**（Chat 块型样式——pre 首行缩进）：fixture 块是同一 codegen 伪影在规范页的表现。
- **P024-2 / P030-2**（auto-plan 架构/基于 Plan 的开发流程）：plans.rs 状态机所属。

代码勘察结论（2026-09-07 实测）：
- `server.rs:451` Tool 分支硬编码 `"status": "success"`；`:690` 会话回放分支按 `type == "tool_result"` 读 JSON，需复核 status 透传一致。
- `forge_store.at:455-461` 前缀嗅探两行 + 已预留 `if ev.status == "error"`；`generic_tool_card.at:35` 消费 `call.status`（store 级 failed/completed），卡片层无需改。
- `useForge.ts:263+`（web 轨）有更宽的启发式——web 已冻结（041），不动，注记即可。
- `vite.config.ts` 产物名平铺无 hash（`assets/index.js`），但该文件是生成物（auto-man 模板），musk 本地只做服务端 no-cache。
- `gen/front/vue/package.json:22-23` 已 file: 链接 vendor 双包 → 048-d 实质已闭。
- `plans.rs:514-520` `move_to_archived` 已有终态漏斗注释，缺显式建模。
- `specs_detail.at:449-452` fixture `pre { class: "fixture-code" }`，`.fixture-code`（:408）依赖 pre 语义；`.details-body`（:236）已有 `white-space: pre-wrap` 先例。
- serve 启动惯例：`musk serve --addr 127.0.0.1:9247`（`scripts/dev-stack.mjs`），CWD=tmp/musk-demo。

## 详细设计

### T1 SSE status 真字段（服务端）

`server.rs` 新增：

```rust
/// auto-ai exec_or_msg（auto-ai-agent tool.rs:152-158）是错误标记唯一产源，
/// 仅两种格式。事件契约：status = "error" | "success"（对齐 gen 轨消费）。
fn tool_result_status(result: &str) -> &'static str {
    if result.starts_with("[security denied") || result.starts_with("[tool error") {
        "error"
    } else {
        "success"
    }
}
```

Tool 分支 `"status": "success"` → `"status": tool_result_status(&result)`；删除分支内"status 真字段仍缺失"过期注释。复核 `server.rs:690` 回放分支：持久化 JSON 若存的是旧 `success`，回放侧同样经 `tool_result_status` 重判或确认存量事件已含真值——以实测为准，两轨一致即收。

### T2 gen 前端去嗅探

`forge_store.at:455-461`：删除两行 `result.starts_with(...)` 嗅探，保留 `if ev.status == "error" { failed = true }`，注释更新为"027 债闭环：status 真字段，服务端单点判定"。`useForge.ts` 不动（web 冻结轨，DEBT 注记）。

### T3 静态服务 Cache-Control

`backend/crates/musk/Cargo.toml:28`：`tower-http` features 加 `"set-header"`。`server.rs` 静态服务处：

```rust
use tower_http::set_header::SetResponseHeaderLayer;
let static_service = tower_http::services::ServeDir::new(&web_dist)
    .fallback(...)
    .layer(SetResponseHeaderLayer::overriding(
        http::header::CACHE_CONTROL,
        http::HeaderValue::from_static("no-cache"),
    ));
```

### T4 请求日志 TraceLayer

`Cargo.toml` features 追加 `"trace"`。router 装配处加 `tower_http::trace::TraceLayer::new_for_http()`，INFO 级（tracing_subscriber 已有则直接生效，否则补 `with_level` 配置，以现有 `tracing::warn!` 启动行为准）。

### T5 specs_detail fixture pre→div

`specs_detail.at:449`：`pre {` → `div {`；`.fixture-code`（:408）追加 `white-space: pre-wrap;`（承接换行语义，消除 codegen 对 pre 子树保留的模板缩进伪影——056 T3-T5 同模式）。

### T6 plans.rs force_transition

`plans.rs` 新增：

```rust
/// 系统迁移（033 W1 建模）：不经 `can_transition` 的状态写入。
/// 调用方须自证迁移合法性（如归档漏斗的两条进入路径）。
/// 未来 transition() 增加副作用（事件/钩子）时，系统迁移是显式旁路面。
pub fn force_transition(&self, seq: u32, to: PlanStatus) -> Result<PlanFile, String>
```

体内 = 读计划 → `set_field(status)` → `set_field(updated_at)` → 写回（现 `move_to_archived` 前半段逻辑抽出）；`move_to_archived` 改调 `force_transition(seq, PlanStatus::Archived)` 后只做移位。行为零变化。

### T7-T8 冒烟清单 + 执行

清单落 `docs/plans/attachments/065-smoke-checklist.md`，六项：

| # | 出处 | 面 | 步骤要点 | 预期 |
|---|---|---|---|---|
| S1 | 033 W2 | 浏览器 :9247 | Plans 视图：MetaBlock 布局/徽标中文/按钮组换行 | 布局正常、徽标中文、换行不破版 |
| S2 | 042 T10 | 浏览器 | 真实 LLM 会话：edit 改文件/读大文件截断/刷新回放 | 工具卡 details 区（diff/截断信息）正确显示，回放一致 |
| S3 | 040-1 | 浏览器 | 真实 LLM 会话跑长命令，观察 tool_update 流式进度 | 流式进度实时渲染（可与 S2 同会话） |
| S4 | 493 | VM 实机 | `AUTO_DEBUG_MENTIONS=1` 起实例，@mention 输入 | `[493-MENTIONS]` 段产出 + 蓝色着色像素目验 |
| S5 | 050 #1/#3 | VM 实机 | rail 导航目验：items-baseline 居中 / mt-auto 弹性占位 | 纵向居中、底部贴边无破版 |
| S6 | 061 D29 | 浏览器 | IAB 语言切换（中↔英） | 全界面文案翻转（D29 i18n-instance 根修的活体复验） |

S1-S3 一场浏览器会话顺带完成；S4-S5 VM 实机；S6 浏览器。结果回填清单（✅/🔶+注记），服务拉起按 `scripts/dev-stack.mjs` 惯例（`musk serve --addr 127.0.0.1:9247`，CWD=tmp/musk-demo）。

### 门禁与销号

T9 全量门禁复跑；T10 复审 + DEBT 表销号：027 行、033 W1/W2、040-1、042 T10 遗留注记、048-c、048-d（注记已闭）、056 两行（缓存债 + pre 缩进）、050 rail 目验、061 D29 复验、493 mention 复验。

## 测试设计

- `stream_event_to_json` 新增断言：`[security denied ...]` / `[tool error: ...]` → `status == "error"`；正常文本 → `"success"`；既有 042 details 透传断言回归。
- forge_store 嗅探删除后：gen `pnpm vitest run` 基线全绿（现 36+1skip 口径，以复跑为准）。
- Cache-Control：`curl -sI http://127.0.0.1:9247/` 与 `/assets/index.js` 均含 `cache-control: no-cache`。
- TraceLayer：起 serve 后 curl 任一端点，stdout 出现请求日志行。
- `force_transition`：新测试 `force_transition_sets_status_and_updated_at` + 既有 `move_to_archived` 系测试回归（plans.rs 既有测试文件）。
- specs_detail：DOM 快照/目验（归冒烟 S1 顺带看规范页 fixture 块无首行缩进）。

## 验收标准

1. `[security denied` / `[tool error:` 前缀的 Tool 事件 SSE `status == "error"`，正常事件 `"success"`；`forge_store.at` 无前缀嗅探；单测绿。
2. dist 两类路径响应均带 `Cache-Control: no-cache`（curl 实证）。
3. musk serve 每请求一行 INFO 日志（curl + 日志实证）。
4. 规范页 fixture 块首行无缩进伪影（pre→div + pre-wrap）。
5. `force_transition` 落地 + 测试绿；归档/merge 行为回归不变。
6. 六项冒烟全部执行并回填记录（通过或残留注记）。
7. `cargo test -p musk` + `cd gen/front/vue && pnpm build && pnpm vitest run` 全绿。

## 执行步骤

- [x] T1 SSE status 真字段：`backend/crates/musk/src/server.rs` 加 `tool_result_status` + Tool 分支接线 + `:690` 回放一致性复核 + 新测试；验证 `cargo test -p musk stream_event`
  - [✅ 已完成] `tool_result_status` 落地(commit 3f5a2f5)；新测试 `contract_stream_event_tool_result_status_is_true_field` 绿；持久化分支改从 SSE value 同源取 status(两轨一致)；`cargo test -p musk stream_event` 5 passed(含 042 details 回归)。
- [x] T2 gen 前端去嗅探：`src/front/forge_store.at:455-461` 删两行嗅探保留 ev.status 判定；验证 `cd gen/front/vue && pnpm build && pnpm vitest run`
  - [✅ 已完成] commit 9f90d86;嗅探两行已删,`ev.status == "error"` 判定保留。附带修复:064 T10 漏的 `api.at` ForgeSession.thinking_level 契约字段(后端真源已有,补齐后 vue-tsc 才绿——类型漂移属 064 遗留,非本计划设计变更)。worktree 冷启动需 `auto build --gen-only` 后补 shadcn 快照(--gen-only 跳过 materialize,从 auto-lang assets + 主检出 gen 平拷 ui 组件)+ `pnpm add vue-router`;vitest 按 vitest-shim.d.ts 惯例 `npx -y vitest@2.1.9 run`。验证:`pnpm build` 绿(build 8.46s)+ 36 passed | 1 skipped(基线吻合)。
    - 注记:`auto build --gen-only` 会顺带再生成 auto-lang 组内兄弟 worktree 的 `examples/rust-workspace/auto-musk-back`(含 thinking_level 类型扩散)——该示例产物 musk 不消费,为守"零 auto-lang 改动"约束已在 auto-lang worktree 回退;其正规归宿是 auto-lang 侧周期性 chore 再生成提交(f21dc88f6 先例),不经本计划。
- [x] T3 缓存头：`backend/crates/musk/Cargo.toml` 加 `set-header` feature + `server.rs` SetResponseHeaderLayer；验证 `cargo build -p musk && curl -sI` 两路径含 no-cache
  - [✅ 已完成] commit 32909a0;tower-http features +["set-header"] 追加 tower="0.5" 直接依赖(ServiceBuilder::service 包 ServeDir 链,ServiceExt::layer 方法解析不可用);curl -sI http://127.0.0.1:9247/ 与 /assets/index.js 均含 `cache-control: no-cache` 实证。
- [x] T4 请求日志：`Cargo.toml` 加 `trace` feature + TraceLayer；验证起 serve 后 curl 端点出日志行
  - [✅ 已完成] commit 626a3df;trace feature 随 T3 一并入库(32909a0)。DefaultMakeSpan/INFO + DefaultOnResponse/INFO 挂最外层;实证 GET /、/api/health、/assets/index.js 各一行 `request{method=.. uri=..}: finished processing request latency=.. status=..`。
- [x] T5 fixture pre→div：`src/front/specs_detail.at:449` pre→div + `.fixture-code` 加 `white-space: pre-wrap`；验证 `pnpm build` + 目验
  - [✅ 已完成] commit 4af0182;codegen 后 TestDetail.vue 生成 `<div class="fixture-code">` + CSS 含 pre-wrap,`pnpm build` 绿(8.64s);目验归 S1 顺带(规范页 fixture 块无首行缩进)。
- [x] T6 force_transition：`backend/crates/musk/src/plans.rs` 新增 pub fn + move_to_archived 改调 + 新测试；验证 `cargo test -p musk plans`
  - [✅ 已完成] commit b4c5c70;`force_transition_sets_status_and_updated_at` 绿(哨兵 updated_at 防同秒假阳);`cargo test -p musk plans` 37 passed(archive/transition 系全回归,行为零变化)。
- [x] T7 冒烟清单：写 `docs/plans/attachments/065-smoke-checklist.md` + 拉起服务（dev-stack 惯例）；验证清单文件在案且服务可达
  - [✅ 已完成] 清单已提交(worktree);serve 已在 127.0.0.1:9247 常驻(worktree 构建,含 T1-T6 全部改动;期间 9247 曾被并行会话 lang-566 的 auto.exe 短暂占用,其退出后绑定成功);curl / 200(含 no-cache 头)、/api/health 200。
- [x] T8 执行六项冒烟（S1-S6）并回填结果；验证清单六格全部有记录
  - [✅ 已完成] 2026-09-07 用户执行,六格全记录:S5 ✅;S1/S2/S3/S4/S6 🔶(部分通过/复现失败),新发现 F1-F6 全部在案清单(详情/证据/修法)。期间环境处置:aaid 后起致 serve 的 NoDaemonClient 永久化→重启 serve 自愈(登记新观察见 065 行)。
- [x] T9 门禁复跑：`cargo test -p musk` + `cd gen/front/vue && pnpm build && pnpm vitest run`；验证全绿
  - [✅ 已完成] `cargo test -p musk` 620 passed / 0 failed;`pnpm build` 绿(18.62s);vitest(按 shim 惯例 `npx -y vitest@2.1.9 run`)36 passed + 1 skipped(基线口径吻合)。
- [x] T10 复审 + DEBT 销号（027/033×2/040-1/042/048-c/048-d/056×2/050/061-D29/493 复验）
  - [✅ 已完成] KNOWN-DEBT-AND-RISKS.md 销号 6 条(027/033-W1/048-c/048-d/056×2),复验回填 5 条(040-1/033-W2/050/493/061-D29),新账 065 行登记 F1-F6+serve client 不自愈观察。计划级复审(验收标准逐条核验+spec-impact 元数据)移交 /auto-plan:review。

## 复审记录

- **复审人**：ZCode /auto-plan:review；**时间**：2026-09-07
- **复核足迹**：分支基点 26858e3 → plan-065-dev，12 提交，真实改动 8 文件（backend Cargo.toml/server.rs/plans.rs、src/back/api.at、src/front/forge_store.at/specs_detail.at、docs 账本+冒烟清单）——与计划声称一致；worktree 内静态面 + 活体 curl + 全量门禁全部复跑。
- **逐条验收**：
  1. SSE status 真字段 — **pass**：`tool_result_status`（server.rs:449）+ Tool 分支接线（:494）+ 持久化同源；forge_store.at 前缀嗅探 0 命中、`ev.status` 判定在位（:458）；契约测试（含四断言新测试）在 620 内绿。
  2. Cache-Control — **pass**：复审复跑 curl，`/` 与 `/assets/index.js` 均 `cache-control: no-cache`。
  3. 请求日志 — **pass**：复审复跑 curl `/api/health`，INFO 单行 `request{method=.. uri=..}: finished processing request latency=0 ms status=200`。
  4. fixture 无缩进伪影 — **pass（生成物级）**：TestDetail.vue 生成 `div.fixture-code` + pre-wrap 实证；活体目验无标本（全库 docs/specs 零 `**Fixture:**` 测试项，区块不渲染属预期）——已在 DEBT 056 行注记，非缺陷。
  5. force_transition — **pass**：plans.rs:505 pub fn + `force_transition_sets_status_and_updated_at`（哨兵 updated_at 防同秒假阳）绿；archive/transition 系全回归（行为零变化）。
  6. 六项冒烟 — **pass**：六格全记录（065-smoke-checklist.md）——S5 ✅；S1/S2/S3/S4/S6 🔶 带残留注记，新发现 F1-F6 + serve client 时序观察全部登债（KNOWN-DEBT 065 行）。
  7. 全量门禁 — **pass**：`cargo test -p musk` **620 passed / 0 failed**（复审复跑，唯一全量门禁）；`pnpm build` 8.94s 绿；vitest（npx@2.1.9 锁版）36 passed + 1 skipped。
- **遗漏/延后/workaround 排查**：DIFF 零 TODO/FIXME/hack。①T2 附带 `api.at` ForgeSession.thinking_level 契约字段——计划文本外、但为 vue-tsc 过门的必要对齐（064 T10 遗留类型漂移），已在 T2 标记披露，非隐性遗漏；②web `useForge.ts` 嗅探保留=计划待澄清④明示（web 冻结），合规；③"零 auto-lang/auto-ai 代码改动"约束成立（codegen 对 sibling worktree 的 examples 副作用已回退，两依赖 worktree clean）；④无未批准延后——F1-F6 为冒烟新发现登债，非计划任务顺延。
- **债务候选（已全部登记 KNOWN-DEBT-AND-RISKS 065 行，非阻塞）**：F1 gen 状态徽标 i18n 零消费；F2 composer mention 检测链 codegen 事件契约错位；F3 直播流被 PollStream 快照覆盖；F4 gen 轨无 tool_update 消费臂；F5 details 载荷不持久化（后端契约）；F6 一级导航硬编码中文；附：serve 对 aaid 的 client 启动时缓存不自愈。
- **merge 注意**：main 在分支存续期前进一个提交（f496fc9 composer 一体化改版，动 mention_input.at/chats_view.at/icons 族——与分支 8 文件不相交，预期无冲突；但同属 F2 关联面，折叠后建议复跑 gen 门禁一次）。worktree 的 pnpm node_modules 含 junction 农场（gitignored 构建产物），merge 清理 worktree 前须按 wt-guard 指引先以 junction 安全方式删除 node_modules。
- **结论**：七项验收全 pass，无阻塞债 → **reviewed**，可进入 /auto-plan:merge。

## 待澄清事项

1. **冒烟 S1-S3/S6 需用户在普通浏览器执行**（会话内置浏览器有 webview 挂载限制，033 W2 两次复现）；S4-S5 需 VM 实机目验。T8 为用户执行步骤，agent 负责清单与记录回填。
2. status 取值定为 `"error"`（对齐 `forge_store.at:457` 既有消费），非 "failed"——已裁定，不再议。
3. vite 产物 hash 化归 auto-lang 模板批次，本计划只做服务端 no-cache——已裁定。
4. web 轨 `useForge.ts` 嗅探保留（FROZEN），DEBT 注记销号时说明。
