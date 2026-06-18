use axum::{
    extract::{Multipart, Path, Query, State},
    response::Response,
    Json,
};
use serde::Deserialize;

use crate::errors::AppError;
use crate::middleware::AuthUser;
use crate::response;
use crate::services::{permission_service, user_service as svc};
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
    check_perm(&auth, &state.pool, "user:index").await?;
    let page = q.page.unwrap_or(1);
    let per_page = q.per_page.unwrap_or(10);
    let (items, total) = svc::find_all(&state.pool, q.search.as_deref(), page, per_page, &state.config.app_url).await?;
    let meta = response::page_meta(page, per_page, total);
    Ok(response::ok_paged("Users retrieved", serde_json::json!(items), meta))
}

pub async fn show(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "user:show").await?;
    let data = svc::find_by_id_value(&state.pool, &id, &state.config.app_url).await?;
    Ok(response::ok("User retrieved", data))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<svc::CreateUserRequest>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "user:create").await?;
    let data = svc::create(&state.pool, req, &state.config.app_url).await?;
    Ok(response::created("User created", data))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<svc::UpdateUserRequest>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "user:edit").await?;
    let data = svc::update(&state.pool, &id, req, &state.config.app_url).await?;
    Ok(response::ok("User updated", data))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "user:delete").await?;
    svc::delete(&state.pool, &id).await?;
    Ok(response::no_data("User deleted"))
}

pub async fn upload_photo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    check_perm(&auth, &state.pool, "user:edit").await?;
    let data = svc::update_photo(&state.pool, &id, multipart, &state.config.storage_path, &state.config.app_url).await?;
    Ok(response::ok("Photo updated", data))
}

pub async fn update_my_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<svc::UpdateProfileRequest>,
) -> Result<Response, AppError> {
    let data = svc::update_profile(&state.pool, &auth.user_id, req, &state.config.app_url).await?;
    Ok(response::ok("Profile updated", data))
}

pub async fn upload_my_photo(
    State(state): State<AppState>,
    auth: AuthUser,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let data = svc::update_photo(&state.pool, &auth.user_id, multipart, &state.config.storage_path, &state.config.app_url).await?;
    Ok(response::ok("Photo updated", data))
}
