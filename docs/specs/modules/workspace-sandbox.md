# 工作区沙箱与目录白名单（Workspace Sandbox）模块规范

> PLAN-070 落地（2026-09-15，reviewed 749bdb1）。PLAN-069 单根注入式沙箱的
> 多根扩展：工作区可配置**额外授权目录**（白名单），工具对白名单内路径放行，
> 语义等价 workspace 根。白名单只能由**用户**经 UI/API 显式维护（模型/工具
> 无写路径）；workspace 根恒为第一根且不可移除；白名单之外一律拒绝
> （fail-closed 不变）。
> PLAN-105 增订（2026-10-12，reviewed 62e6a2c）：run_command 引号保真合同、
> 引号感知词法（confinement 与执行层共用）、run 进程收割（Job Object）、
> VM 轨白名单真接线；详见对应各节。

## 多根判定（tool_safety::resolve_multi）

- **任一根命中即放行**：`resolve_multi(path, roots)` 逐根按 `resolve_scoped`
  语义判定；全部越界 → Err（报文列出全部根，供门卡片/报文展示）。
- **两段式**（防止第一根"吞掉"其他根下真实存在的相对路径）：
  1. 存在/绝对优先——绝对路径落任一根内、或相对路径在某根下**真实存在** →
     放行（读/改既有文件按真实位置命中）；
  2. 新建归第一根——相对且尚不存在的路径（新建写）→ 归第一根（workspace
     根恒为第一根，新建位置可预测，不按猜测散落白名单）。
- **命令路径同口径**：`confine_offending_paths_multi`（human 门收集变体）/
  `confine_command_paths_multi`（auto 硬拒变体）按注入多根判定。070 顺带
  修复：旧 `confine_*` 走全局链（thread-local 退役后 = startup CWD），与
  注入 scope 脱节——workspace 内绝对路径误判越界（human 误弹门/auto 误拒）。
- **Windows 开关豁免（PLAN-103）**：token 形如 `/` + 纯字母数字单段（不含
  `\` `.` `:` 或第二个 `/`）且按路径在磁盘上不存在 → 视为命令开关，不入
  越界收集（`dir /b /ad` 曾被误判、human 会话误挂整段门等待，实机定罪）。
  cmd 语义下单段 `/x` 不是合法路径引用；带分隔符 `/x/y`、反斜杠、盘符路径
  与磁盘存在路径不受豁免照常收集（残余风险：Unix 风格单段根路径在 cmd 下
  不可达，记为已知边界）。
- **报错即事实**：`map_path_error` SecurityDenied 的 `root` 字段列出全部
  注入根；hint 指引"白名单视图添加目录"。

## 持久层与 API（workspace.rs）

- `WorkspaceMeta.extra_roots: Vec<String>`（canonical 绝对路径，serde
  default）随 `~/.config/autoos/workspaces.json` 持久化，跨 serve 重启生效。
- `add_extra_root` / `remove_extra_root` / `extra_roots`：canonical 化统一
  `\\?\` 形态（防伪：存储与判定两侧同形态）+ 存在且为目录校验 + 去重
  （Windows 忽略大小写）。
- **黑名单口径**（授权 = 子树全权，故按根型分级）：任意驱动器根拒；
  `C:\Windows`、`C:\Program Files( (x86))`、`C:\ProgramData` **前缀**拒
  （子树整体受保护）；`C:\Users` 与用户主目录本身**等值**拒（更深层子目录
  是合法的定向授权）。
- API（hw 路由，auth 保护区同 pick_routes）：`GET /api/workspace/roots?
  workspace={id}`、`POST /api/workspace/roots/add|remove`（`{workspace,
  root}` → `{roots}`）；缺参/未知 workspace/黑名单/不存在 → 400 + 文案。

## 运行入口合成与门联动

- `registry.sandbox_roots(ws_id)`（workflow step/errand 等无 id 场景用
  `sandbox_roots_by_path`）合成 `[workspace 根, *白名单]`，**每次运行构造
  工具前现读**——白名单增删对**后续运行**即时生效；**在途运行保持 spawn
  时快照**（`with_roots(Arc<Vec>)` 即快照语义；DisplayImage 例外，registry
  实时合成）。
- 九工具（read/write/edit/search/list_dir/list_symbols/glob/run_command/
  display_image）`root → roots: Option<Arc<Vec<PathBuf>>>`；`with_root`
  保留为单根兼容包装（CLI v1 仍单根 CWD——CLI 无白名单 UI）。
- **审批门联动（PLAN-069 W3）**：门判定与放行同一多根函数——human 会话
  首个越界 token（全部根之外）才挂 `tool_gate` 门；白名单内不触发；
  approve 放行一次/deny 回灌语义不变。

## 前端（web 轨）

- 主导航"白名单"（shield-check，viewstate 白名单 `whitelist`）→
  WhitelistView：列表（逐行移除）/添加（文本框+按钮）/空态/错误文案条/
  workspace 根恒定行；进入视图即重拉（v-if 重挂载 freshness）。
- 调用经 `ports/whitelist.web.at` → `whitelist_web.ts` **结果对象门面**
  （不走 api.at 绑定：绑定非 2xx 丢弃 error body，400 文案须原样展示；
  结果对象取 error 不用 catch——.at catch 值 unknown TS2571）。workspace
  显式携带（拦截器对 `/api/workspace/*` 豁免注入）。
- **VM 轨（PLAN-105 真接线）**：`ports/whitelist.vm.at` 经
  `musk_extern_dispatch` → `vm_backend` host! 注册表
  （`workspace_roots_list/add/remove`，in-process registry 直调）——不走
  HTTP face（VM http native 非 2xx 丢响应体，白名单须原样展示拒绝原因），
  add/remove 的 Err 文案经桥透传 = web 轨 400 body 同语义；宿主未注册 →
  诚实 `{ok:false,error}`（原 `{ok:true}` 空桩假成功——UAT 87af4d4b
  17:28:06 实锤——已退役）。

## run_command 执行合同（PLAN-105）

- **引号保真（Windows）**：`cmd /S /C` + 命令串整体 `raw_arg`（外层包
  引号；/S 语义 = 剥最外层引号后原样执行，等价交互式 cmd 输入）。原
  `.arg(cmd)` 经 Rust argv 引用规则 + cmd /C 剥引号把字面引号送进 argv
  （UAT 87af4d4b 四类损坏：`git -C '"D:/…"' Invalid argument`、
  `git commit -m "…"` 引号丢失中止、findstr 多词失配、curl 带引号 URL
  exit 3）。Unix `sh -c` 语义本就保真，不变。
- **引号感知词法（共用）**：`tool_safety::tokenize_command` 是
  confinement/classify 词法的单一来源——双引号段内不切分、引号自身剥离
  （`"D:/path with space"` 是一个 token）、相邻引号段拼接、反斜杠字面
  （cmd 习惯）、未闭合引号保守吞尾、单引号普通字符。原
  `split_whitespace` 引号盲——引号内越界路径以「首残片恰在根内」形态
  漏判（补红两例在案）。
- **进程收割**：run_command spawn 后立即上报进程凭证（`ExecOptions.on_proc`）
  进 `AppState.chat_procs` 注册表（键同守卫，经 build_agent_with_context
  闭包注入）——Windows = **kill-on-close Job Object** 句柄（`start` 分离
  孙进程唯一可靠收割面；pid+taskkill /T 对「父已退、孙分离」无效，UAT
  PID 8008 锁 worktree 实锤）、Unix = pid（spawn 已 process_group(0)
  组杀）。chat_run_owner 全部收束出口 + panic GuardOnDrop 统一收割
  （CloseHandle / killpg，≤5s）；正常完成的工具**不**注销凭证（分离孙
  在工具返回后仍活，收割点在 run 收束）。仅收 run_command 面（用户自启/
  画布/CDP 不碰）。宿主进程死亡 → 句柄全关 = 子孙全灭（进程卫生兜底）。
- **邻接合同**：超时臂读任务收尾 2s 有界（管道被存活孙持有时 EOF 永不到，
  裸 await 会把超时臂自身悬死——15 分钟实测复现后修复）。

## 测试口径

cargo lib：隔离（多工作区互不可见）/往返去重持久化（重载后一致）/7 类非法
输入拒绝/get_exact 严格解析回归/resolve_multi 命中-越界-移除恢复/confine
多根判定/ReadFile+RunCommand with_roots/roots API wire 往返与 400；
vitest：i18n 中英键 parity（whitelist.* + nav.whitelist）；E2E：:8083 serve
走查（ADD canonical 入账 → 400 文案三类 → 重启仍在 → REMOVE 即时空；
registry 快照-还原口径）。

## PLAN-096 修订（run 级执行作用域，2026-10-02）

- plan 流 execute/review/document 相位：`ToolContext.execution_root`
  显式执行作用域——文件/命令工具根与 Canvas 路径解析（resolve_within_
  sandbox 优先该字段）限定在 `plan_worktree::ensure_plan_worktree` 登记
  的开发 worktree；主检出代码**不注册为可写根**，白名单语义与用户配置
  不变（run 级注入根 ≠ 全局白名单扩权）。plan 相位与无作用域 run 维持
  本章多根判定。
- worktree 租约纪律：main checkout 校验、默认分支探测（不硬编码）、
  占用不接管、reparse point guard、合入核对前置的安全移除——详见
  `plan-execution-contract.md`。
