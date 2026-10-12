# demo-evidence — 隔离 VM 全流程演示与证据合同

> 来源：PLAN-097（SD-04，2026-10-12 沉淀）。固定可复现的「空仓三计划
> 真实模型演示」合同：驱动只观察与验证，证据链版本绑定、缺件显式、
> 机器可验。实现真源：`scripts/demo/{contract,vm-session,capture,
> evidence,focusboard,assertions,wiki,landing,report,run}.mjs` 与
> `scripts/demo-focusboard.mjs`；需求件 `examples/focusboard-demo/`；
> 针对性测试 `tests/demo/*.test.mjs`（V02 门）。
> v0.5/v0.6 边界（用户 2026-10-12 裁定）：canvas 完整三轮 delivered
> 真机演示延期 v0.6（KNOWN-DEBT 097-D1）；本合同描述的管线/证据语义
> 已全部交付并经五跑真机验证。

## 隔离启动

- 每跑唯一 run_id/目录（`D:/autostack/.demo/musk-097/<run_id>/`）；
  自有 musk serve（私有端口段 + MUSK_CONFIG_DIR 隔离）+ 原生 VM
  （AUTO_EXE 显式锁定当日构建——PATH 滞后构建禁令）；AAID 只读复用
  （不修改配置/不重启/不按名结束）；关停只杀自有 spawn 树 + 端口回收
  验收；遗留进程/端口占用/旧二进制 = 显式失败。

## 空仓三计划

- bootstrap 仅物化 README/AGENTS/requests 三篇（白名单审计，任何业务
  文件 = 违规）；git init 线性仓；三轮增量需求走正式
  `flow_id=plan`+`authorization=auto` 入口；needs_replan/blocked/修复
  封顶/failed/canceled 即停（零 nudge、零外部修码）；**工程未交付不发
  下一需求**（deliveredGate：completed+delivered+五检查点齐）。

## 驱动权限

- 驱动只观察与验证——发送需求、读事实、采集、断言；不推进内部阶段、
  不写完成标记、不制造评审结果；行为断言经目标 VM 动作（宿主 MCP
  vnode 直选，零固定假坐标）+ 磁盘数据交叉验证。

## 截图与证据

- 来源双轨 `host-vm`（autoui_screenshot 原图直存，采集前后身份双验）/
  `target-canvas`（generation 归属核验，跨计划/重启代次 409 拒）；
  capture 记录 capture_id/source/UTC/viewport/png 相对路径+SHA256/
  kind(live|completed-stage-record)/stage_ref；live 采集必须绑定已登记
  阶段事实（缺阶段注入拒绝；场景级整屏显式 allowUnstaged 且落记录）；
  PNG 原始画面不重绘不拼接。
- **manifest v1**：运行指纹（musk/auto 二进制 sha256+版本+worktree
  head）、workspace/session/run/plan 身份、revision、各阶段
  stage/attempt/outcome、reviewed_commit、五交付检查点、收据引用、
  落地主提交与 capture 集合；外部保存（run 目录，与 worktree 生命
  周期解耦）；verify 独立核验（删除图/错 hash/schema 缺件 = 非零）。

## 知识出版

- 每轮 Agent 产出 `docs/wiki/*.md` 为唯一正文来源；驱动经既有
  workspace Wiki API 发布**同一正文**（create→409 幂等 update），源/
  回读 SHA256 一致判言；报告披露该确定性出版步骤区别于模型行为。

## 落地对账

- 默认分支线性（无 merge commit）、本地分支只剩默认、git worktree
  注册表除主检出外无项、计划 worktree 目录已清（约定外置 + 后端托管
  `<plan>/<repo>` 布局）、docs/specs 权威 + ledger 派生 + 归档非空 +
  Wiki 产出。

## P003 上下文

- 新 UI 会话（id 不复用）、会话输入 == 需求原文（空白归一后全等——
  无注入）、run 事件窗内存在 docs/specs|plans|wiki 的 turn_tool_call
  读取证据（声称已读不算）。

## 失败语义

- 任何轮未交付/验收失败/采集缺件/canceled → 演示非零退出并保留现场
  （不覆盖旧跑、不改 A 上限、不伪成功）。
