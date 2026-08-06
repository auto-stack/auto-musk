//! parity_wiki_http.rs — Plan 020 Phase D: wiki.at HTTP 层等价测试。
//!
//! hw `wiki::wiki_routes` vs ag `auto_generated::wiki::wiki_routes` 双 router
//! 对照:同一请求序列跑两边,比较**状态码 + wire 形状**。
//!
//! - wiki CRUD(tree/pages/page/search):逐键等价断言(页面含 slug/title/content/
//!   source_type/tags/version/created_at/updated_at;create/update 的时间戳与版本
//!   是确定性递增,两边独立 temp store 产生相同值)。
//! - raw(tree/mkdir/file/delete):文件系统操作 + 204 空响应。
//! - 错误路径:404 page、400 invalid path、409 已存在、404 update/delete。
//! - raw_upload(multipart):用 axum Body 构造 multipart 请求。

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

use auto_ai_agent::Client;
use auto_ai_client::{ClientError, CompletionRequest, CompletionResponse};
use musk::server::AppState;

struct MockClient;
#[async_trait::async_trait]
impl Client for MockClient {
    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ClientError> {
        Err(ClientError::DaemonUnavailable)
    }
}

fn tmp_state() -> AppState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "musk-parity-wiki-http-{}-{}",
        std::process::id(),
        n
    ));
    let _ = std::fs::create_dir_all(&dir);
    let registry = musk::workspace::WorkspaceRegistry::load(dir.join("workspaces.json"), dir.clone());
    AppState {
        client: Arc::new(MockClient) as Arc<dyn Client>,
        auth: Arc::new(musk::auto_generated::auth::AuthStore::new(dir.join("users.json"))),
        registry: Arc::new(registry),
    }
}

fn hw_app(state: AppState) -> Router {
    musk::wiki::wiki_routes().with_state(state)
}

fn ag_app(state: AppState) -> Router {
    musk::auto_generated::wiki::wiki_routes().with_state(state)
}

/// Send JSON request;return (status, body)。非 JSON body → Value::String(raw)。
async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let resp = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            app.clone()
                .oneshot(builder.body(Body::from(b.to_string())).unwrap())
                .await
                .unwrap()
        }
        None => app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap(),
    };
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) => v,
            Err(_) => Value::String(String::from_utf8_lossy(&bytes).into_owned()),
        }
    };
    (status, json)
}

// ── Wiki CRUD ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn wiki_crud_hw_vs_ag() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());

    // 初始 tree + pages 为空,两边逐键等价。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/wiki/proj/tree", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/wiki/proj/tree", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "empty wiki tree parity");
    assert_eq!(b_hw, json!([]));

    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/wiki/proj/pages", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/wiki/proj/pages", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "empty pages parity");
    assert_eq!(b_hw, json!({"pages": []}));

    // create → {"page": {…}};两边 slug/title/content/source_type 等价。
    let body = json!({
        "slug": "intro",
        "title": "Intro",
        "content": "# Hello",
        "source_type": "manual",
        "tags": ["a", "b"]
    });
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/wiki/proj/pages", Some(body.clone())).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/wiki/proj/pages", Some(body)).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_hw["page"]["slug"], "intro");
    assert_eq!(b_ag["page"]["slug"], "intro");
    assert_eq!(b_ag["page"], b_hw["page"], "create wire parity (version/timestamps deterministic)");
    assert_eq!(b_hw["page"]["version"], 1);
    assert_eq!(b_hw["page"]["source_type"], "manual");
    assert_eq!(b_hw["page"]["tags"], json!(["a", "b"]));

    // create 同 slug → 409 纯文本(两边一致)。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/wiki/proj/pages", Some(json!({"slug": "intro", "title": "x", "content": "y"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/wiki/proj/pages", Some(json!({"slug": "intro", "title": "x", "content": "y"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "409 create parity");
    assert_eq!(s_hw, StatusCode::CONFLICT);
    assert!(b_hw.as_str().unwrap_or("").contains("already exists"));

    // create 非法 path → 400 "Invalid path"。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/wiki/proj/pages", Some(json!({"slug": "../evil", "title": "x", "content": "y"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/wiki/proj/pages", Some(json!({"slug": "../evil", "title": "x", "content": "y"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "400 invalid slug parity");
    assert_eq!(s_hw, StatusCode::BAD_REQUEST);
    assert_eq!(b_hw, json!("Invalid path"));

    // list_pages → 1 页,两边等价。
    let (_, b_hw) = send(&hw, "GET", "/api/forge/wiki/proj/pages", None).await;
    let (_, b_ag) = send(&ag, "GET", "/api/forge/wiki/proj/pages", None).await;
    assert_eq!(b_hw, b_ag, "pages list parity");
    assert_eq!(b_hw["pages"].as_array().unwrap().len(), 1);
    assert_eq!(b_hw["pages"][0]["slug"], "intro");

    // get_page → {"page": {…}}。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/wiki/proj/page/intro", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/wiki/proj/page/intro", None).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_ag["page"], b_hw["page"], "get_page wire parity");
    assert_eq!(b_hw["page"]["content"], "# Hello");

    // get_page 缺失 → 404 空响应。
    let (s_hw, _) = send(&hw, "GET", "/api/forge/wiki/proj/page/missing", None).await;
    let (s_ag, _) = send(&ag, "GET", "/api/forge/wiki/proj/page/missing", None).await;
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
    assert_eq!(s_ag, StatusCode::NOT_FOUND);

    // update_page → {"page": {…}};content/title 更新,version 递增。
    let (s_hw, b_hw) = send(&hw, "PUT", "/api/forge/wiki/proj/page/intro", Some(json!({"content": "# Hello v2", "title": "Intro v2", "tags": ["a"]}))).await;
    let (s_ag, b_ag) = send(&ag, "PUT", "/api/forge/wiki/proj/page/intro", Some(json!({"content": "# Hello v2", "title": "Intro v2", "tags": ["a"]}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_ag["page"], b_hw["page"], "update wire parity");
    assert_eq!(b_hw["page"]["content"], "# Hello v2");
    assert_eq!(b_hw["page"]["title"], "Intro v2");
    assert_eq!(b_hw["page"]["version"], 2);

    // update 缺失 → 404 纯文本。
    let (s_hw, b_hw) = send(&hw, "PUT", "/api/forge/wiki/proj/page/missing", Some(json!({"content": "x"}))).await;
    let (s_ag, b_ag) = send(&ag, "PUT", "/api/forge/wiki/proj/page/missing", Some(json!({"content": "x"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "404 update parity");
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
    assert!(b_hw.as_str().unwrap_or("").contains("not found"));

    // search → {"results": […]};命中 content 与 title。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/wiki/proj/search", Some(json!({"query": "v2"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/wiki/proj/search", Some(json!({"query": "v2"}))).await;
    assert_eq!(s_hw, StatusCode::OK);
    assert_eq!(s_ag, StatusCode::OK);
    assert_eq!(b_ag, b_hw, "search wire parity");
    assert_eq!(b_hw["results"].as_array().unwrap().len(), 1);
    assert_eq!(b_hw["results"][0]["slug"], "intro");

    // tree → 去 .md 后缀的页面节点(hw 带 modified mtime;ag 留 None —— 已文档化
    // 分歧(parity_wiki L13),比较前从 hw 侧剔除 modified)。
    let (_, mut b_hw) = send(&hw, "GET", "/api/forge/wiki/proj/tree", None).await;
    let (_, b_ag) = send(&ag, "GET", "/api/forge/wiki/proj/tree", None).await;
    for node in b_hw.as_array_mut().unwrap() {
        if let Some(obj) = node.as_object_mut() {
            obj.remove("modified");
        }
    }
    assert_eq!(b_hw, b_ag, "tree parity after create");
    assert_eq!(b_hw[0]["name"], "intro");
    assert_eq!(b_hw[0]["type"], "file");

    // delete_page → 204 空。
    let (s_hw, b_hw) = send(&hw, "DELETE", "/api/forge/wiki/proj/page/intro", None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", "/api/forge/wiki/proj/page/intro", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "delete parity");
    assert_eq!(s_hw, StatusCode::NO_CONTENT);
    assert_eq!(b_hw, Value::Null);

    // delete 已删 → 404 纯文本。
    let (s_hw, b_hw) = send(&hw, "DELETE", "/api/forge/wiki/proj/page/intro", None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", "/api/forge/wiki/proj/page/intro", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "404 delete parity");
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
}

// ── Raw endpoints ──────────────────────────────────────────────────────────

#[tokio::test]
async fn raw_tree_mkdir_file_delete_hw_vs_ag() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());

    // 空 tree。
    let (s_hw, b_hw) = send(&hw, "GET", "/api/forge/raw/proj/tree", None).await;
    let (s_ag, b_ag) = send(&ag, "GET", "/api/forge/raw/proj/tree", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "empty raw tree parity");

    // mkdir → 204。
    let (s_hw, _) = send(&hw, "POST", "/api/forge/raw/proj/mkdir", Some(json!({"path": "docs"}))).await;
    let (s_ag, _) = send(&ag, "POST", "/api/forge/raw/proj/mkdir", Some(json!({"path": "docs"}))).await;
    assert_eq!(s_hw, StatusCode::NO_CONTENT);
    assert_eq!(s_ag, StatusCode::NO_CONTENT);

    // 直接写文件到 raw_dir(两边各自 store),然后 raw_file 读回 + 404。
    // 从 tree 拿到 raw_dir 不可行(不在 wire),故经 ws store 直接写。
    // 这里通过 mkdir 已验证 204;文件读写用 ag/hw 各写一个同名文件验证等价。
    for (app, name) in [(&hw, "hw"), (&ag, "ag")] {
        // 定位 state 的 raw_dir —— 通过 tree 端点不可行,改用 registry 内 store。
        // 简化:tree 断言两边结构等价即可;文件上传/读取走 multipart 端点测。
        let (s, b) = send(app, "GET", "/api/forge/raw/proj/tree", None).await;
        assert_eq!(s, StatusCode::OK, "{name} raw tree after mkdir");
        assert_eq!(b, json!([{"name": "docs", "path": "docs", "type": "folder", "children": []}]), "{name} raw tree node");
    }

    // 删除不存在的文件 → 404 纯文本。
    let (s_hw, b_hw) = send(&hw, "DELETE", "/api/forge/raw/proj/file/nope.txt", None).await;
    let (s_ag, b_ag) = send(&ag, "DELETE", "/api/forge/raw/proj/file/nope.txt", None).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "raw delete 404 parity");
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
    assert_eq!(b_hw, json!("Not found"));

    // raw_file 缺失 → 404。
    let (s_hw, _) = send(&hw, "GET", "/api/forge/raw/proj/file/nope.txt", None).await;
    let (s_ag, _) = send(&ag, "GET", "/api/forge/raw/proj/file/nope.txt", None).await;
    assert_eq!(s_hw, StatusCode::NOT_FOUND);
    assert_eq!(s_ag, StatusCode::NOT_FOUND);

    // mkdir 非法 path → 400。
    let (s_hw, b_hw) = send(&hw, "POST", "/api/forge/raw/proj/mkdir", Some(json!({"path": "../x"}))).await;
    let (s_ag, b_ag) = send(&ag, "POST", "/api/forge/raw/proj/mkdir", Some(json!({"path": "../x"}))).await;
    assert_eq!((s_hw, &b_hw), (s_ag, &b_ag), "raw mkdir 400 parity");
    assert_eq!(s_hw, StatusCode::BAD_REQUEST);
    assert_eq!(b_hw, json!("Invalid path"));
}

#[tokio::test]
async fn raw_upload_multipart_hw_vs_ag() {
    let hw = hw_app(tmp_state());
    let ag = ag_app(tmp_state());

    // multipart body:一个 field,filename="hello.txt",内容 "hi from musk"。
    let boundary = "X-TEST-BOUNDARY";
    let part = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\nContent-Type: text/plain\r\n\r\nhi from musk\r\n--{boundary}--\r\n"
    );
    for (app, name) in [(&hw, "hw"), (&ag, "ag")] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/forge/raw/proj/upload")
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .body(Body::from(part.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{name} upload 200");
        let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, json!({"uploaded": ["hello.txt"]}), "{name} upload wire");
    }

    // raw_file 读回 → 200 + 正确 content-type + 内容(两边一致)。
    for (app, name) in [(&hw, "hw"), (&ag, "ag")] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/forge/raw/proj/file/hello.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "{name} raw_file 200");
        assert_eq!(
            resp.headers()["content-type"].to_str().unwrap(),
            "text/plain",
            "{name} mime"
        );
        let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        assert_eq!(String::from_utf8_lossy(&bytes), "hi from musk", "{name} raw content");
    }

    // tree 现在含 hello.txt。
    for (app, name) in [(&hw, "hw"), (&ag, "ag")] {
        let (s, b) = send(app, "GET", "/api/forge/raw/proj/tree", None).await;
        assert_eq!(s, StatusCode::OK, "{name} tree 200");
        let names: Vec<&str> = b.as_array()
            .unwrap()
            .iter()
            .filter_map(|n| n["name"].as_str())
            .collect();
        assert!(names.contains(&"hello.txt"), "{name} tree has hello.txt: {b:?}");
    }

    // 删除上传文件 → 204;再删 → 404。
    for (app, name) in [(&hw, "hw"), (&ag, "ag")] {
        let (s, _) = send(app, "DELETE", "/api/forge/raw/proj/file/hello.txt", None).await;
        assert_eq!(s, StatusCode::NO_CONTENT, "{name} raw delete 204");
        let (s2, _) = send(app, "DELETE", "/api/forge/raw/proj/file/hello.txt", None).await;
        assert_eq!(s2, StatusCode::NOT_FOUND, "{name} raw delete 404 after");
    }
}
