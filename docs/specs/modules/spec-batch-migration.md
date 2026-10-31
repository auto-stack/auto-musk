# 多项目旧账本迁移、去重与恢复

> PLAN-107；沿用 document-source-v1 权威边界，manifest v2 增加来源登记。实现：spec_migration.rs、scripts/spec-migration-batch.py、scripts/spec-migration-reconcile.py。

## 数据与身份

当前知识在 docs/specs 的 UTF-8 Markdown/AutoDown 文档；历史记录在 history/legacy-ledger。原 specs.json 只作明确选择的只读输入，不能因导入而生成新的批准状态或把旧测试数字当本次执行。

支持 sections 对象数组、section→items 数组字典，以及带 items 的 section 对象字典。错形明确失败；空正文/未知字段/缺旧 ID 可保全。serde_json 保持对象顺序与任意精度数字，原字节另行备份。JSON 围栏长度按原内容选择，完整记录可从 Markdown 围栏回读。

来源键、完整源 SHA256、区块键/位置与条目位置共同定义迁移身份，旧 ID 非全局唯一键。v2使用完整哈希的路径命名空间；不同项目或 root/backend 不因内容或 ID 碰巧相同而合并。

生成 Markdown 统一使用 LF，包含用于阅读的旧正文；围栏中的原 JSON 字符串仍保留原始 CRLF 值，源字节备份不变。输出字节哈希必须跨 Git 提交/检出保持一致。旧混合换行 manifest 遇到 Git 归一化后的文件应明确报告指纹漂移；保留旧 journal，重新 dry-run 生成独立恢复材料，不覆盖或伪造旧证据。

## 显式批次

`musk spec-migrate --root <worktree> --source <snapshot> --source-key <registered-key> --journal <local-journal>` 默认 dry-run；apply/rollback 仍使用相同引擎。source-key 不得含路径字符，目标只能在受限 history 目录。专用工作树可写，主检出拒绝。

批次脚本接受显式 --registry、--binary、--action prepare/apply/verify/rollback 与可选 --only 项目。登记包含只读原源、隔离快照、两者哈希、明确目标/解析根、初始提交和副本别名。每次操作核验相关原源/快照/副本；源漂移需重新冻结，不能丢弃新增记录。一个项目漂移不阻止显式选择的其他项目。

auto-musk/backend 的引用与目标根均为 auto-musk 项目根，不能创建 backend/docs/specs。archive/archived 按真实路径识别，歧义计划编号不猜测跨仓归属。file 字段及旧 CLI 的 Canonical reference 标签是原源定位线索，也可能指代码/历史证据；实际当前/历史归属以逐条对账为准，不能将其当自动内容批准。

六个组合子模块在独立仓身份及记录集等价确认后仅登记 alias。原字节格式差异仍有各自备份和 hash；不重复输出、不改变子模块 gitlink，发行组合保持原 pin。副本独有内容必须先列差异，不能静默当重复删除。

## 发布、恢复与撤销

恢复材料使用 exact/no-clobber 原子写入，备份必须等于 frozen source hash。v2 manifest 还需校验独立 digest；v1 旧 manifest 可按已有备份策略恢复，不伪造过去的摘要。CLI绑定实际root/source/source-key，拒绝使用其他项目的journal。

compare-and-save 保持 OS 文件锁与原子落盘；迁移另有项目锁。批次状态以实际文件重建 prepared/verified/partial-or-conflict/import-rolled-back，失效和部分完成不会报成全批成功。精确产物识别可续跑同一manifest，重复执行不新建第二份历史。

rollback 仅删除自身新增且 after hash 未变的导入文件，pre-existing 或后续编辑不会被覆盖。批次 rollback 的 import-rolled-back 只表示原值导入撤销，不意味着报告/人工提炼/规约的 Git 改动也已撤销。完整写集合记录before/after指纹与提交；已合入的撤销需新反向Git提交，保留后续无关修改，重叠修改明确冲突。跨仓无原子提交承诺，逐仓保存已落地与撤销结果。

## 知识核对与运行验证

每条旧记录都保全原文/状态/依赖及扩展字段，并登记当前canonical、唯一归档Plan、历史源上下文或显式例外。现有规范优先，不机械复制Plan章节；无正文测试占位不制造业务要求，外部旧测试收据不冒称已重跑。

旧Web集成合同从 docs/web-integration.md 提炼归位到各自 docs/specs/modules/web-integration.md，原路径只转引，职责和历史证据边界保持。无Spec树的项目建立最小知识入口，不把旧架构快照冒称当前完整实现。

实际自动索引和Web/VM阅读验证与字段回读分别记录；索引不是发布门，agent不维护中央清单。各仓保留原评审、受影响验证和ff-only纪律；仅移除旧JSON/upsert/配额与强制手填索引义务。临时证据依当前保留政策整理，长期去向和撤销依据进入各仓文档及Plan收据。
