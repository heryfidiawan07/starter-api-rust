use axum::{
    extract::{Path, State},
    response::Response,
};

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::response;
use crate::services::permission_service;
use crate::AppState;

async fn check_perm(auth: &AuthUser, pool: &sqlx::AnyPool) -> Result<(), AppError> {
    if auth.is_root { return Ok(()); }
    let has = permission_service::check_user_permission(pool, &auth.user_id, "permission:index").await?;
    if !has { return Err(AppError::Forbidden("Permission denied".into())); }
    Ok(())
}

pub async fn index(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool).await?;
    let perms = permission_service::find_all(&state.pool).await?;
    Ok(response::ok("Permissions retrieved", serde_json::json!(permission_service::serialize_all(&perms))))
}

pub async fn tree(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool).await?;
    let tree = permission_service::find_tree(&state.pool).await?;
    Ok(response::ok("Permission tree retrieved", serde_json::json!(tree)))
}

pub async fn by_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(role_id): Path<String>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool).await?;
    let perms = permission_service::find_by_role_id(&state.pool, &role_id).await?;
    Ok(response::ok("Role permissions retrieved", serde_json::json!(permission_service::serialize_all(&perms))))
}
