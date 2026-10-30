# Spec 分类索引与旧账本兼容

> PLAN-106 替代 PLAN-094/096/104 中央账本写入合同；PLAN-105 的错误诚实性保留。

`docs/specs/` 是当前知识唯一权威。应用从文档与归档 Plan 自动生成六类只读投影：goals、architecture、designs、tests、reviews、reports。历史带 historical 标签，不定义当前要求。保存和迁移合同见 [spec-document-store.md](spec-document-store.md)。

`.autoos/specs.json` 退役为显式迁移输入，不由 merge、工具或界面维护。应用工作区使用 SpecsStore::for_documents；既有 SpecsDocument 形状仅作查询兼容。旧 item/transition/delete HTTP 写接口明确返回 410 和源文档指引，write_spec/update_spec/write_goals 工具返回明确错误；read/list/overview/rebuild 从文档生成，rebuild 不落账本。

不再要求 merge 的最低条目配额、file/tags 投影发布、双轨 HTTP/离线 JSON 写入、ledger_refreshed 检查点或手填 docs/specs/index.json。缓存、服务或旧账本不可用不影响已验证的 canonical 交付。

旧账本独有内容不得删除；显式迁移逐条保全原始字段、状态和正文到 docs/specs/history/legacy-ledger，当前知识与历史分离。原字节备份和运行 journal 留本地，Markdown 报告可提交。不批量改写旧归档收据。
