use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sqlx::AnyPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::role::{Role, serialize_role};
use crate::services::permission_service;

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<String>>,
}

pub async fn find_all(pool: &AnyPool, search: Option<&str>, page: i64, per_page: i64) -> AppResult<(Vec<Value>, i64)> {
    let offset = (page - 1) * per_page;
    let pattern = format!("%{}%", search.unwrap_or(""));
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL AND name LIKE ?"
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await?;

    let roles = sqlx::query_as::<_, Role>(
        "SELECT id, name, description, created_at, updated_at, deleted_at
         FROM roles WHERE deleted_at IS NULL AND name LIKE ? ORDER BY name ASC LIMIT ? OFFSET ?"
    )
    .bind(&pattern)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for r in &roles {
        let perms = permission_service::find_by_role_id(pool, &r.id).await?;
        let perm_vals = permission_service::serialize_all(&perms);
        result.push(serialize_role(r, Some(perm_vals)));
    }
    Ok((result, total.0))
}

pub async fn find_by_id(pool: &AnyPool, id: &str) -> AppResult<(Role, Vec<Value>)> {
    let role = sqlx::query_as::<_, Role>(
        "SELECT id, name, description, created_at, updated_at, deleted_at FROM roles WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Role not found".into()))?;

    let perms = permission_service::find_by_role_id(pool, id).await?;
    let perm_vals = permission_service::serialize_all(&perms);
    Ok((role, perm_vals))
}

pub async fn create(pool: &AnyPool, req: CreateRoleRequest) -> AppResult<Value> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO roles (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    if let Some(pids) = &req.permission_ids {
        sync_permissions(pool, &id, pids).await?;
    }

    let (role, perms) = find_by_id(pool, &id).await?;
    Ok(serialize_role(&role, Some(perms)))
}

pub async fn update(pool: &AnyPool, id: &str, req: UpdateRoleRequest) -> AppResult<Value> {
    let (role, _) = find_by_id(pool, id).await?;
    let now = Utc::now();
    let name = req.name.unwrap_or(role.name);
    let desc = req.description.or(role.description);

    sqlx::query("UPDATE roles SET name = ?, description = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(&desc)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    if let Some(pids) = &req.permission_ids {
        sync_permissions(pool, id, pids).await?;
    }

    let (updated, perms) = find_by_id(pool, id).await?;
    Ok(serialize_role(&updated, Some(perms)))
}

pub async fn delete(pool: &AnyPool, id: &str) -> AppResult<()> {
    find_by_id(pool, id).await?;
    let in_use: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE role_id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    if in_use.0 > 0 {
        return Err(AppError::Conflict("Role is assigned to users and cannot be deleted".into()));
    }
    let now = Utc::now();
    sqlx::query("UPDATE roles SET deleted_at = ? WHERE id = ?")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn sync_permissions(pool: &AnyPool, role_id: &str, permission_ids: &[String]) -> AppResult<()> {
    sqlx::query("DELETE FROM role_permissions WHERE role_id = ?")
        .bind(role_id)
        .execute(pool)
        .await?;
    for pid in permission_ids {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES (?, ?)")
            .bind(role_id)
            .bind(pid)
            .execute(pool)
            .await?;
    }
    Ok(())
}
