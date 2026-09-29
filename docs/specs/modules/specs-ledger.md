# specs ledger — Spec 账本（`.autoos/specs.json`）契约

> PLAN-094 SD-01 定型（UAT K6 实测教训：无契约文本导致聊天侧 merge 即兴手写
> 外语格式 JSON、reviewer 手工 JSON 手术抹掉既有条目）。本文为账本的规范性
> 描述；实现锚点见各节行内引用。

## 性质：派生视图，非权威

`docs/specs/` 是当前项目知识的**唯一权威**；ledger 只是它的派生视图——索引、
关系与历史的机器可读落点。**永不**把 Plan 章节复制成第二份独立维护的规格。
两者内容冲突时，以 `docs/specs/` 为准并修账本，不反之。

## Schema：由 store 拥有（应用原生格式）

账本只存在一种合法格式 = `backend/crates/musk/src/specs.rs` 的
`SpecsDocument` serde 形状：

- 顶层字段：`project`（string）/ `version`（number，每次写递增）/ `sections`
  （数组）。
- section id 恰好六个：`goals` / `architecture` / `designs` / `tests` /
  `reviews` / `reports`。**没有 `plans` 区**——它在 PLAN-024 已退役
  （`SectionType::Unknown`：load 时容忍并过滤、永不回写；向其写入的内容会在
  下一次 load 时被静默丢弃）。
- item（`SpecItem`）字段由 store 的 `SpecItem::new` 工厂补全（id / title /
  content / status / depends_on / related / priority / assignee / test_file /
  file / milestone / module / tags / created_at / modified_at /
  completed_at）；`related` 是计算的反向链接字段，不得作为唯一出处存储。

## 写入通道：只经 store 语义（store-mediated）

对 `{root}/.autoos/specs.json`（workspace 作用域）的一切写操作必须经过
store 的 upsert/transition 语义，二选一：

1. **spec 工具**：`write_spec` / `update_spec`（及只读的 `read_specs` /
   `list_specs` / `write_goals`）——聊天会话与 Relay 相位均已注册，
   workspace 作用域（`spec_tools.rs ws_specs_of`）。
2. **HTTP 等价端点**：`POST /api/specs/item`（upsert）与
   `/api/specs/transition`——同样走 `load → SpecItem::new → upsert_item →
   save`（`extern_impl.rs specs_upsert_of`），ledger 不可解析时同样失败。

**禁止**以任何脚本、文本替换、手拼 JSON 直写该文件（含"离线读改写"变体）：
未经 schema 校验的手工替换是外语格式与丢数据的成因（K6）。没有可达的
store-mediated 写通道时，正确动作是**记录 blocked**，不是绕过。

## 解析失败：响亮失败 + 零破坏

- `SpecsStore::load()` 对"文件存在但解析失败"只报错（`InvalidData`），
  **绝不重写文件**；NotFound 的自愈建新档保留（文件缺失时无数据可损）。
- 五个 spec 工具的 load 失败统一经 `spec_tools.rs load_err` 返回带上下文的
  错误：原 serde 错误 + 期望字段 + 六区清单 + **"Do NOT hand-edit or
  recreate"明令** + 恢复路径（备份恢复，或以 `docs/specs/` 重建后重试）。
- 模型/agent 侧的对应纪律：看到 load 错误即**停止**，按错误指引恢复；手改、
  删除重建、逐字段"手术"都是被明令禁止的破坏路径（UAT 2026-09-29 K6 实录：
  reviewer 的手工手术抹掉 SD-01 条目）。
- 用例锚点：`spec_tools.rs` 测试 `foreign_shape_ledger_fails_loud_and_intact`
  （外语 fixture 上调用四个写读工具 → 错误含关键字 + **fixture 字节前后
  不变**）。

## 区映射建议（派生索引的落点）

按**知识类型**映射，不按 Plan 章节：目标 → `goals`；架构决策 →
`architecture`；模块/设计契约 → `designs`；测试覆盖 → `tests`；复审结论 →
`reviews`；沉淀报告 → `reports`。源哈希与 commit 写入 item 的 `tags`/`file`；
正文只放派生摘要，不复制 Plan 章节。现有同目标 item 按 canonical target +
section 复用，不新造重复条目。

## 已知限制

- 无 schema 版本迁移/自动格式转换：外语格式只该被拒绝并指引，不被猜。
- `load()` 不做隔离改名（读路径保持无副作用）；响亮错误 + 禁手改指引已足够。
- 单文件 JSON 存储（无并发事务）；多写者协调靠"一个写者"纪律，工具序列
  read/check/write 不构成锁。
