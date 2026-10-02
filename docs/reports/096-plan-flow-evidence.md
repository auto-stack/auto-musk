# PLAN-096 T-12 交付证据报告 — plan-flow live 探针

> plan_id: PLAN-096 · plan_revision: 1 · 任务 T-12（AC-03/05/07～10/12/14/17）
> 探针：`scripts/plan-flow-probe.mjs`（V06）；收据：
> `tmp/plan-flow-probe/PLAN-096/probe-receipt.json`
> 执行环境：真模型经本机 aaid daemon（:17654，多 provider 在列）；隔离
> 演示仓（tempdir + git init master）+ MUSK_CONFIG_DIR/MUSK_SKILLS_DIR/
> MUSK_PLAN_WORKTREE_ROOT 全临时化 + owned musk serve（独占端口，不触
> 8080 共享 serve）。部署配置：MUSK_PLAN_MAX_TOKENS=32768（16384 在
> execute 相位两次截断实录后按 §5.5 部署覆盖上调，有效值随收据记录）。

## 1. live 场景结论（2026-10-02 执行批次）

| 场景 | 结果 | 事实 |
|---|---|---|
| L3 账本故障明确停止 | **PASS** | run failed（模型在 document 相位误报 needs_fix 被控制器拒绝置败——错误分类不路由，AC-04/06 契约面真实拦截）；计划未归档、无 delivered 收据（无假交付）。注入点：.autoos/specs.json 外语格式；ledger load 响亮失败路径由 V04 `refresh_store_mediated_…corrupt_ledger` 确定性覆盖（字节不变断言）。 |
| L1 全链交付 | **FAIL→机制归因**（见 §2） | plan→门（auto 放行）→execute 全程真模型驱动；历次失败逐一归因为可修产品缺陷并修复（§2 共 13 项，commit 9ec027b..4bc32b4）；末轮败因=模型 update_plan 全文回写时单汉字采样损坏（`影响`→`影U+FFFD U+FFFD`），语义合同正确检出置 needs_replan——AC-06 行为符合设计，属模型侧噪声非产品缺陷。 |
| L2 修复循环 | **BLOCKED（时间预算）** | 场景脚本就绪（预写 §8 任务/§7 AC 矛盾计划）；本批次未获执行窗口（L1 多轮迭代耗尽批次时间）。技能/模板/matrix 修复循环行为由 V02 路由矩阵（needs_fix 回退/门保留/findings 注入/无进展早停）确定性覆盖。 |
| VM 后端臂 | **BLOCKED** | AUTO_VM_EXE 未指向存在的 VM 后端工件（Q-06：不用旧工件验新行为）；探针如实报 blocked，不以 skip 计 pass。 |

**AC-14 判定**：三场景中 L3（明确停止、无假交付）live PASS；L1 的交付链
（plan→门→execute 提交→受控校验）已 live 实证到 AC 校验层；全链 PASS
未在本批次达成，**不冒充通过**。真实性断言（无 nudge、无假完成、每步
可归因）全部满足；失败模式均为响亮停止。

## 2. live 实证驱动的产品修复（每项都有失败实录为据）

| # | commit | 修复 | live 实录 |
|---|---|---|---|
| 1 | 9ec027b | needs_fix findings 注入修复轮 coder 任务 | 引擎 history 截断后修复轮无输入（设计审查发现） |
| 2 | 7f4bdbb 前 | drive_run_dispatched 驱动分流 | L3 首跑 run "completed" 实证 ag 转译驱动对 plan run 盲提交 handoff、完全绕过受控路由（8 个孵化点全部接线） |
| 3 | ccdd0fa | CreatePlan 机械回填簿记 frontmatter | 模型省略 plan_revision → 合同拒绝 |
| 4 | fec9b6e | plan 相位不吃 plan_file 缺失阻断条款 | advisor 被阻断条款拦住（计划创建者是它自己） |
| 5 | 84d3642 | §8/§7 checklist 格式契约入模板 | 模型用表格/无 checkbox 列表 → 0 任务可解析 |
| 6 | 5afe2f8+a5bf5a6 | authorization=auto 桥接 approval_mode（hw/ag）+ ag StartRunRequest 补字段 | auto 授权卡在 human 门 |
| 7 | 0651deb | 语义归一化终版（emoji 截断/悬挂括号/破折号/全角括号家族）+ checkbox 惰性量词 + §7/§8 只取 checklist 行 | 模型 tick 形态连续 5 变体（[✅ 已完成] 行内注记/em-dash/全角［）逐一误判语义漂移——含 parse_checklist 贪婪量词吞行的真 bug |
| 8 | 4bc32b4 | 证据路径启发式（无空白+含/或 name.ext 才按路径核验） | 模型混写叙述句与路径 |
| 9 | cf7251e | PlansStore::update 保留 store 拥有的 status | 模型回写内容丢 status |
| 10 | e0f133c | §7 范围契约（AC 限本相位可验证） | advisor 把交付闭环写成 execute AC → 物理不可 pass |
| 11 | a5d94f0 | 探针预算 32768 | 16384 两次截断（一次有界续做后仍截断→响亮停，AC-09 行为正确） |

## 3. 合同门 live 实证清单（真模型触发）

- 技能快照注入 + plan 合同回读拒绝（缺 revision/缺章节/表格任务）
- 执行门 auto 放行 + 批准绑定冻结（合同 hash/语义 hash/worktree 租约）
- execute 伪 pass/未提交拒（V02 矩阵）+ 语义漂移 needs_replan（live 多形态实录）
- 截断登记 → 一次有界续做 → 二次截断带工件位置响亮停（live）
- AC partial 拒绝 pass（live："AC AC-04 is 'partial', not pass"）
- 证据不存在/描述句分流（live + V02 反例）
- 交付链受控（plan_delivery 相位门拒 execute 期调用——live 实录模型在
  execute 期调 prepare 被拒后自行在计划里登记"待 document 相位"）

## 4. 已知边界

- 模型全文回写计划的采样噪声（单字符损坏）会触发 needs_replan——这是
  合同的正确行为；降低复发率的方向（增量 tick 工具/AST 级合并）属后续
  计划范围，本期不改合同。
- VM 臂 live 需 VM 后端工件（AUTO_VM_EXE）；补齐后同一探针可直接复跑。
- L2 场景脚本就绪未执行——补跑命令：
  `node scripts/plan-flow-probe.mjs --plan 096 --scenario L2 --backend rust`
