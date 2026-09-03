---
plan_id: PLAN-057
status: archived
feature_name: VM/Vue Block 全量对拍第五批——VM/TS 语义等价性根修
author: [zhaopuming]
created_at: 2026-09-02T17:05:00+08:00
updated_at: 2026-09-03T02:40:00+08:00
supersedes_spec_components:
  - "KNOWN-DEBT「VM/TS 语义等价性缺陷族」行①–⑤——改写为已修注记（根修要点+回归面+门禁配套；缓期项转 057 残差行）"
  - "src/front: forge_helpers toolArgsJson（PLAN-055 T0 降级）——恢复 JSON.stringify 直通（T6 native 落地）"
  - "src/front: questionnaire_helpers 问卷多选摘要（PLAN-055 T0 旁路）——恢复 Array.isArray 判别"
  - "src/front: forge_helpers messageBlocks（PLAN-055 T0 注记）——T2 后 VM 已合法，按待澄清③保留字面量重建（注释更新）"
  - "src/front: questionnaire.at questionnaireFor 与 forge_helpers stripQuestionnaire——Regex 围栏提取改 indexOf 纯串通道（VM Regex 恒空）"
  - "src/front: 六类卡根容器（generic_tool_card/errand_card/report_card/questionnaire_card/relay_run_box/task_plan_card）——补 tailwind 框工具类（原始 CSS 为 Vue 专属直通）"
new_spec_components:
  - "scripts/vm-safe-lint.mjs——VM-safe .at 子集五模式静态门禁（P1 新键/P2 for-in 调用源/P3 web 内建白名单/P4 字符接收者/P5 isArray）+ // vm-safe-allow 行级豁免，白名单随 T6 natives 增补"
  - "auto-lang: SET_FIELD ObjectData 新键插入语义（engine.rs）——JS obj.k=v 赋值语义，GenericInstanceData 保 Plan 118 严格性"
  - "auto-lang: for-in Call 源索引通道泛化（codegen.rs E5b）——例外=迭代器协议链（含生成器 fn）+stream/sse_ 惰性流"
  - "auto-lang: web 内建 natives 五件（Array.isArray 1919/JSON.stringify 1918/parse 数组接线/Math.trunc/Math.imul）+模块式全局编译期门禁（// vm-safe-allow 豁免）"
  - "auto-lang: CALL_SPEC 三处未知方法兜底配平 + Char 码点恒等臂重定位 + nv_to_vm_value 嵌套句柄堆判定前移"
  - "KNOWN-DEBT 057 行——T11 实机残差七笔（__json_object 字符串字段读污染/Regex 恒空族/ThinkBlock 读侧/框色差近似/ext-stub 警告口径/合成输入守卫/markdown 缓期）"
touched_goals:
  - "P028 block-autolang-full-migration：VM 轨语义地基补齐——带 tool_calls 消息 VM 整条渲染（a1 全家福死因根除）+工具卡参数 JSON+卡片框架"
  - "P038 第三方库 Auto 版替换（VM 轨）：web 内建 natives 补齐+编译期门禁（isArray/stringify/parse/trunc/imul）"
  - "P029 frontend-escape-hatch-elimination：vm-safe-lint 门禁固化 VM-safe 子集纪律"
current_step: 12
total_steps: 12
---

# [PLAN-057] VM/Vue Block 全量对拍第五批——VM/TS 语义等价性根修

## 变更摘要

2026-09-02 实机对拍诊断出 VM/TS 语义等价性缺陷族（KNOWN-DEBT「VM/TS 语义等价性缺陷族」行①–⑥，micro-probe 实测 `tmp/vmprobe/`）：①SET_FIELD 拒绝新键写入（Plan 118 过度矫正）②for-in 源为调用结果静默零迭代（Plan 454 E5b 通道缺口）③web 内建覆盖不全且未实现者静默 None ④for-in 裸字符接收者方法落 None ⑤实参含直接调用时相邻串参数错乱。五项合谋导致 VM 轨带 tool_calls 的消息整条空白（Block 全家福 a1）、工具卡参数展示空白、问卷多选摘要不渲染。

本计划两仓联合根修：auto-lang 侧修 SET_FIELD/E5b 通道/char 分派/参数入栈序、补 web 内建 natives、未实现内建升编译期报错、图标桥扩容；musk 侧固化 VM-safe .at 子集静态门禁、解除 055 T0 临时规避、执行 Block 逐类对拍矩阵。**markdown/autodown 富渲染明确 out of scope**（`renderer.vm.at` 纯文本维持，等 autodown 下一波；切换路线=port 实现替换 + auto bin 开 `autodown` feature，已登记账本）。

## 目标

1. `tmp/vmprobe/case_setfield_newkey.at` 跑完不中止且 A–D 四 PASS（SET_FIELD 支持动态新键写入）。
2. `case_forin_call.at` 5/5 PASS（for-in 源为任意调用表达式按 TS 语义迭代 N 次）。
3. `case_web_builtins.at` 8/8 PASS（Array.isArray/JSON.stringify/JSON.parse 数组形态与 JS 同语义）。
4. `case_str_charcode.at` 3/3 PASS（for-in 字符接收者方法返回码点）。
5. 实机（musk serve + `auto run -r vm --no-merge`）：Block 全家福会话 a1 出 ThinkBlock+文本+7 张 ToolBlock，工具卡参数展示非空、问卷多选摘要渲染；与 gen Vue 同会话逐块对拍，残差仅为已登记降级项。
6. VM 目标对未实现 web 内建**编译期报错**（构造用例验证），不再运行期静默 None。
7. 图标桥扩容后 VM 实机快照无图标类 no-op（ext stub 警告清零或仅余已登记豁免）。
8. 两仓常驻门禁零新增红；worktree 按 AGENTS.md 合回+清理。

## 架构方案

**两仓 worktree 布局**（AGENTS.md）：auto-musk 侧在 `.worktrees/plan-057-dev`；auto-lang 侧在 `D:\autostack\auto-lang\.worktrees\auto-musk-dev`（分支已存在则复用）。auto-lang 侧改动一旦被 musk 消费（四 case 全绿+实机验证）即合回 auto-lang master 并清理。

**根修策略**：
- SET_FIELD（`engine.rs:5030` ObjectData 臂）：查不到键时改走 `obj.set` 插入（底层数据结构本就是开放 HashMap，`types.rs:157`）；`GenericInstanceData` 分支保持报错，保 Plan 118 的类型严格性语义。
- for-in Call 源（`codegen.rs:2650` Plan 454 E5b）：把"临时句柄+索引循环+GET_ELEM"通道从静态 Array 型 Call 泛化到全部 Call 源（或运行时 `.iter()` 对 List 句柄返真迭代器，二选一以实测代价定）。
- 参数求值栈错乱（⑤）：与 E5b 同族，排查实参含直接调用时的入栈序/求值序，以 case_web_builtins A/B/C 标签完整性为回归面。
- char 接收者（`engine.rs:7213` 恒等臂在位但未命中）：定位 for-in 裸字符的实际 CALL 路径，使恒等臂生效或补分派臂。
- web 内建：补 natives（Array.isArray/JSON.stringify/JSON.parse 数组接线）；**编译期报错门禁**在 a2vm codegen 收敛点对无法解析的 web 内建全局报错，豁免机制沿用/统一既有 ext-stub 登记体系（icons 先例，050 C5）。

**musk 侧策略**：
- VM-safe .at 子集检查单固化为 `scripts/vm-safe-lint.mjs` 静态扫描（五模式：新键赋值启发式/for-in 直接调用源/web 内建调用 vs 已实现白名单/字符接收者/`Array.isArray`），支持 `// vm-safe-allow <原因>` 行级豁免。
- 055 T0 的临时规避（字面量重建/降级展示）在上游 natives 落地后**择机解除**：stringify/isArray 直接回归真实现；messageBlocks 字面量重建倾向保留（双轨行为更可预测，回退无收益）。
- Block 逐类对拍矩阵以实机 MCP snapshot + 截图为准，残差二分：语义缺失→修复；纯样式降级→账本登记。

## 技术栈

- auto-lang：Rust（VM engine `engine.rs` / codegen `codegen.rs` / native FFI `stdlib.rs` / 图标注册表 `lib.rs`）
- auto-musk：.at 单源（`src/front/forge_helpers.at` 等）、Node 门禁脚本、gen Vue 轨（对照基准）
- 验收工具：`tmp/vmprobe/case_*.at` 四件（自判定 PASS/FAIL）、AutoUI MCP（:9247）snapshot、`scripts/vm-first-run.mjs`

## 需求分析与背景调查

- 本波最终目标（用户 2026-09-02 定）：**VM/Vue 各类 Block 基本一致显示；markdown/autodown 除外**（等 autodown 下一波更新后移植，路线已备：VM 原生 markdown 臂 `aura_view_builder.rs:1395` autodown-core 真渲染存在，仅差 musk port 接线 + auto bin `autodown` feature 未编入）。
- 缺陷族根因与探针证据：KNOWN-DEBT「VM/TS 语义等价性缺陷族」行①–⑥（本会话实测）；`ObjectData` 开放 HashMap（`types.rs:157`）、SET_FIELD 策略翻转史（Plan 118 Fix 4，`docs/plans/archive/118-vm-test-failures-analysis.md`）、E5b 通道自述（`codegen.rs:2650`）。
- musk 源五模式普查结论（账本⑥）：P1 仅 `forge_helpers.at:236` 踩雷（gate_inbox 三处既有键安全）；P2/P4 零命中；P3 掩蔽雷点 `toolArgsJson`/`questionnaire_helpers.at:76` 两处。
- 关联 spec 模块：P038（第三方库 Auto 版替换——i18n/icons + 渲染真源切 auto-down——VM 轨）、P028（block-autolang-full-migration）、P029（frontend-escape-hatch-elimination）。本计划为其 VM 轨分项补齐语义地基。
- 依赖关系：PLAN-055 T0（临时规避）先行；本计划 T10 解除之；PLAN-055 T5/T13/T19 实机验收在本计划根修折入后执行。

## 详细设计

- **SET_FIELD 插入语义**：`ObjectData` 臂在 Str 键 `obj.get` 未命中时不走 Plan 118 报错，直接 `obj.set(Str(key), value)`（插入）；Int/Bool 键格式与 GenericInstanceData 分支维持现报错。配三用例：空字面量加键、for-in 变量加键、typed instance 加键仍 Err。
- **E5b 泛化**：`Stmt::For` 的 `Expr::Call` 源不再区分静态型别，统一走临时句柄+索引循环（保留既有 Array 型路径）；`let` 绑定路径不动。性能敏感场景（大列表）实测后再定是否补真迭代器通道。
- **web 内建解析收敛**：codegen 对 `Array.*`/`Object.*`/`JSON.*`/`Math.*` 全局收敛到一张 natives 表（已实现者直连，未实现者**编译错误**，错误信息列出账本豁免名单机制 `// vm-safe-allow`）；icons 等既有 ext-stub 体系并表统一。
- **lint 脚本**：Node ESM，扫 `src/front/**/*.at`，输出五模式命中清单 + 非零退出码；豁免行 `// vm-safe-allow <原因>`。基线：当前源 = 1 P1（055 T0 修复后归零）/ 2 P3（T10 解除后归零）/ 0 P2 / 0 P4。

## 测试设计

- auto-lang 单测：每项根修配最小 .at 用例（新键赋值三态、Call 源迭代计数、标签完整性、裸字符码点、isArray/stringify/parse-array、编译期报错）。
- musk 门禁：四 case 文件（`tmp/vmprobe/case_*.at`）+ `vm-safe-lint.mjs` + `vm-first-run.mjs` + gen `pnpm build && pnpm vitest run` + `cargo test -p musk`。
- 实机对拍：musk-demo workspace Block 全家福会话，VM vs gen Vue 逐块 MCP snapshot + 截图（ThinkBlock/ToolBlock 7 卡/问卷交互），残差登记。

## 验收标准

见「目标」1–8；其中 1–4 为上游修复的机器判据，5 为用户可见判据，6 为门禁机制判据，7–8 为收尾判据。

## 执行步骤

> 布局：musk 侧任务在 `.worktrees/plan-057-dev`；auto-lang 侧任务在 `D:\autostack\auto-lang\.worktrees\auto-musk-dev`。前置：PLAN-055 T0 已落（临时规避就位，掩蔽雷点已处置）；PLAN-056 已归档。

- [✅ 已完成] **T1 VM-safe lint 门禁脚本**（musk）
  新建 `scripts/vm-safe-lint.mjs`：五模式静态扫描 + `// vm-safe-allow` 豁免 + 基线快照；`package.json` 不挂（独立执行）。
  验证：`node scripts/vm-safe-lint.mjs` 对当前源输出与账本⑥普查一致（055 T0 后 P1 归零、P3 两处豁免、P2/P4 零）。
  [✅ 已完成] worktree 57ceae1：零红+8 豁免；实测基线与普查口径有偏差（见待澄清 4）：P3 实为 4 处（+Math.trunc/Math.imul，探针实证 trunc(int) 32 位回绕、trunc(float)/imul 恒 None）、P4 实为 1 处（forge_helpers:141，普查称零）、P2 的 `(x ?? [])` 括号源实测迭代正常故收窄到真调用形态、另实证 map 点号新键同崩（relay_store:181 活雷，T2 根修后合法化）；auto build 绿。
- [✅ 已完成] **T2 SET_FIELD 新键插入**（auto-lang）
  `crates/auto-lang/src/vm/engine.rs:5030` ObjectData 臂按详细设计改造；新增三用例（空字面量加键/for-in 变量加键/typed instance 仍 Err）入 `musk_vm_track_tests` 或 vm 单测。
  验证：`cargo test -p auto-lang` 新用例绿；`node tmp/vmprobe/case_setfield_newkey.at` A–D 四 PASS。
  [✅ 已完成] auto-musk-dev 77c4a5306：末 else 报错→产出 Str 新键（obj.set=insert）；musk_vm_track_p057 4/4 绿（TDD 先红后绿）；case_setfield_newkey A–D 四 PASS（worktree 复编 auto 实跑）；实测钉正：typed instance 写侧=编译告警+静默跳过、错在读回——第三用例按可观测不变量改写（写不落键/读回中止）。
- [✅ 已完成] **T3 for-in Call 源通道泛化**（auto-lang）
  `crates/auto-lang/src/vm/codegen.rs:2650` 按详细设计泛化；用例：obj/list 注解返回 × 直接调用源 × 计数/求和。
  验证：`cargo test -p auto-lang`；`node tmp/vmprobe/case_forin_call.at` 5/5 PASS。
  [✅ 已完成] auto-musk-dev e01eeba0b：索引通道承接全部 Call 源；例外=迭代器协议链（iter/take/skip/rev/chain/zip，vm_types 编译形态测试钉通道）+stream/sse_ 惰性流（plan341 SSE 3 测试绿）+未知形态保守回退。p057_forin 3/3 绿（TDD）+case 5/5 PASS+setfield 回归 4/4。实测钉正：运行期 .iter() 本身零迭代（返回 length=0 迭代器对象）=既有独立债非本计划范围（待澄清1 按默认索引循环方案落地）。
- [✅ 已完成] **T4 实参含直接调用的串参数错乱修复**（auto-lang）
  与 T3 同族排查（参数求值/入栈序）；回归面=`case_web_builtins.at` A/B/C 标签完整性 + `c1_isarray` 形态。
  验证：`cargo test -p auto-lang`；case 文件 FAIL 行标签不再乱码。
  [✅ 已完成] auto-musk-dev cc0f43702：根因=CALL_SPEC 三处未知方法兜底（str/List/unknown 接收者臂）只压 None 不弹 receiver+实参（栈失衡 +1 平移参数槽）；未解析静态调用接收者=类型名字符串正落 str 臂。配平后 p057_arg 3/3 绿（TDD）+case A/B/C 标签完整（值级 FAIL 全归 T6）。实测钉正：用户 fn 调用作实参本就正确（此前普查 T2 形态系探针函数自身返回 None 的误报）。
- [✅ 已完成] **T5 for-in 裸字符接收者分派重定位**（auto-lang）
  定位 `engine.rs:7213` 恒等臂未命中的实际 CALL 路径并接通；用例：for-in 字符 `char_code_at` 码点序列。
  验证：`cargo test -p auto-lang`；`node tmp/vmprobe/case_str_charcode.at` 3/3 PASS。
  [✅ 已完成] auto-musk-dev bc5310b33：拦截者=CALL_SPEC `<unknown:` 接收者臂（整型字面量方法族）在恒等臂之前吞 None；恒等臂提升到该臂之前。p057_char 2/2 绿（TDD）+case 3/3 PASS+全族 12/12+三 case 无回归。
- [✅ 已完成] **T6 web 内建补 natives**（auto-lang）
  `crates/auto-lang/src/vm/ffi/stdlib.rs`（或对应 natives 表）：补 `Array.isArray`、`JSON.stringify`、`JSON.parse` 数组形态接线。
  验证：`node tmp/vmprobe/case_web_builtins.at` 8/8 PASS。
  [✅ 已完成] auto-musk-dev c7b67663b：四件 + census 漏计补遗 Math.trunc/Math.imul（待澄清⑤倾向落地，T7 连贯）。要点：canonical 双名（isArray camelCase 形此前漏注册——canonical 化保留方法名大小写）；stringify 固 arity+codegen 补参（Plan 197 先例）+vm_value_to_json 补 ObjectData 臂+键字典序；parse 数组=shim_str_len 堆句柄判定扩编码/变体；trunc int 恒等防回绕；is_static_method 白名单补 JS 全局名。p057 全族 18/18 绿+四 case 全绿（4/5/8/3）。
- [✅ 已完成] **T7 未解析 web 内建编译期报错**（auto-lang）
  a2vm codegen 收敛点对 natives 表外内建报编译错误；豁免机制与 ext-stub 体系统一；构造用例（含未实现内建的 .at 编译失败 + `// vm-safe-allow` 豁免通过）。
  验证：`cargo test -p auto-lang`；手工构造用例一红一绿。
  [✅ 已完成] auto-musk-dev aa733736c：收敛点=native 解析级联终局兜底；判别收窄到模块式全局调用（接收者字面∈四命名空间且非变量——实例方法 func_name 被类型重写带 Array. 前缀不误伤）；豁免=Codegen.source_text（管线注入）+current_source_line 回读 `// vm-safe-allow` 行（待澄清2 落定：与 musk lint 同机制双轨，非 ext-stub 并表——icons 体系是渲染 stub，语义门禁走源码注记更直接）。p057_compile_gate 4/4（一红一绿+双控制组）+全族 22/22+musk_vm_track 47/47。
- [✅ 已完成] **T8 图标桥扩容**（auto-lang）
  从 VM 实机 ext-stub 警告清单取卡片用图标名单（ChevronDown/Copy/Check/Plus/Trash2/Search/Info/Send/FileText/Download/FolderOpen 等），补 `lib.rs` 装载名单/注册表（050 C5 机制）。
  验证：VM 实机快照聊天+卡片视图无 `[Image]` 空图标；ext-stub 警告清零或仅余登记豁免。
  [✅ 已完成] auto-musk-dev 26fb2675f：勘察结论=扩容已由 054 R2 等会话在上游完成（musk icons.web.at 44 项全部命中 lucide_svg 字形表，账本「renderer 27 glyph」数字过时），本计划零字形新增；补 p057_musk_icon_bridge_full_coverage 全量锁定测试（44 名单×kebab×字形表命中，防新增图标漏同步）。实机 ext-stub 警告清零验证归 T11。
- [✅ 已完成] **T9 auto-lang 门禁 + 合回**（auto-lang）
  `cargo test -p auto-lang --lib` 基线差分零新增；四 case 文件全绿；按 AGENTS.md 合回 auto-lang master、清理 worktree；musk 侧复编/复链验证消费。
  验证：master 提交 + musk 侧 `auto run --render=vm` 冒烟绿。
  [✅ 已完成] auto-lang master 016e95df7（8 提交：T2-T8+门禁回归修复；分支先 merge master 同步 517 并行推进再快进合回）。门禁回归一笔当场修复：T3 泛化漏生成器（for-in 生成器调用源被索引通道劫走，generator_tests 5 红，master 对照坐实）→ callee∈generator_fns 保留迭代器通道（6574a319a）。全量 --lib **3383 绿 0 败**（96 ignored）+四 case 全绿（worktree 与 PATH 重建二进制双验证）+musk 消费 auto build 绿+lint PASS。worktree/分支已清理。注：auto-lang 主检出有 517 会话未提交改动（442/KNOWN-DEBT/examples/scratch），合并未触及、未处置。
- [✅ 已完成] **T10 解除 055 T0 临时规避**（musk）
  `forge_helpers.at:85` `toolArgsJson` 恢复 `JSON.stringify` 直通、`questionnaire_helpers.at:76` 恢复 `Array.isArray` 分支；`messageBlocks` 字面量重建保留（注释注明缘由）；`// vm-safe-allow` 豁免行清数归零核对。
  验证：实机全家福会话工具卡参数非空、问卷多选摘要渲染；`node scripts/vm-safe-lint.mjs` 零红。
  [✅ 已完成] worktree ba16066：直通/isArray 双恢复+messageBlocks 保留（待澄清③倾向落定）+豁免清 5 留 3（census 判定类）+lint 白名单/模式语义随根修收口。门禁：auto build 绿+lint 五模式零红+vitest 23+1skip（基线一致；vitest 2.x 会话级补装）。实机验收（工具卡参数非空/问卷多选渲染）并入 T11 矩阵执行。另：执行期 055 被并行会话复审合入 main（39a34ef，think_block 孤儿清理），T10 前 merge main 同步基线。
- [✅ 已完成] **T11 Block 逐类对拍矩阵**（musk）
  ThinkBlock、ToolBlock→{GenericToolCard/ErrandCard/GateCard/TaskPlanCard/RelayRunBox/ReportCard/QuestionnaireCard}、问卷交互，VM vs gen Vue MCP snapshot+截图逐类对拍；语义缺失修 auto-lang/musk，纯样式降级登记账本。
  验证：attachments 对拍证据 ≥9 类；残差全部有账本行。
  [✅ 已完成] 实机环境：musk serve :9092（plan057-demo 隔离工作区）+worktree VM（MCP :9272）+gen dev :3341（浏览器 IAB）。对拍结论：ThinkBlock/文本块/工具卡×5/Run 窗口/报告卡=VM 渲染一致（gen DOM 快照全类取证+VM 快照/截图×4）；**执行中修三件**——嵌套 stringify 裸句柄（auto-lang cb6e8a6ef）、卡片框架双轨注记（4dcec22，六类卡根补工具类——原始 CSS 为 Vue 专属直通）、问卷检测 Regex 解除（11b6c20，indexOf 纯串通道）；问卷卡被新发现 pre-existing 缺陷阻断（__json_object 字符串字段读污染，pre-057 二进制复现）。残差七笔全入账本 057 行（含 VM 合成输入守卫致视口下块自动化受限——用户实机实测补充：点击展开+滚动目验）。证据：attachments/p057-t11-vm-{overview,framed-cards,framed-max}.png + 用户实机截图两帧 + gen 截图。
- [✅ 已完成] **T12 全量门禁 + 账本回写**（musk）
  `cargo test -p musk`、gen `pnpm build && pnpm vitest run`、`node scripts/vm-first-run.mjs`、四 case 全绿、lint 零红；KNOWN-DEBT「等价性缺陷族」行改写为已修注记（保留 markdown/autodown 缓期项）；worktree 合回+清理。
  验证：全绿 + 账本 diff 自洽 + 无悬挂 worktree。
  [✅ 已完成] 48a6cae：musk cargo 617 绿 0 败（5 ignored 基线；backend 需 .worktrees/{auto-ai,auto-lang} junction 补路径依赖）+auto build 绿+vitest 23+1skip（会话级 vitest 2.x 补装；auto build 再生成抹 package.json devDeps 需重装）+vm-first-run alive reds=0+四 case 全绿（4/5/8/3）+lint 五模式零红/3 豁免；账本缺陷族行①–⑤改已修注记+新增 057 行（T11 残差七笔）。worktree 按 /auto-plan:merge 流程保留待复审（阶段成果已 merge main：框注记/问卷通道/账本/探针全在）。

## 复审记录

**复审人**：zhaopuming（ZCode 会话）｜**时间**：2026-09-03 02:40｜**入口状态**：execution_done → **reviewed（通过，目标 5 部分收敛在案）**

**逐项验收复验（全部重跑，不信任勾选框）**：

| # | 判据 | 结论 | 证据 |
|---|---|---|---|
| 1 | case_setfield_newkey A–D | **PASS** | 复跑 4 PASS（PATH 二进制 auto） |
| 2 | case_forin_call 5/5 | **PASS** | 复跑 5 PASS |
| 3 | case_web_builtins 8/8 | **PASS** | 复跑 8 PASS |
| 4 | case_str_charcode 3/3 | **PASS** | 复跑 3 PASS |
| 5 | 实机全家福：ThinkBlock+文本+7 工具卡/参数非空/问卷多选摘要 | **部分收敛** | ThinkBlock+文本+工具卡+框架=实机截图×3+用户实机目验（用户反馈驱动框修复并确认内容渲染）；参数 JSON=嵌套句柄修复+case F；**问卷卡不渲染**——被值打标污染族层层阻断（见下） |
| 6 | 未实现内建编译期报错 | **PASS** | 构造用例一红一绿复跑（gate_red 编译错含豁免提示/gate_green 输出 NONE） |
| 7 | 图标桥扩容+ext-stub 口径 | **PASS** | 44 图标全命中字形表（p057_musk_icon_bridge_full_coverage）+实机图标渲染视觉证据；54 警告=44 图标桥假警报+10 web 专属平台函数，登记豁免（账本 057 行⑤） |
| 8 | 两仓门禁零新增红+合回 | **PASS** | auto-lang 全量 --lib **3386 绿 0 败**（复审时点重跑）；musk cargo **617 绿 0 败**（复审重跑，先前"5 failed"系 awk 字段误切已纠）；vitest 23+1skip；first-run alive reds=0；lint 零红；auto-lang 已合回清理、musk 阶段折叠 main |

**遗漏/延后/workaround 猎捕**：
- **workaround（已登记）**：卡片框架双轨注记（原始 CSS 为 Vue 专属直通——native_css_tests 明文，工具类注记为正解非债）；messageBlocks 字面量重建保留（待澄清③用户在场倾向）；问卷 Regex→indexOf 通道重写（Regex 族根修欠账，账本 057 行②）。
- **延后（用户在场知情的分拆）**：问卷卡渲染——T11 执行中已向用户实时报告受阻于新发现缺陷，用户随后主动发起复审=知情接受分拆。**复审勘察将堵点根因精确到三层**（均 pre-existing，b68ce46fb 基点二进制复现）：①`indexOf(sub, from)` 双参形态返回 0（wl_probe24）；②特定 fn 形态（≥3 参/参重赋值）下 indexOf 返回**字符串标签的整数**（wl_probe29："5"+"13"="513"）；③含转义引号字面量经 substring 后 indexOf 返回值再次污染（wl_probe36："2713"）。另 `__json_object` 字符串字段 print 空但 EQ 正常（wl_probe22）。全族归 **VM 值打标体系**（055-3 ① 同族深化实证）——建议独立立项根修，解锁问卷卡+print 空+标签污染。
- **遗漏**：无（逐任务对照 diff，25 文件 +1177/-116）。

**门禁注意项（复审发现）**：musk backend 自 worktree 构建需 `.worktrees/{auto-ai,auto-lang}` junction（已建）；vitest 为会话级补装且 `auto build` 再生成会抹 package.json devDeps。

**路由**：通过——目标 1–4/6/7/8 全过，目标 5 部分收敛（问卷部分受阻于计划外 pre-existing 缺陷族，用户知情、账本 057 行在案）。**后续建议**：值打标体系根修独立立项（问卷卡/字符串字段 print/标签污染一并解锁）；markdown/autodown 富渲染维持既有缓期路线。

## 待澄清事项

1. E5b 泛化采用"索引循环"还是"运行时真迭代器"——大列表性能实测后定（默认前者，改动面小）。
2. 编译期报错门禁与既有 ext-stub 警告体系（icons）的统一方式——并表还是双轨并存，T7 勘察后定。
3. `messageBlocks` 字面量重建在上游修复后是否回退 `raw.status=` 形态——倾向保留（双轨行为可预测），复审时定。
4. **（T1 实证）账本⑥普查口径偏差**：P3 实为 4 处而非两处——`Math.trunc`（forge_store:532 nowSec，探针实证 trunc(int) 32 位回绕成 -2147483647、trunc(float) 恒 None，与 census「✓Math 系」印象相反）、`Math.imul`（forge_helpers:143，恒 None，avatar 色相 hash 退化）；P4 实为 1 处（forge_helpers:141 `ch.char_code_at`，普查称零命中）；另实证 `var body map` 后点号新键同样 RuntimeError（relay_store:181 活雷：VM 带 feedback 的 gate 审批崩，T2 根修后自动合法化）。已全部 vm-safe-allow 豁免留痕。
5. **（由 4 派生，T6/T7 前需定）** T7 编译期门禁若按「natives 表外 web 内建即报错」实现，将在 nowSec 的 `Math.trunc` 与 avatar hash 的 `Math.imul` 处炸 musk VM 构建。倾向：T6 顺带补这两个 natives（小改，trunc 双臂 + imul i32 回绕语义）；否则这两处需沿用 vm-safe-allow 豁免编译门禁。执行到 T6 时按此倾向处理，复审时复核。
6. **VM int 算术 32 位回绕根修是否并入本计划**——055-4 账本登记「同族归 PLAN-057」（t1_datefmt 实锤 `ts*1000` 回绕坏、字面量传 native 保宽好），但本计划任务清单未列。未定夺前按已登记债处置（nowSec 已用 int 除规避乘法；msgTimeLabel 残项维持）。
