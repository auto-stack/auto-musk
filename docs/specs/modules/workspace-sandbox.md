# 工作区沙箱与目录白名单（Workspace Sandbox）模块规范

> PLAN-070 落地（2026-09-15，reviewed 749bdb1）。PLAN-069 单根注入式沙箱的
> 多根扩展：工作区可配置**额外授权目录**（白名单），工具对白名单内路径放行，
> 语义等价 workspace 根。白名单只能由**用户**经 UI/API 显式维护（模型/工具
> 无写路径）；workspace 根恒为第一根且不可移除；白名单之外一律拒绝
> （fail-closed 不变）。

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
- **VM 轨缺口**：无白名单视图（whitelist.vm.at 缺，web-only 门面）——
  登记差异非回归（PLAN-068 files 同口径）；VM CLI/VM 运行无白名单入口。

## 测试口径

cargo lib：隔离（多工作区互不可见）/往返去重持久化（重载后一致）/7 类非法
输入拒绝/get_exact 严格解析回归/resolve_multi 命中-越界-移除恢复/confine
多根判定/ReadFile+RunCommand with_roots/roots API wire 往返与 400；
vitest：i18n 中英键 parity（whitelist.* + nav.whitelist）；E2E：:8083 serve
走查（ADD canonical 入账 → 400 文案三类 → 重启仍在 → REMOVE 即时空；
registry 快照-还原口径）。
