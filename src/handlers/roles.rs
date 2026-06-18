use axum::{
    extract::{Path, Query, State},
    response::Response,
    Json,
};
use serde::Deserialize;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::response;
use crate::services::{permission_service, role_service as svc};
use crate::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

async fn check_perm(auth: &AuthUser, pool: &sqlx::AnyPool, perm: &str) -> Result<(), AppError> {
    if auth.is_root { return Ok(()); }
    let has = permission_service::check_user_permission(pool, &auth.user_id, perm).await?;
    if !has { return Err(AppError::Forbidden("Permission denied".into())); }
    Ok(())
}

pub async fn index(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "role:index").await?;
    let page = q.page.unwrap_or(1);
    let per_page = q.per_page.unwrap_or(10);
    let (items, total) = svc::find_all(&state.pool, q.search.as_deref(), page, per_page).await?;
    let meta = response::page_meta(page, per_page, total);
    Ok(response::ok_paged("Roles retrieved", serde_json::json!(items), meta))
}

pub async fn show(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "role:show").await?;
    let (role, perms) = svc::find_by_id(&state.pool, &id).await?;
    Ok(response::ok("Role retrieved", crate::models::role::serialize_role(&role, Some(perms))))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<svc::CreateRoleRequest>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "role:create").await?;
    let data = svc::create(&state.pool, req).await?;
    Ok(response::created("Role created", data))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<svc::UpdateRoleRequest>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "role:edit").await?;
    let data = svc::update(&state.pool, &id, req).await?;
    Ok(response::ok("Role updated", data))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "role:delete").await?;
    svc::delete(&state.pool, &id).await?;
    Ok(response::no_data("Role deleted"))
}
