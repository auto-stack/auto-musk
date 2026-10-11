# PLAN-104 规范增量对照（SD-01 / SD-02）

- 对应计划 §5 规范增量表；review 终版以本对照 + 计划 AC 并集为准。
- 与 PLAN-105 SD-03 共改 `specs-ledger.md`（105 T-06 改 transition/upsert
  API 面，104 T-06 改写入通道/最少条目合同）——合并时序后到者 rebase 适配。

## SD-02（已落地，T-06 commit 93d2aad）

- 目标：`docs/specs/modules/specs-ledger.md`
- before：写入通道仅隐含（会话工具缺白名单+覆盖双闸门描述；HTTP DTO 无
  file/tags；merge 收尾无最少条目合同）。
- after：三通道合同（会话工具=白名单+PLAN-030 覆盖双闸门 / HTTP 含
  file-tags / errand 同工作区账本）+「每 merge 最少条目合同」
  （designs≥1+reviews≥1+reports≥1 有则必记、file/tags 必带、docs 版本
  控制前置 cleaned）。
- 验证：V07 实证——台账 4 条（DES/TST/REV/REP）全带 file+tags，
  spec-blob 哈希与 git hash-object 一致；merge SKILL.md 收尾清单同步。

## SD-01（未落地——随 AC-01/AC-02 blocked on auto-lang 挂起）

- 目标：`docs/specs/modules/vm-process-stability.md`
- 拟增：活跃轮询渲染内存有界（斜率阈值）+ 大载荷降级渲染行为（阈值+
  尾窗+横幅）+ 审计判读矩阵 active 场景行。
- 状态：musk 侧杠杆（tier1/tier2 增量合并）已尽并随仓（0cb5656/ac5b22d），
  全额阈值达标需 auto-lang 面（idle 30-72MB/min 残留 + 轮询负载 2-4 分钟
  原生 exit(1)）；T-03 大页降级与 SD-01 文本随解阻后补（§10① 裁定）。
