//! PLAN-087 T-03: AutoUI MCP 客户端（JSON-RPC over HTTP）。
//!
//! 协议契约（auto-lang `ui/mcp_server.rs`，8fecfcf69 基线核实）：
//! - 单端点 `POST /mcp`，每请求独立处理（无会话头/SSE）。
//! - `initialize` → `{protocolVersion:"2024-11-05", capabilities, serverInfo}`，
//!   随后 `notifications/initialized`（无响应体）。服务器无状态，initialize
//!   尽力而为不作为门。
//! - `tools/call` params = `{name, arguments}`；结果封装
//!   `{content:[{type:"text",text}], isError}` —— 四个 autoui_* 工具全部返回
//!   纯文本（计划原稿"截图 base64"与实现不符：实际返回落盘路径文本，此处
//!   按路径语义消费，读文件取 PNG 字节）。
//! - 工具面：autoui_screenshot（默认写 `<app>/tmp/autoui-screenshot-<ms>.png`
//!   → "Screenshot saved to: <abs>"；baseline 臂写固定名但污染 app 目录，弃）、
//!   autoui_action（`element_id`+`action` 枚举 press/type_text/…/key_press + 可选
//!   `value`）、autoui_state（`fields:[..]` 过滤，返回 "State:\n  count: 3 (int)"）、
//!   autoui_snapshot（AURA 文本树，含 bounds 开关）。

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

/// 截图/动作调用超时（>10s 判挂起，看门狗走复活路径）。
pub const CALL_TIMEOUT: Duration = Duration::from_secs(10);
/// snapshot 载荷较大，放宽到 15s。
pub const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(15);

pub struct McpClient {
    http: reqwest::Client,
    base: String,
    next_id: std::sync::atomic::AtomicU64,
}

impl McpClient {
    pub fn new(base: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base,
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    async fn rpc(&self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let body = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let resp = self
            .http
            .post(format!("{}/mcp", self.base))
            .header("Accept", "application/json")
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("mcp request failed: {e}"))?;
        let status = resp.status();
        let payload: Value = resp.json().await.map_err(|e| format!("mcp response not json: {e}"))?;
        if !status.is_success() {
            return Err(format!("mcp http {}: {payload}", status.as_u16()));
        }
        if let Some(err) = payload.get("error") {
            return Err(format!(
                "mcp error {}: {}",
                err.get("code").and_then(|c| c.as_i64()).unwrap_or(0),
                err.get("message").and_then(|m| m.as_str()).unwrap_or("?")
            ));
        }
        Ok(payload.get("result").cloned().unwrap_or(Value::Null))
    }

    /// initialize + initialized 通知（尽力而为：服务器无状态，失败不阻断）。
    pub async fn initialize(&self) {
        let init = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "musk-canvas", "version": "0.1.0" }
        });
        let _ = self.rpc("initialize", init, CALL_TIMEOUT).await;
        let _ = self
            .rpc(
                "notifications/initialized",
                json!({}),
                Duration::from_secs(2),
            )
            .await;
    }

    /// tools/call → 解封装 content[0].text；isError=true 转 Err。
    async fn call_tool(&self, name: &str, args: Value, timeout: Duration) -> Result<String, String> {
        let result = self
            .rpc("tools/call", json!({ "name": name, "arguments": args }), timeout)
            .await?;
        let is_error = result
            .get("isError")
            .and_then(|e| e.as_bool())
            .unwrap_or(false);
        let text = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if is_error {
            return Err(if text.is_empty() {
                "mcp tool reported error without text".to_string()
            } else {
                text
            });
        }
        Ok(text)
    }

    /// 截图并读回 PNG 字节。默认臂写 `<app>/tmp/autoui-screenshot-<ms>.png`
    /// （cwd = app 目录），返回 "Screenshot saved to: <abs>"；返回
    /// `(落盘路径, 字节)`——调用方（看门狗）删上一帧文件防堆积。
    pub async fn screenshot(&self) -> Result<(std::path::PathBuf, Vec<u8>), String> {
        let text = self
            .call_tool("autoui_screenshot", json!({}), CALL_TIMEOUT)
            .await?;
        let path = text
            .strip_prefix("Screenshot saved to: ")
            .map(str::trim)
            .ok_or_else(|| format!("screenshot: unexpected reply: {text}"))?;
        let path = std::path::PathBuf::from(path);
        let bytes = read_png(&path).await?;
        Ok((path, bytes))
    }

    /// 驱动界面。action ∈ press/type_text/submit/toggle/select_option/
    /// set_value/clear/scroll/drag/pen/resize_col/key_press/editor_drag。
    pub async fn action(
        &self,
        element_id: &str,
        action: &str,
        value: Option<Value>,
    ) -> Result<String, String> {
        let mut args = json!({ "element_id": element_id, "action": action });
        if let Some(v) = value {
            args["value"] = v;
        }
        self.call_tool("autoui_action", args, CALL_TIMEOUT).await
    }

    /// 求值态读取（fields 过滤：全等或 `.field` 后缀匹配）。
    pub async fn state(&self, fields: Option<Vec<String>>) -> Result<String, String> {
        let args = match fields {
            Some(f) if !f.is_empty() => json!({ "fields": f }),
            _ => json!({}),
        };
        self.call_tool("autoui_state", args, CALL_TIMEOUT).await
    }

    /// AURA 结构树（vtree）。include_bounds 出 bounds 标注。
    pub async fn snapshot(&self, include_bounds: bool) -> Result<String, String> {
        self.call_tool(
            "autoui_snapshot",
            json!({ "include_bounds": include_bounds }),
            SNAPSHOT_TIMEOUT,
        )
        .await
    }
}

async fn read_png(path: &Path) -> Result<Vec<u8>, String> {
    // 截图落盘与响应返回间存在极小窗口，重试一次读。
    for attempt in 0..2 {
        match tokio::fs::read(path).await {
            Ok(bytes) if !bytes.is_empty() => return Ok(bytes),
            Ok(_) => return Err(format!("screenshot file is empty: {}", path.display())),
            Err(e) if attempt == 0 && e.kind() == std::io::ErrorKind::NotFound => {
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
            Err(e) => return Err(format!("screenshot read failed: {e}")),
        }
    }
    unreachable!("retry loop returns on both arms")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn screenshot_parses_saved_to_reply() {
        // 对路径解析逻辑的独立验证：screenshot 的文本契约消费点。
        let text = "Screenshot saved to: D:\\app\\tmp\\autoui-screenshot-123.png";
        let path = text.strip_prefix("Screenshot saved to: ").map(str::trim);
        assert_eq!(
            path.map(Path::new).map(|p| p.extension().and_then(|e| e.to_str()).unwrap_or("")),
            Some("png")
        );
    }
}
