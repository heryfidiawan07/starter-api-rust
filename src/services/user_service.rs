use axum::extract::Multipart;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sqlx::AnyPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::role::Role;
use crate::models::user::{User, serialize_user};
use crate::utils::upload;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
    pub password: String,
    pub role_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub role_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub username: Option<String>,
}

async fn get_role(pool: &AnyPool, role_id: Option<&str>) -> Option<Value> {
    let id = role_id?;
    let r = sqlx::query_as::<_, Role>(
        "SELECT id, name, description, created_at, updated_at, deleted_at FROM roles WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .ok()??;
    Some(crate::models::role::serialize_role(&r, None))
}

pub async fn find_all(pool: &AnyPool, search: Option<&str>, page: i64, per_page: i64, app_url: &str) -> AppResult<(Vec<Value>, i64)> {
    let offset = (page - 1) * per_page;
    let pattern = format!("%{}%", search.unwrap_or(""));
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL AND (name LIKE ? OR email LIKE ?)"
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_one(pool)
    .await?;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email, username, password, photo, is_root, is_active, email_verified, role_id, created_at, updated_at, deleted_at
         FROM users WHERE deleted_at IS NULL AND (name LIKE ? OR email LIKE ?) ORDER BY name ASC LIMIT ? OFFSET ?"
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for u in &users {
        let role = get_role(pool, u.role_id.as_deref()).await;
        result.push(serialize_user(u, role, app_url));
    }
    Ok((result, total.0))
}

pub async fn find_by_id_value(pool: &AnyPool, id: &str, app_url: &str) -> AppResult<Value> {
    let u = find_by_id(pool, id).await?;
    let role = get_role(pool, u.role_id.as_deref()).await;
    Ok(serialize_user(&u, role, app_url))
}

pub async fn find_by_id(pool: &AnyPool, id: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, username, password, photo, is_root, is_active, email_verified, role_id, created_at, updated_at, deleted_at
         FROM users WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn create(pool: &AnyPool, req: CreateUserRequest, app_url: &str) -> AppResult<Value> {
    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ? AND deleted_at IS NULL")
        .bind(&req.email)
        .fetch_one(pool)
        .await?;
    if exists.0 > 0 {
        return Err(AppError::Conflict("Email already registered".into()));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let pw = hash(&req.password, DEFAULT_COST).map_err(|e| AppError::Internal(e.into()))?;
    sqlx::query(
        "INSERT INTO users (id, name, email, username, password, is_root, is_active, email_verified, role_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, false, ?, false, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.username)
    .bind(&pw)
    .bind(req.is_active.unwrap_or(true))
    .bind(&req.role_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    find_by_id_value(pool, &id, app_url).await
}

pub async fn update(pool: &AnyPool, id: &str, req: UpdateUserRequest, app_url: &str) -> AppResult<Value> {
    let u = find_by_id(pool, id).await?;
    let now = Utc::now();
    let name = req.name.unwrap_or(u.name);
    let email = req.email.unwrap_or(u.email);
    let username = req.username.or(u.username);
    let role_id = req.role_id.or(u.role_id);
    let is_active = req.is_active.unwrap_or(u.is_active);
    let pw = if let Some(p) = req.password {
        hash(&p, DEFAULT_COST).map_err(|e| AppError::Internal(e.into()))?
    } else {
        u.password.unwrap_or_default()
    };
    sqlx::query(
        "UPDATE users SET name=?, email=?, username=?, password=?, role_id=?, is_active=?, updated_at=? WHERE id=?"
    )
    .bind(&name).bind(&email).bind(&username).bind(&pw)
    .bind(&role_id).bind(is_active).bind(now).bind(id)
    .execute(pool).await?;
    find_by_id_value(pool, id, app_url).await
}

pub async fn delete(pool: &AnyPool, id: &str) -> AppResult<()> {
    find_by_id(pool, id).await?;
    let now = Utc::now();
    sqlx::query("UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ?")
        .bind(now).bind(now).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn update_photo(pool: &AnyPool, user_id: &str, multipart: Multipart, storage_path: &str, app_url: &str) -> AppResult<Value> {
    let u = find_by_id(pool, user_id).await?;
    let filename = upload::save_photo(multipart, storage_path).await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    if let Some(old) = &u.photo {
        upload::delete_photo(storage_path, old).await;
    }
    let now = Utc::now();
    sqlx::query("UPDATE users SET photo = ?, updated_at = ? WHERE id = ?")
        .bind(&filename).bind(now).bind(user_id)
        .execute(pool).await?;
    find_by_id_value(pool, user_id, app_url).await
}

pub async fn update_profile(pool: &AnyPool, user_id: &str, req: UpdateProfileRequest, app_url: &str) -> AppResult<Value> {
    let u = find_by_id(pool, user_id).await?;
    let now = Utc::now();
    let name = req.name.unwrap_or(u.name);
    let username = req.username.or(u.username);
    sqlx::query("UPDATE users SET name = ?, username = ?, updated_at = ? WHERE id = ?")
        .bind(&name).bind(&username).bind(now).bind(user_id)
        .execute(pool).await?;
    find_by_id_value(pool, user_id, app_url).await
}
