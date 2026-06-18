use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<Utc>>,
}

pub fn serialize_role(r: &Role, permissions: Option<Vec<Value>>) -> Value {
    json!({
        "id": r.id,
        "name": r.name,
        "description": r.description,
        "permissions": permissions,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
    })
}
