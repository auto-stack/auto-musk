# 全局架构

> Spec 模块树知识库 —— 架构占位（008 §5）。本骨架由 PLAN-025 建立。

auto-musk 的全局架构概览。待沉淀。

## 双落点说明

Spec 知识目前有两个落点（职责分离）：

- **`.autoos/specs.json`** —— 结构化工作台（item 级 CRUD + 状态机 + gate 审批）
- **`docs/specs/`（本目录）** —— 知识沉淀层（markdown 长文档，本骨架）

前端 SpecsView 顶部 toggle 可在"结构化编辑"与"文件树"两种视图间切换。
