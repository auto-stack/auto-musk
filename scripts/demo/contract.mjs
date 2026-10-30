// scripts/demo/contract.mjs — PLAN-097 T-02：B 轨消费合同模块（冻结自
// PLAN-096 交付落地代码，worktree plan-097-dev @ 7924304）。
//
// 冻结来源（SHA-256 前 16 位，全文见 docs/reports/097-vm-demo-baseline.md §T-02）：
//   spec    docs/specs/modules/plan-execution-contract.md  43c26a74df899fff
//   dto     backend/crates/musk/src/relay/plan_contract.rs e95fa7556c30bf2b
//   control backend/crates/musk/src/relay/plan_control.rs  a23500e350926042
//   store   backend/crates/musk/src/relay/store.rs         20e84b7f30c605d0
//   probe   scripts/plan-flow-probe.mjs                    49dc9fdc9a93f374
//
// 事实面（spec §事实面）：GET /api/forge/relay/runs/{run_id}?workspace=<id>
// 返回 RunState 全量 serde 序列化（404→null body）；`plan_execution` 可选
// （serde default 旧消费者兼容）；事件 `plan_stage_facts`（RunPlanEvent）在
// RunState.events 窗口内（500 条，F-R1）；B 轨以 facts 序列还原时间线，不解析
// 自然语言 handoff。本模块只消费事实并显式拒绝未知版本/缺件，不猜测。

export const PLAN_EXECUTION_CONTRACT_VERSION = 1;
export const CONTRACT_FROZEN = {
  at_commit: '7924304',
  // 路径 → 冻结时 SHA-256 前 16 位（preflight 对账用；漂移=演示身份失真）
  sources: {
    'docs/specs/modules/plan-execution-contract.md': '43c26a74df899fff',
    'backend/crates/musk/src/relay/plan_contract.rs': 'e95fa7556c30bf2b',
    'backend/crates/musk/src/relay/plan_control.rs': 'a23500e350926042',
    'backend/crates/musk/src/relay/store.rs': '20e84b7f30c605d0',
    'scripts/plan-flow-probe.mjs': '49dc9fdc9a93f374',
  },
};

export const PHASES = ['plan', 'execute', 'review', 'document', 'delivered'];
export const OUTCOMES = ['pass', 'needs_fix', 'needs_replan', 'blocked'];
// 交付检查点（plan_delivery.rs record_checkpoint 键；delivered 相位要求全五在列）
export const DELIVERY_CHECKPOINTS = ['prepared', 'landed', 'archived', 'cleaned'];
// PlanExecutionState 必填（缺任一 = 后端合同损坏，显式拒绝；serde default 字段可缺席）
export const PLAN_EXECUTION_REQUIRED = ['contract_version', 'plan_id', 'plan_path', 'phase'];
// RunPlanEvent（事件名 plan_stage_facts）必填
export const PLAN_EVENT_REQUIRED = ['plan_id', 'stage', 'outcome'];

// ── 冻结样例（形态按 landed 结构体重建；live 全量样例 T-12 绑定） ──────
// 成功形态：096 L1 全链（status=completed + phase=delivered + 五检查点 + 收据
// completion_kind=delivered；live 汇总见 docs/plans/archived/096-* d1dd7f0）。
export const SUCCESS_SAMPLE = Object.freeze({
  run_id: 'run-sample-1',
  status: 'completed',
  plan_execution: {
    contract_version: 1,
    plan_id: 'PLAN-XXX',
    plan_seq: 1,
    plan_path: 'docs/plans/xxx-sample.md',
    plan_revision: 1,
    phase: 'delivered',
    attempt: 1,
    outcome: 'pass',
    repair_count: 0,
    repair_limit: 3,
    continuations: {},
    blocker: null,
    binding: { contract_version: 1, authorization: 'auto', repair_limit: 3 },
    reviewed_commit: 'ffffffffffffffffffffffffffffffffffffffff',
    delivery_checkpoints: {
      prepared: { checkpoint: 'prepared' },
      landed: { checkpoint: 'landed' },
      ledger_refreshed: { checkpoint: 'ledger_refreshed' },
      archived: { checkpoint: 'archived' },
      cleaned: { checkpoint: 'cleaned' },
    },
    receipt_ref: '.autoos/plan-delivery/PLAN-XXX.json',
    skills: {},
    stage_results: [],
  },
});
// 失败形态：096 L3（run failed；blocker 落事实；计划未归档、无 delivered 收据
// ——错误分类不路由，响亮停止）。只冻结字段面，不冻结具体 blocker 文本。
export const FAILURE_SAMPLE = Object.freeze({
  run_id: 'run-sample-2',
  status: 'failed',
  plan_execution: {
    contract_version: 1,
    plan_id: 'PLAN-XXX',
    plan_seq: 1,
    plan_path: 'docs/plans/xxx-sample.md',
    plan_revision: 1,
    phase: 'document',
    attempt: 1,
    outcome: 'blocked',
    repair_count: 0,
    repair_limit: 3,
    blocker: 'ledger load failed loudly (corrupt specs store)',
    reviewed_commit: null,
    delivery_checkpoints: {},
    receipt_ref: null,
    stage_results: [],
  },
});

const isStr = v => typeof v === 'string' && v.length > 0;
const err = (list, msg) => list.push(msg);

// 校验 RunState.plan_execution（值存在即校验；缺字段 = 显式错误，不默认成功）
export function validatePlanExecution(pe) {
  const errors = [];
  if (!pe || typeof pe !== 'object') { err(errors, 'plan_execution missing or not an object'); return { ok: false, errors }; }
  for (const k of PLAN_EXECUTION_REQUIRED) {
    if (!(k in pe) || !isStr(pe[k]) && k !== 'contract_version') err(errors, `plan_execution.${k} missing/empty`);
  }
  if (pe.contract_version !== PLAN_EXECUTION_CONTRACT_VERSION) {
    err(errors, `unknown contract_version ${JSON.stringify(pe.contract_version)} (frozen: ${PLAN_EXECUTION_CONTRACT_VERSION}) — refusing to guess`);
  }
  if (isStr(pe.phase) && !PHASES.includes(pe.phase)) err(errors, `plan_execution.phase "${pe.phase}" not in ${PHASES.join('|')}`);
  if (pe.outcome != null && !OUTCOMES.includes(pe.outcome)) err(errors, `plan_execution.outcome "${pe.outcome}" not in ${OUTCOMES.join('|')}`);
  if (pe.delivery_checkpoints != null) {
    if (typeof pe.delivery_checkpoints !== 'object' || Array.isArray(pe.delivery_checkpoints)) {
      err(errors, 'delivery_checkpoints must be an object');
    } else {
      for (const k of Object.keys(pe.delivery_checkpoints)) {
        if (!DELIVERY_CHECKPOINTS.includes(k) && k !== "ledger_refreshed") err(errors, `delivery_checkpoints key "${k}" unknown`);
      }
      if (pe.phase === 'delivered') {
        for (const k of DELIVERY_CHECKPOINTS) {
          if (!(k in pe.delivery_checkpoints)) err(errors, `phase=delivered but checkpoint "${k}" missing (缺件完成不绿)`);
        }
      }
    }
  }
  if (pe.receipt_ref != null) {
    if (!isStr(pe.receipt_ref)) err(errors, 'receipt_ref must be a non-empty string when present');
    else if (/^[A-Za-z]:[\\/]/.test(pe.receipt_ref) || pe.receipt_ref.startsWith('/')) {
      err(errors, `receipt_ref must be workspace-relative, got "${pe.receipt_ref}"`);
    }
  }
  return { ok: errors.length === 0, errors };
}

// 校验单条 plan_stage_facts 事件（RunPlanEvent 载荷）
export function validatePlanEvent(ev) {
  const errors = [];
  for (const k of PLAN_EVENT_REQUIRED) {
    if (!isStr(ev?.[k])) err(errors, `event.${k} missing/empty`);
  }
  if (isStr(ev?.stage) && !PHASES.includes(ev.stage) && ev.stage !== 'delivery') {
    // delivery 检查点事件 stage 面见 plan_delivery record_checkpoint（T-12 绑定）
    err(errors, `event.stage "${ev.stage}" not in ${PHASES.join('|')}|delivery`);
  }
  if (isStr(ev?.outcome) && !OUTCOMES.includes(ev.outcome)) err(errors, `event.outcome "${ev.outcome}" not in ${OUTCOMES.join('|')}`);
  return { ok: errors.length === 0, errors };
}

// 校验 run 响应整体：顶层 status + plan_execution + events 窗口内的事实事件。
// 事件面（T-12 live 实证合同漂移修订）：A 控制器会发出 outcome='continuation'
// 等枚举外的事实事件（同阶段续跑通知）——事件是通知面,权威面是顶层
// plan_execution（其 outcome 枚举保持严格）。事件违约不拒收整个 RunState,
// 降级为 _contract_warnings 由消费方留痕（响亮可见,不静默）。
export function validateRunState(run) {
  const errors = [];
  const warnings = [];
  if (!run || typeof run !== 'object') { err(errors, 'run state missing or not an object'); return { ok: false, errors, warnings }; }
  if (!isStr(run.run_id)) err(errors, 'run.run_id missing/empty');
  if (run.plan_execution !== null && run.plan_execution !== undefined) {
    const r = validatePlanExecution(run.plan_execution);
    if (!r.ok) errors.push(...r.errors);
  }
  if (run.events !== undefined) {
    if (!Array.isArray(run.events)) err(errors, 'run.events must be an array (500-event window)');
    else {
      for (const [i, ev] of run.events.entries()) {
        if (ev?.type !== 'plan_stage_facts') continue;
        // RunEvent::PlanStageFacts serde 形态 = { type, timestamp, facts: RunPlanEvent }
        // （T-12 live 实证:载荷嵌套在 facts 键;顶层直挂形态兼容读）
        const payload = ev.facts ?? ev;
        const r = validatePlanEvent(payload);
        if (!r.ok) for (const e of r.errors) warnings.push(`events[${i}]: ${e}`);
      }
    }
  }
  return { ok: errors.length === 0, errors, warnings };
}

// 解析 run 详情响应。404 形态 = 纯文本 "run 'x' not found"（relay_api.rs
// get_run: value_is_null → text_response 404），非 JSON、非 null body。
// 事件面警告附在 `_contract_warnings`（消费方留痕;见 validateRunState）。
export function parseRunStateResponse(text) {
  let j;
  try { j = JSON.parse(text); } catch {
    if (/run '.*' not found/.test(String(text))) throw new Error(`run not found (404): ${String(text).trim()}`);
    throw new Error(`run state response is not JSON: ${String(text).slice(0, 120)}`);
  }
  if (j === null) throw new Error('run state response is null (run not found semantics)');
  const v = validateRunState(j);
  if (!v.ok) { const e = new Error(`run state failed contract validation: ${v.errors.join('; ')}`); e.errors = v.errors; throw e; }
  if (v.warnings.length) j._contract_warnings = v.warnings;
  return j;
}

// 从列表摘要({runs:[RunSummary]})挑出候选并逐个取详情，返回首个带
// plan_execution 的 RunState（驱动等待交付时的兜底发现通道；正常路径驱动
// 自己发起 run、直接持有 run_id）。fetchDetail(runId) 由调用方注入。
export async function discoverPlanRun(summaries, fetchDetail) {
  if (!Array.isArray(summaries)) return null;
  for (const s of summaries) {
    const rid = s?.run_id;
    if (!rid || s?.status === 'failed' || s?.status === 'canceled') continue;
    try {
      const detail = await fetchDetail(rid);
      if (detail?.plan_execution) return detail;
    } catch { /* 404/暂不可读的候选跳过 */ }
  }
  return null;
}
