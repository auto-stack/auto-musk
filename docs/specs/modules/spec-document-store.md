# Spec 权威文档、编辑和自动索引

> PLAN-106；策略 document-source-v1。实现：spec_documents.rs、spec_index.rs、spec_migration.rs、spec_tree.rs。

## 数据和元数据

当前知识以 docs/specs 下 UTF-8 .md/.markdown/.ad 为源，AutoDown 负责双端编辑展示，不强制转换格式。无 frontmatter 的现有文档可直接使用。可选 YAML 字段 spec_id/title/section/status/tags/depends_on；原文保存保留未知字段。显式 ID 保持重命名身份，否则使用 file:docs/specs/相对路径。重复 ID、未知状态/分类有诊断，不覆盖另一文档或凭空变成 approved。

归档 Plan 和 docs/specs/history 为历史输入，进入 reports 投影并带 historical 标签，不覆盖现行要求。六区为只读查询兼容形状，历史生命周期状态保存在 source_status 标签，不能误当 Spec 批准状态。version 仅兼容显示，源身份使用完整 source_hash。

## 保存与并发

GET `/api/specs/document/{path}?workspace=…` 返回 content/body/preamble/hash/editable/error；富文本编辑 body，原始 preamble（含 BOM/CRLF 与未知元数据）逐字拼回保存；路径相对于 docs/specs。PUT 同地址接收 content/expected_hash（新建 null，更新读取所得哈希），返回实际落盘后的内容与哈希。版本冲突 409，主检出或越界 403，未知工作区或源 404，格式不支持 400。失败不假成功，冲突保留草稿。未保存时拦截文件/栏目切换并显示保存或放弃提示，Web 关闭页面有 beforeunload 保护。

文件哈希比较和原子写入由按文件 OS 锁一起保护，跨进程也成立；不同文件独立。锁文件位于 .autoos/spec-locks，句柄关闭/进程退出自动解锁，锁文件不删除以免出现多 inode 竞争。

Git 主检出只读，专用 worktree 可编辑；非 Git 工作区可编辑。应用不自动提交或合并；agent 正常文件工具遵守 AGENTS.md。路径阻止遍历、绝对路径与经链接越界，创建验证父目录，现有 symlink/reparse 组件拒绝。源保存成功与索引诊断分别表达。

## 自动索引

GET `/api/specs/index` 返回 document/source_hash/fresh/diagnostics。按需扫描文档与历史、排序计算内容指纹、两次快照校验。只生成内存结果，不发布共享缓存文件，因此没有旧构建覆盖新持久缓存的问题。外部编辑、Git 合回和停机期间增改删名在下次读取发现。扫描过程中变化时 fresh=false 并提示重读；普通读取不把混合结果伪装为成功索引。

depends_on 生成反向关系并诊断断链，兼容正文 ID 关系扫描。旧 JSON 损坏或不可写不影响重建；原文读取不依赖索引。docs/specs/index.json 手填清单退役。

## 显式旧数据迁移

`musk spec-migrate --root <worktree> --source <old-specs.json> --journal <local-manifest>` 缺省 dry-run，只读源，生成逐条映射和字节级备份。`--action apply --legacy-writer-stopped` 要求事先停止/隔离旧进程；校验源/目标/产物指纹，写入历史导入文档，重复执行不重复生成，可从部分完成继续。所有顶层/区扩展字段保留在 envelope，每条记录保留未知字段、状态及完整正文，重复旧 ID 按源指纹和位置区分。

已有 canonical 文档优先关联，不覆盖；缺少可核实来源的当前知识标记 needs-reconciliation，需正常评审后提炼为模块合同，不能因导入成功自动变成现行要求。历史优先关联归档 Plan。

`--action rollback` 只删除本次新增且仍等于迁移哈希的文件；后续用户编辑使回滚拒绝，保留部分完成状态用于恢复。源损坏不改源，指引备份恢复；当前 docs/specs 与 merge 可继续使用。原 JSON 不删除，不自动导入、不批量操作其他工作区，指纹变化重新 dry-run。

## 交付恢复

prepare → land → archive → cleanup；归档核验 landed commit 为默认分支祖先及 canonical 目标存在于该提交，不要求 ledger/index checkpoint。refresh 仅作旧调用兼容的 canonical 验证，不写 JSON。旧 ledger_refreshed 为历史事实，不能证明 landing。

skills_hashes 保留批准快照作审计，不静默改写。新 run 使用更新技能，旧 run 的 document 相位须以当前文档策略覆盖旧发布要求，同时维持合同、提交、依赖检查。已 landed 恢复不补写账本，已归档只验证/清理、不重放旧规范增量。
