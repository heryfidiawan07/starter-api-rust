use axum::{
    extract::State,
    response::Response,
};
use serde::Serialize;
use sqlx::FromRow;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::response;
use crate::services::permission_service;
use crate::AppState;

#[derive(Debug, FromRow, Serialize)]
struct RoleLookup {
    pub id: String,
    pub name: String,
}

pub async fn get_lookup_roles(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Response, AppError> {
    let roles = sqlx::query_as::<_, RoleLookup>(
        "SELECT id, name FROM roles WHERE deleted_at IS NULL ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(response::ok("Roles lookup retrieved", serde_json::json!(roles)))
}

pub async fn get_lookup_permissions(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Response, AppError> {
    let perms = permission_service::find_all(&state.pool).await?;
    Ok(response::ok(
        "Permissions lookup retrieved",
        serde_json::json!(permission_service::serialize_all(&perms)),
    ))
}
