//! Tool-level approval gate (PLAN-069 W3).
//!
//! `run_command` 预执行检出越界路径且会话审批模式为 human 时，工具挂起等待
//! 用户决议（SSE `tool_gate_waiting` 事件 + 审批端点）。hub 保存
//! gate_id → oneshot::Sender；`/api/chats/tool-gate/{id}/approve|deny` 决议
//! 唤醒挂起的工具（approve → 放行执行；deny → 拒绝回灌模型）。auto 模式
//! 不经过本 hub（维持硬拒 + 继续）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::oneshot;

static HUB: OnceLock<Mutex<HashMap<String, oneshot::Sender<bool>>>> = OnceLock::new();

fn hub() -> &'static Mutex<HashMap<String, oneshot::Sender<bool>>> {
    HUB.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 注册一个待决议门；返回接收端（true = approve，false = deny）。
pub fn register(gate_id: &str) -> oneshot::Receiver<bool> {
    let (tx, rx) = oneshot::channel();
    hub().lock().unwrap().insert(gate_id.to_string(), tx);
    rx
}

/// 决议一个门。返回 false = 门不存在（已超时/重复决议）。
pub fn resolve(gate_id: &str, approved: bool) -> bool {
    if let Some(tx) = hub().lock().unwrap().remove(gate_id) {
        let _ = tx.send(approved);
        true
    } else {
        false
    }
}
