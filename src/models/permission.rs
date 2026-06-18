use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Permission {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub label: String,
    #[sqlx(rename = "type")]
    pub perm_type: String,
    pub route: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<Utc>>,
}

pub fn serialize_permission(p: &Permission) -> Value {
    json!({
        "id": p.id,
        "parent_id": p.parent_id,
        "name": p.name,
        "label": p.label,
        "type": p.perm_type,
        "route": p.route,
        "sort_order": p.sort_order,
        "created_at": p.created_at,
        "updated_at": p.updated_at,
    })
}

pub fn serialize_permission_tree(p: &Permission, children: Vec<Value>) -> Value {
    let mut v = serialize_permission(p);
    if !children.is_empty() {
        v["children"] = json!(children);
    }
    v
}
