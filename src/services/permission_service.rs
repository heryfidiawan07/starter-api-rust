use sqlx::AnyPool;

use crate::errors::{AppError, AppResult};
use crate::models::permission::{Permission, serialize_permission, serialize_permission_tree};
use serde_json::Value;

pub async fn find_all(pool: &AnyPool) -> AppResult<Vec<Permission>> {
    let rows = sqlx::query_as::<_, Permission>(
        "SELECT id, parent_id, name, label, type as perm_type, route, sort_order, created_at, updated_at, deleted_at
         FROM permissions WHERE deleted_at IS NULL ORDER BY sort_order ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn find_tree(pool: &AnyPool) -> AppResult<Vec<Value>> {
    let all = find_all(pool).await?;
    Ok(build_tree(&all, None))
}

fn build_tree(all: &[Permission], parent_id: Option<&str>) -> Vec<Value> {
    all.iter()
        .filter(|p| p.parent_id.as_deref() == parent_id)
        .map(|p| {
            let children = build_tree(all, Some(&p.id));
            serialize_permission_tree(p, children)
        })
        .collect()
}

pub async fn find_by_role_id(pool: &AnyPool, role_id: &str) -> AppResult<Vec<Permission>> {
    let rows = sqlx::query_as::<_, Permission>(
        "SELECT p.id, p.parent_id, p.name, p.label, p.type as perm_type, p.route, p.sort_order, p.created_at, p.updated_at, p.deleted_at
         FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         WHERE rp.role_id = ? AND p.deleted_at IS NULL"
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn check_user_permission(pool: &AnyPool, user_id: &str, permission_name: &str) -> AppResult<bool> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         JOIN users u ON u.role_id = rp.role_id
         WHERE u.id = ? AND p.name = ? AND p.deleted_at IS NULL AND u.deleted_at IS NULL"
    )
    .bind(user_id)
    .bind(permission_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(c,)| c > 0).unwrap_or(false))
}

pub fn serialize_all(perms: &[Permission]) -> Vec<Value> {
    perms.iter().map(serialize_permission).collect()
}
