//! PLAN-104 T-05（UAT-K12 附带）— `SpecItemPayload`（HTTP upsert 请求体）
//! `file`/`tags` 字段验证：
//! ① 携带 file/tags 的新客户端载荷可反序列化（merge 技能要求条目记录
//!    source hash/commit，此前 HTTP 等价通道无法携带）；
//! ② 旧客户端四字段载荷（缺 file/tags）零破坏兼容（serde Option 容忍缺省）。
//!
//! 注：payload → `SpecItem` 的映射发生在 `auto_generated::extern_impl::
//! specs_upsert_of`（需 `State<AppState>`，属集成面，由 parity/HTTP 冒烟
//! 覆盖）；此处钉住 wire 契约本身。

use musk::auto_generated::server::{SpecItemPayload, SpecsUpsertRequest};

#[test]
fn payload_with_file_and_tags_deserializes() {
    let body = r#"{
        "section": "designs",
        "item": {
            "id": "password-generator",
            "title": "本地密码生成器",
            "content": "docs/specs/password-generator.md 的派生索引",
            "status": "draft",
            "file": "docs/specs/password-generator.md",
            "tags": ["PLAN-001", "delivered", "commit:89530d2"]
        }
    }"#;
    let req: SpecsUpsertRequest = serde_json::from_str(body).expect("full payload must parse");
    assert_eq!(req.section, "designs");
    assert_eq!(req.item.file.as_deref(), Some("docs/specs/password-generator.md"));
    assert_eq!(
        req.item.tags.as_deref(),
        Some(&["PLAN-001".to_string(), "delivered".to_string(), "commit:89530d2".to_string()][..])
    );
}

#[test]
fn legacy_four_field_payload_still_parses() {
    // 旧客户端（PLAN-104 之前形态）缺 file/tags —— Option 容忍缺省，零破坏。
    let body = r#"{
        "section": "designs",
        "item": {
            "id": "legacy",
            "title": "Legacy entry",
            "content": "docs/specs/legacy.md",
            "status": "draft"
        }
    }"#;
    let req: SpecsUpsertRequest = serde_json::from_str(body).expect("legacy payload must parse");
    assert_eq!(req.item.id, "legacy");
    assert!(req.item.file.is_none(), "absent file must deserialize to None");
    assert!(req.item.tags.is_none(), "absent tags must deserialize to None");
}

#[test]
fn tags_null_tolerated() {
    // 显式 null 与缺省同义（部分客户端会回显 null）。
    let body = r#"{
        "section": "reviews",
        "item": { "id": "r1", "title": "t", "content": "c", "status": "approved", "file": null, "tags": null }
    }"#;
    let req: SpecsUpsertRequest = serde_json::from_str(body).expect("null file/tags must parse");
    let payload: SpecItemPayload = req.item;
    assert!(payload.file.is_none());
    assert!(payload.tags.is_none());
}
