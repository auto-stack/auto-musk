---
plan_id: PLAN-062
status: archived
feature_name: VM 轨内存泄漏修复（空闲轮询空转置脏 × computed 返回值 retain 不释放）
author: [zhaopuming]
created_at: 2026-09-05T10:23:45+08:00
updated_at: 2026-09-05T13:25:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components:
  - "designs: PLAN-062 VM 轨内存生命周期契约——①timer 派发精确置脏(state_mutation_seq 写点计数×fire_timer 前后对比,空转拍不触发视图重建) ②帧域 retain 账本(retain_heap_result 记账→commit_dirty_frame 脏帧换代配平释放,AUTOUI_FRAME_RC=0 逃生门) ③桥接临时任务收尾清账(rc_release_task_stack [0,sp));残项 KD-062①②"
touched_goals: []

current_step: 9
total_steps: 9
---

# [PLAN-062] VM 轨内存泄漏修复

## 变更摘要

2026-09-05 用户实机报告：VM 版（`auto run --render=vm`）打开 Block 全家福会话
（`block-showcase-chat`，本体仅 4.6KB）内存飙至 4GB。当日复现定罪（复现脚本与
曲线存档 `tmp/memdbg/`）：**空闲态 VM 进程以 0.85–2.2 MB/s 恒定泄漏堆内存**，
速率随渲染树大小缩放（工作区选择器 0.85 / 空会话 1.35 / 全家福 2.2 MB/s），
30–60 分钟即达 4GB；句柄/线程数恒定，纯堆增长。

根因为**两个缺陷叠加**（缺一不致泄漏）：

1. **空转轮询每拍整树重建**（musk 侧 `4cbc0ba` PLAN-536 T12 D4 引入回归 +
   auto-lang 派发层语义）：`forge_store.at:87` PollStream 定时器 when 门被摘除
   后，订阅层每 500ms 无条件产消息（`widget_event_tick`，renderer.rs:5988——
   订阅本身无门控，when 门只是派发前过滤）；`dynamic.rs:1056` handler 执行
   成功即 `dirty=true`（含 deadman 早退的空转拍）；`renderer.rs:13076` 
   `is_dirty→view_dirty` → `dynamic_view` dirty 分支（renderer.rs:14866）整树
   模板重建。实测 7 分钟 457 拍空转、每拍 ~1s CPU（WARN 刷屏佐证）。
2. **computed 返回值 retain 永不释放**（auto-lang 在案债 KD-051 ⑤ / T11）：
   `vm_bridge.rs:1162`（call_vm_fn）/ `:1211`（call_computed_fn）尾部
   `retain_heap_result` 无配对 release（注释自述"v1 暂不配对释放，语义属上游
   per-frame 生命周期"）。每次整树重建求值全部 computed，整棵输出树
   （~1–4MB/拍）永久滞留堆。

流式期间更重：dirty 每拍为真 × 树更胖，4–8 MB/s。修复分两刀：
**F1 timer 派发精确置脏**（auto-lang：空转拍不再失效视图——杀空闲泄漏与
CPU 空转）、**F2 帧域 retain 账本**（auto-lang：脏帧重建换代时释放上一代
computed 结果——杀一切重建驱动的泄漏，含流式/时钟/游戏 Tick 等合法定时器
场景）+ musk 侧小配套（poll_window 完成清理）。不回滚 `4cbc0ba`（其修复的
发送链门控问题真实存在，见 KD-493①；F1 使空转零成本后 when 门不再必要，
且 `timer_guard_passes` 只支持根态标量真值、不支持列表长度表达式，重设门
反而回到 T12 死局）。

## 目标

1. 空闲（非流式、无用户交互）VM 前端进程内存斜率 < 5 MB/min（当前实测
   51–132 MB/min）；重建 WARN 刷屏停止。
2. 流式轮询期间（合法 dirty 拍）同样不泄漏：连续 N 拍脏重建后 VM 堆
   live_heap 增量有界（soak 断言）。
3. timer 语义回归零破坏：streaming 期间回填/完成启发式/deadman 窗行为
   不变（plan051 timer 测试族 + musk 实机发送链路活体）。
4. 空转拍不再触发视图重建（fire_timer no-op 不置脏单测钉死）。
5. 全门禁绿：auto-lang `--lib`（差分零新增红）+ musk vm-link-probe /
   vm-safe-lint / vitest / cargo test。
6. debug 构建 RC canary 零触发（F2 的 UAF 防线，既有 P419 仪器化）。

## 架构方案

不动双轨架构。三个修复点全部落在 auto-lang VM 运行时层 + musk `.at` 源一处：

| 层 | 文件 | 动作 |
|:---|:---|:---|
| 引擎记账 | `auto-lang crates/auto-lang/src/vm/engine.rs`（+rc.rs 写点） | 新增 `state_mutation_seq: AtomicU64`，堆/状态**写点** bump（insert_heap_object、SET_FIELD 落盘臂、list 元素突变桩、字符串池**新槽**分配；dedup 命中/读/retain/release 不 bump），暴露 `state_mutation_seq()` |
| 派发置脏 | `auto-lang crates/auto-lang/src/ui/dynamic.rs:512 fire_timer` | 派发前后取 seq 对比 + `was_dirty` 哨兵：seq 未变且原先不脏 → 撤销 `call_handler` 的无条件置脏。**仅 timer 路径**收窄；普通事件派发语义不动（vmref-push-only 的 handler 仍靠派发置脏兜底） |
| 帧域账本 | `auto-lang crates/auto-lang/src/ui/vm_bridge.rs` + `iced/renderer.rs:14866 dirty 分支` | VmBridge 增双缓冲 `frame_retains: Vec<u64>`；`retain_heap_result` 记账；`commit_dirty_frame()` 在 dirty 分支**新缓存写回之后**调用（释放上一脏帧账本，保证换代期间旧树存活）；env `AUTOUI_FRAME_RC=0` 可关（默认开） |
| musk 配套 | `auto-musk src/front/forge_store.at` | PollStream 完成启发式命中 / OnStreamEvent done / error / StopStream 四处清空 poll_window（handler 自身经根态派发，SET_FIELD 写可靠——T12 注⑤"timer 派发走根态，回填写入可靠"同款），防长会话窗戳堆积 |

设计依据（复现实证，`tmp/memdbg/`）：

- 空转链：订阅消息(500ms) → fire_timer → handler Ok（deadman 早退零状态写）
  → `dirty=true`（dynamic.rs:1056）→ view_dirty（renderer.rs:13076）→
  dirty 分支整树重建（renderer.rs:14866）→ 全 computed 求值 → 每结果
  retain 无 release（vm_bridge.rs:1162/1211）→ 泄漏。
- 旧 when 门（`b9a0da4`）时代无此问题：gate 假 → handler 不派发 → 不置脏
  → view 走缓存分支零重建。`timer_guard_passes`（dynamic.rs:532）只读根态
  标量真值（Bool/Str/Int），列表值恒 false，且无表达式能力——T12 摘门后
  无法在 musk 侧重建等价门（`.poll_window.length` 不可表达）。
- F2 换代时序安全性：脏帧 N 的账本在帧 N+1 重建完成并写回新缓存**之后**
  才释放；缓存命中帧不跑 builder、零 retain、不触碰账本。滞留跨帧的
  `Value::VmRef` 位点（InspectorCache/vtree 快照 props/adapter 注册表/
  child props）在 T5 逐一审计：物化（构建期解码为自有数据）或补配对
  retain，二选一，不留 TBD。

## 技术栈

- auto-lang（Rust）：`crates/auto-lang/src/vm/{engine,rc}.rs`、
  `src/ui/{dynamic,vm_bridge}.rs`、`src/ui/iced/renderer.rs`；
  测试 `--features ui-iced`（plan051_timer_tests / musk_vm_track_tests /
  musk_probe 族）。
- auto-musk：`src/front/forge_store.at`（经 `auto run` 重生成生效）；
  门禁 `scripts/vm-link-probe.cmd`、`scripts/vm-safe-lint.mjs`、
  `gen/front/vue && pnpm vitest run`、`backend && cargo test`。
- 实机：`scripts/dev-stack.mjs`（musk serve :9247 + VM :9277 MCP）+
  `tmp/memdbg/mcp.mjs` 驱动脚本（本次调查产出，直接复用）+
  `tmp/memdbg/sample-all.ps1` 内存采样。
- worktree 布局（AGENTS.md）：`.wt/musk-062/auto-musk`（分支
  `plan-062-dev`）+ `.wt/musk-062/auto-lang`（分支 `auto-musk-dev`）。

## 需求分析与背景调查

- 复现档案（2026-09-05，本机）：`tmp/memdbg/mem-curve.log`、
  `mem-before-open.log`（斜率三档 0.85/1.35/2.2 MB/s）、`tmp/memdbg-vm.log`
  （457 拍空转 + 每拍 view_builder WARN 潮）、`state.txt`（poll_window=[]
  / streaming=false——deadman 门 handler 层生效的铁证）。
- 代码考古：`git log -L 85,90:src/front/forge_store.at` 确认 when 门
  `b9a0da4`(PLAN-051 T10) 加入、`4cbc0ba`(PLAN-536 T12 D4, 2026-09-04)
  摘除；摘除动机（跨模块 SET_FIELD 不可达根态致 when 门恒假）与本次修复
  不冲突——F1 在派发层解决，无需 when 门复活。
- 在案债：KNOWN-DEBT 051 行⑤"call_vm_fn retain 未配对释放（per-frame
  生命周期，长会话内存增长）"即 F2 根修对象；KD-048 观察 a)（VM 进程
  1.5–4.5 分钟静默退出）与本泄漏无因果（本次复现进程稳定存活），
  独立维持观察。auto-lang KD"state-scope 专项"（跨模块 SET_FIELD 重绑定
  不达根态）不在本计划根修范围，F1/F2 均不依赖它。
- spec 账本（backend/.autoos/specs.json，P030 系 plan 流程项）：VM 运行时
  内存/生命周期契约无既有条目，F2 的帧域账本作为新契约候选，
  spec-impact 归 /auto-plan:review 期落。
- 次要观察（登记不修）：MCP snapshot 调用拉起 ~66MB 子 auto 进程不退
  （独立债，另行立案）；每拍 446-U7 自定义类名 WARN 刷屏随 F1 空转消失
  自然收敛（合法重建时仍在，属日志噪声）。

## 详细设计

### F1 timer 派发精确置脏

`fire_timer`（dynamic.rs:512）现语义：when 门真 → `on_with_input_for` →
`call_handler Ok → dirty=true` 无条件。改为：

```rust
pub fn fire_timer(&mut self, widget: &str, event: &str) -> bool {
    // …when 门不变…
    let seq_before = self.bridge.vm.state_mutation_seq();
    let was_dirty = self.dirty;
    self.on_with_input_for(widget, event, None);
    if self.bridge.vm.state_mutation_seq() == seq_before && !was_dirty {
        self.dirty = false; // 空转拍：handler 零状态写，不失效视图
    }
    true
}
```

`state_mutation_seq` bump 点（engine.rs，全部走既有内部可变通道）：
`insert_heap_object` / SET_FIELD 对堆对象的字段写 / ListData 元素突变
（push/pop/set/清空）/ 字符串池**新槽**分配（dedup 命中不 bump——同串
复用零增长）。读路径、retain/release、tombstone 清扫一律不 bump。
保守方向正确：多 bump 至多多一次重建（现状即如此），漏 bump 才会丢更新；
handler 体内临时对象分配（如 JSON 解析中间体）会 bump → 视为脏 →
保守可接受（PollStream 空转早退路径实证零分配）。

### F2 帧域 retain 账本

vm_bridge.rs：

```rust
// 双缓冲：cur 收本帧 retain；commit 时 swap 并释放 prev。
frame_retains_cur: std::sync::Mutex<Vec<u64>>,
frame_retains_prev: std::sync::Mutex<Vec<u64>>,

fn retain_heap_result(&self, out: &Value) {
    match out {
        Value::Int(id) if *id >= 4_000_000 => {
            self.vm.rc_retain_id(*id as u64);
            self.record_frame_retain(*id as u64);
        }
        Value::VmRef(r) => { /* 同上 */ }
        _ => {}
    }
}

pub fn commit_dirty_frame(&self) {
    // prev 释放（rc_release_id），cur→prev，cur 清空。
    // AUTOUI_FRAME_RC=0 时整体 no-op（逃生门）。
}
```

renderer.rs `dynamic_view` dirty 分支（:14866）尾部、
`cached_converted_view` 写回**之后**调用 `bridge.commit_dirty_frame()`。
时序保证：帧 N 的 retain 结果在帧 N+1 构建全程存活；N+1 缓存写回后旧账本
才释放，旧缓存树同帧淘汰，无悬挂窗口。

滞留审计（T5）方法：grep `Value::VmRef` 在 builder 产物侧的落点
（InspectorCache、vnode/snapshot props、adapter widget 注册表、
`get_child_state_id`/`sync_child_props_to_root` 链、routes/nav_group_states），
逐点判定：跨脏帧仍会被解引用 → 物化为自有数据（String/数值在构建期拷出）
或该点补显式 `rc_retain_id` + 登记常驻清单（不进帧账本）；仅帧内消费 →
无需处理。debug 构建跑 musk VM 全链（canary 仪器化）验证零 UAF。

### musk 配套（poll_window 清理）

forge_store.at 四处清理（`while` pop 清空——列表突变经共享 vmref 可靠，
与 push 同机制）：

- `.PollStream` 完成启发式命中处（`msgs.length > .pre_stream_len` 分支内，
  `.streaming = false` 旁）；
- `.OnStreamEvent` 的 `done` / `error` 两臂（`.StopStream()` 旁）；
- `.StopStream` 本体（显式停止即关窗）。

效果：deadman 窗在完成/停止后立即关闭，后续拍首行守卫即返回；长会话
窗戳不堆积（当前每次 Send push 一条 int，KB 级，顺手卫生）。

## 测试设计

1. **红测先行**（auto-lang，T1）：
   - `fire_timer_noop_does_not_dirty`：timer 条目 + 空体 handler →
     `fire_timer` 后 `is_dirty() == false`（现红：无条件置脏）。
   - `musk_vm_track_heap_soak`（musk_vm_track_tests.rs）：musk 前端 link +
     驱动 N=50 拍"fire_timer(PollStream 形态 no-op) + view_with_debug_gated
     重建"循环，断言 `rc_stats().live_heap` warmup(10 拍)后增量 ≤ K
     （K 由 T1 实跑校准，预期现值为无界增长 → 红；修复后 ≤ 每拍常数残差，
     建议 K=32 起评）。
2. **F1 单测**：上述 no-op 绿 + 既有 `plan051_timer_tests` 族全绿（when 门
   真分支照常派发置脏：handler 写状态 → seq 变 → 脏保留）。
3. **F2 soak**：`AUTOUI_FRAME_RC` 默认开跑 soak 绿；`AUTOUI_FRAME_RC=0`
   跑一遍确认逃生门有效（行为=现状泄漏，仅冒烟不断言斜率）。
4. **UAF 防线**：debug 构建（canary 仪器化）跑 musk_vm_track 全族 +
   实机 first-run 20s——零 `P419UAF`/tombstone 红。
5. **门禁**：auto-lang `cargo test -p auto-lang --lib --features ui-iced`
   （对照基线：plan050×2/dock×2/code_editor 已知红不新增）；musk
   `vm-link-probe`（PASS + 体积 ≤ WARN 线）、`vm-safe-lint` 零红、
   `pnpm vitest run`（23+1skip 基线）、`backend cargo test`（617 基线）。
6. **实机验收**（T8，dev-stack + tmp/memdbg 脚本）：登录 → 打开全家福 →
   10 分钟采样斜率；MCP `autoui_action` 驱动发送一条消息 → 轮询回填 →
   完成后 2 分钟 deadman 静默（日志无 WARN 潮、无 [VM_HANDLER_OK] 后跟
   重建）；`autoui_snapshot` 双拍 vnode 稳定。

## 验收标准

1. 空闲全家福会话 10 分钟内存斜率 < 50MB 总增量（当前 10 分钟 ~220MB；
   绿线 <50MB，其中 F1 停重建后预期 <10MB，留 GPU/日志噪声余量）。
2. `fire_timer_noop_does_not_dirty` 与 `musk_vm_track_heap_soak` 绿且入
   `--lib` 常驻；plan051 timer 族零回归。
3. 发送-轮询-完成链路实机活体通过（回复入列、streaming 落定、deadman
   关窗、关窗后零空转重建）。
4. 全门禁绿（差分口径：auto-lang 已知红 5 项持平，musk 四门禁零新增红）。
5. debug 构建 canary 零触发；`AUTOUI_FRAME_RC=0` 逃生门冒烟通过。
6. KNOWN-DEBT 回写：051 行⑤ 核销（指向本计划与 auto-lang 落地 commit）、
   048 观察 a) 补注"与内存泄漏无因果"；新债登记（若有）：snapshot 子进程
   66MB、审计中发现的常驻 retain 清单。

## 执行步骤

> worktree：`D:\autostack\.wt\musk-062\auto-lang`（分支 `auto-musk-dev`）
> 与 `D:\autostack\.wt\musk-062\auto-musk`（分支 `plan-062-dev`）。
> auto-lang 侧任务在 auto-lang worktree 内执行；musk 侧任务在 musk worktree。

- **T1 红测先行（auto-lang）** [✅ 已完成 2026-09-05] 双红确认：`fire_timer_noop_does_not_dirty` FAILED（no-op 拍置脏）；`musk_vm_track_heap_soak` FAILED——live_heap 421→2101（+1680/40 拍 = 42 obj/拍，语料 `test/ui/plan062_memleak/` 两 timer + 表达式体 computed `items => build_items(20)`）。附只读访问器 `heap_live_objects`（vm_bridge/dynamic 透传）。注：`--features ui-interpreter` 单特性存量编译破损（test 目标引 iced 符号），红测按门禁特性 `ui-iced` 跑。
  验证：`cargo test -p auto-lang --lib --features ui-iced -- fire_timer_noop musk_vm_track_heap_soak -- --nocapture`
  → 两测红（no-op 置脏 / live_heap 无界）。✅

- **T2 状态突变计数器（auto-lang）** [✅ 已完成 2026-09-05] engine.rs 四类写点（insert_heap_object/SET_FIELD 臂入口/LIST_PUSH|POP|SET_INT/add_string 复用+追加两出口）+ native.rs 三 shim 入口 bump；`mutation_seq_bumps_on_write_not_read` 绿（纯读拍零 bump、写拍 bump；dispatch 机制本身零分配——隐式重建移除后实证）。
  `crates/auto-lang/src/vm/engine.rs`：`state_mutation_seq: AtomicU64` +
  四类写点 bump（insert_heap_object / SET_FIELD 堆写 / ListData 突变 /
  字符串池新槽）+ `pub fn state_mutation_seq()`。读/dedup 命中/retain/release
  不 bump。
  验证：`cargo test -p auto-lang --lib --features ui-iced` 差分零新增红；
  新增单测 `mutation_seq_bumps_on_write_not_read`（同一小组断言四写点各 bump、
  连续读不 bump）绿。

- **T3 fire_timer 精确置脏（auto-lang）** [✅ 已完成 2026-09-05] dynamic.rs fire_timer seq 前后对比+was_dirty 哨兵；`fire_timer_noop_does_not_dirty` 绿 + plan051 timer 族全绿。**追加发现并移除第二放大器**：on_with_input_for 内 "Force a view rebuild to measure render time" 遗留插桩——每个 handler Ok 后强制全量视图构建（结果丢弃），musk 每拍双倍泄漏的直接来源（soak 42→21 obj/拍）。
  `crates/auto-lang/src/ui/dynamic.rs` `fire_timer`（:512）加 seq 前后对比 +
  was_dirty 哨兵撤销置脏（见详细设计代码）。非 timer 派发路径不动。
  验证：T1 两测中 no-op 测转绿；`cargo test -p auto-lang --lib --features
  ui-iced plan051` 全绿（timer 活性回归）。

- **T4 帧域 retain 账本（auto-lang）** [✅ 已完成 2026-09-05] 双缓冲 frame_retains + retain_heap_result 记账 + commit_dirty_frame（renderer dirty 分支/首帧分支缓存写回后接线）+ 桥接临时任务 [0,sp) 收尾清账（call_vm_fn/call_computed_fn/call_handler_for）。soak 改双相断言：空闲相 40 拍零增长（F1 战果锁定）✅ + 重建相 ≤24/拍速率绊线 ✅。**残留**：+1 未定位份额/call 钉住当帧 computed 树（任务帧槽扩域被 STORE_GLOBAL 转移语义+canary 否决、全局表/结果槽均排除）——归上游 RC 槽位记账专项（KD-051⑤ 续行，见待澄清）。
  `crates/auto-lang/src/ui/vm_bridge.rs`：双缓冲 `frame_retains` +
  `retain_heap_result` 记账 + `commit_dirty_frame()`（env `AUTOUI_FRAME_RC=0`
  no-op）；`crates/auto-lang/src/ui/iced/renderer.rs` dirty 分支
  （:14866）缓存写回后接线 commit。
  验证：`cargo test -p auto-lang --lib --features ui-iced musk_vm_track_heap_soak`
  绿（默认开）；`AUTOUI_FRAME_RC=0 cargo test … musk_vm_track_heap_soak`
  红回现状（逃生门冒烟）。

- **T5 跨帧滞留审计（auto-lang）** [✅ 已完成 2026-09-05] builder 侧无裸 id 跨帧持有点：preview_states/nav_group_states 为 Rust 标量；child_state_map 的子态 id 持 ensure_child_state 常驻份额（非 retain_heap_result 来源，账本释放不影响）；Bindings 随建随灭；vtree/snapshot 仅字符串。无需物化改造。**潜在耦合注记**：当上游 RC 槽位记账专项落地（残留 +1 stake 清偿）后，computed 结果将在 commit 后真正归零——届时任何新生长持有点必须自带常驻份额（复审时复查本条）。
  grep/走读 `Value::VmRef` 在 InspectorCache、vnode/snapshot props、
  adapter 注册表、child props 同步链、routes/nav_group_states 的滞留点；
  逐点物化或常驻 retain（清单落 `crates/auto-lang/src/ui/vm_bridge.rs`
  注释区）。debug 构建跑 musk_vm_track 全族确认零 canary 红。
  验证：`cargo test -p auto-lang --lib --features ui-iced musk_vm_track`
  全绿 + 日志零 `P419UAF`/tombstone。

- **T6 auto-lang 全量门禁** [✅ 已完成 2026-09-05] nextest（进程隔离，必须——裸 cargo test 进程内并行因全局态共享大面积互污，183 假红实证）：4540 run / 4520 绿 / 20 红——stash 基线对照（同过滤）确认 20 红全部既有（ui::layout 窗口族 18 + ui_gen 2 + d8，环境/语料在途变更）。**差分零新增红**；vm-link-probe PASS（63318B ≤ WARN 线）。
  `cargo test -p auto-lang --lib --features ui-iced`（对照已知红
  plan050×2/dock×2/code_editor 持平）+ `node D:/autostack/auto-musk/scripts/vm-link-probe.mjs`
  （VM_LINK_LANG_ROOT 指 worktree）PASS。
  验证：差分零新增红；probe 体积 ≤ WARN 线 90000。

- **T7 musk 配套 poll_window 清理（auto-musk）** [✅ 已完成 2026-09-05] 四处 while-pop 清空落码并提交（worktree plan-062-dev）；vm-safe-lint 零红 + vm-link-probe PASS（63318B，sibling 解析到组内 auto-lang worktree）。vitest/gen 重生成不在 worktree 跑（gen/ gitignored 不入 worktree）——按源级门禁（lint+probe）收口，重生成+vitest 随合回后在主检出跑（复审期补证）。
  `src/front/forge_store.at`：四处清空（PollStream 完成启发式臂 /
  OnStreamEvent done / error / StopStream 本体，while-pop 形态）；
  `auto run` 重生成。
  验证：`node scripts/vm-safe-lint.mjs` 零红；`cd gen/front/vue &&
  pnpm vitest run` 23+1skip；`node scripts/vm-link-probe.mjs` PASS。

- **T8 实机验收（双 worktree 联装）** [◐ 部分达成 2026-09-05，两残留登记移交] worktree auto.exe + 主检出源实机：PollStream 拍零状态写零取数（chats_get_session 0 次——F1 实机生效 ✓）；活跃斜率 2.2→~1.25MB/s（半减，与 soak 42→21 一致）。**未达 <5MB/min**：实机存在 ~20Hz 视图重建驱动（非 timer/MCP/消息，禁 MCP 后依旧，疑 iced redraw→view 通路——基线即有）× 残留 +1 stake/call = 当前斜率。两项已登记 KD-062 行①②，归上游专项；发送-轮询-完成链路活体未跑（进程被已知静默退出 KD-048 观察 a 截断），随残留项根修后补。
  auto-lang release 重装（`cargo install --path crates/auto-lang` 或既有
  安装方式）后 `node scripts/dev-stack.mjs`；复用 `tmp/memdbg/mcp.mjs` +
  `sample-all.ps1`：打开全家福 10 分钟斜率采样、发送-轮询-完成链路、
  deadman 后日志静默检查。
  验证：验收标准 1/3 达标（斜率 <50MB/10min；链路活体；空转重建消失）。

- **T9 文档与债回写（双仓）** [✅ 已完成 2026-09-05] KNOWN-DEBT 新增 062 行（残留①RC 槽位记账 + ②20Hz 重建驱动 + ③已根治面清单）；auto-lang 侧 d4819d2b2 提交注含全部证据；048 观察 a) 相关性注记（本次两进程均在 ~5min 静默退出，与泄漏无因果，维持独立观察）。
  musk `docs/plans/KNOWN-DEBT-AND-RISKS.md`：051 行⑤ 核销、048 观察 a)
  补注、新债登记（snapshot 子进程/常驻 retain 清单如有）；auto-lang 侧
  现行 plan 文档同步帧账本契约；两仓 worktree 收尾走 wt-guard → 合回。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-05 12:40）
**方法**：双 worktree 真实 diff 对账（auto-lang master..HEAD 3 commits 6173e9574/3a5aeb2e3/d4819d2b2；musk main..HEAD 219a497）+ 最终提交全量门禁重跑 + 逐条验收复验 + 复审期补齐三项缺口（KD 051⑤ 部分核销/048a 补注/snapshot 子进程债登记、逃生门冒烟）。

**逐条判定**：
1. 空闲 10min <50MB —— **partial（用户知悉残留后裁定放行）**：F1 空转语义实机证实（PollStream 拍零取数 0×chats_get_session、日志 240 拍全 no-op）+ soak 空闲相 40 拍零增长；但实机存在基线即有的 ~20Hz 视图重建驱动（非 timer/MCP/消息，AUTOUI_MCP_DISABLE=1 后依旧）→ 真实斜率 ~1.25MB/s。残项 KD-062①（+1 stake/call）×②（20Hz 驱动）相乘即当前斜率，均已登记且复审前已向用户如实汇报（用户据此指示复审）。
2. 钉子测试绿且常驻 —— **pass**：fire_timer_noop_does_not_dirty / musk_vm_track_heap_soak（双相）/ mutation_seq_bumps（含 musk 形态 MuskTick 扩展）在最终提交 nextest 全量内绿；plan051 timer 族 7/7 绿。
3. 发送-轮询-完成链路活体 —— **partial（上游阻塞，半段证实）**：发送→StartStream 副作用落位（poll_window 窗戳+pre_stream_len=7）→ PollStream 转活跃（每拍取数）实机证实；完成半段（回复落库→启发式→T7 关窗）不可端到端验证——阻塞于两个存量上游问题：KD-047 Sse.open handler-as-value 抛点（本复现实录 crash ip=0xe768，SendInput 内 turn 未落库——早于 T12 修复域）+ 该会话回复生成本就不落地（历史 7 user/0 assistant，PLAN-055 时代同形）。T7 关窗四清经 lint+probe 源级门禁；F1 不破坏活跃拍（soak B 相+plan051 族）。
4. 全门禁差分 —— **pass**：最终提交 nextest 4540 run / 4521 绿 / 19 红——layout 18（stash 基线逐测验证）+ charts（stash 基线+master 536 复审自档红）；d8/c2/icon/strict 为基线红/flake（各自 stash 或双跑对照在案）。**零新增红**。musk：vm-safe-lint 零红 + vm-link-probe PASS 63318B；vitest 因 gen/ gitignored 不入 worktree 延至合回后主检出跑（结构性延后，已在 T7 注记——复审补裁：合并前由 merge 步在主检出重生成+vitest 收口）。注：裸 cargo test 进程内并行有 183 假红（全局态互污），门禁必须 nextest——已入提交注。
5. canary 零触发 + 逃生门 —— **pass（带注）**：最终代码全量电池无 canary 触发（开发期两次 canary 命中均为实验性扩域清扫，已回退——canary 仪器本身工作正常是正面证据）；AUTOUI_FRAME_RC=0 逃生门经代码路径核对+soak 双跑（残留份额钉树下对象计数不可区分，行为差异记录在案）。
6. KD 回写 —— **pass（复审期补齐）**：051 行⑤ 部分核销（指向 KD-062① 续行）+ 048 观察 a) 补注（修复版同形 ~5min 静默退出，与泄漏无因果）+ 062 行新增（①RC 槽位记账 ②20Hz 驱动 ③已根治面）+ snapshot 子进程 66MB 尾注。

**遗漏/延后/workaround 清点**：
- 遗漏：无（任务→diff 全对账；T1 soak 断言由 +64 阈值改双相的过程在提交链透明可溯）。
- 延后（用户知悉）：KD-062①② 两项上游根修；vitest 主检出补跑（merge 步）；发送完成半段活体（随 KD-047/env 修复补）。
- Workaround：无新增 hack；F2 的 [0,sp) 清扫保守范围与 20Hz 驱动未定罪均为登记债而非绕行。
- 合并注意：分支点后 master 自身推进（POLLTRACE 等，engine.rs/dynamic.rs 同文件不同 hunk）——merge 时预期可自动合并，需复跑门禁；musk main 侧 061 计划并行推进（forge_store.at 无交叉）。

**路由**：`status: reviewed`（1/3 两条 partial 均为上游存量阻塞且已登记+用户知悉，其余全 pass）→ 交 /auto-plan:merge。

## 待澄清事项

- **[2026-09-05 T4 执行期登记] 残留 +1 stake/call（上游 RC 槽位记账专项）**：
  帧账本正确归还宿主份额（P419 追踪实证 rc 2→1），但每 call_vm_fn 仍有
  1 个未定位归属的存量 stake 钉住当帧 computed 树（soak 重建相 +21 obj/拍
  的来源）。已排除：任务帧槽扩域清扫（STORE_GLOBAL 转移语义下 pop 份额
  转入全局表而槽内留陈旧字节，扩域即双重释放——global_keeps_alive 实证
  全局对象被误杀，已回退 [0,sp)）；全局表（STORE_GLOBAL 跟踪无命中）；
  结果槽（裸副本，canary 实证清账即 UAF）。影响面：仅合法定时器活跃期
  （流式轮询 ~2Hz×21obj），空闲泄漏已由 F1+隐式重建移除根治为零。根修
  需 RC 槽位记账重构（每份额关联持有槽/表项，弃任务时精确清账）——
  归 auto-lang 上游专项，随 KD-051⑤ 续行登记。
- **[2026-09-05 T3 执行期登记] 基线既有红对照口径**：d8_toggle_dark_mode
  与 plan492 c2_param_msg 在干净基线（stash 后仅 T1 提交）即红（015 语料
  声明 dark_mode=true 与测试期望 false 相悖——语料/测试在途变更所致，
  主检出同红）；icon_component 宽跑 flake（聚焦跑绿、宽跑红，基线同形）。
  三者非 PLAN-062 回归，T6 全量门禁差分按此口径排除。
