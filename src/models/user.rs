use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub photo: Option<String>,
    pub is_root: bool,
    pub is_active: bool,
    pub email_verified: bool,
    pub role_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<Utc>>,
}

pub fn serialize_user(u: &User, role: Option<Value>, app_url: &str) -> Value {
    let photo_url = u.photo.as_ref().map(|p| format!("{}/storage/photos/{}", app_url, p));
    json!({
        "id": u.id,
        "name": u.name,
        "email": u.email,
        "username": u.username,
        "photo": photo_url,
        "is_root": u.is_root,
        "is_active": u.is_active,
        "email_verified": u.email_verified,
        "role_id": u.role_id,
        "role": role,
        "created_at": u.created_at,
        "updated_at": u.updated_at,
    })
}
