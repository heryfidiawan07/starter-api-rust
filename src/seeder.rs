use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use sqlx::AnyPool;
use uuid::Uuid;

pub async fn seed(pool: &AnyPool) -> Result<(), sqlx::Error> {
    seed_permissions(pool).await?;
    seed_root_user(pool).await?;
    Ok(())
}

async fn save_permission(pool: &AnyPool, parent_id: Option<&str>, label: &str, name: &str, ptype: &str, route: Option<&str>, order: i32) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO permissions (id, parent_id, label, name, type, route, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(parent_id)
    .bind(label)
    .bind(name)
    .bind(ptype)
    .bind(route)
    .bind(order)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(id)
}

async fn seed_permissions(pool: &AnyPool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM permissions")
        .fetch_one(pool).await?;
    if count.0 > 0 { return Ok(()); }

    let main_id = save_permission(pool, None, "Main", "main", "category", None, 1).await?;
    let settings_id = save_permission(pool, None, "Settings", "settings", "category", None, 2).await?;

    save_permission(pool, Some(&main_id), "Dashboard", "dashboard", "menu", Some("/dashboard"), 1).await?;

    let admin_id = save_permission(pool, Some(&settings_id), "Administrator", "administrator", "category", None, 1).await?;
    let user_id = save_permission(pool, Some(&admin_id), "User", "user", "menu", Some("/settings/users"), 1).await?;
    let role_id = save_permission(pool, Some(&admin_id), "Role", "role", "menu", Some("/settings/roles"), 2).await?;
    let perm_id = save_permission(pool, Some(&admin_id), "Permission", "permission", "menu", Some("/settings/permissions"), 3).await?;

    for (i, action) in ["index", "show", "create", "edit", "delete"].iter().enumerate() {
        let label = capitalize(action);
        save_permission(pool, Some(&user_id), &label, &format!("user:{}", action), "action", None, i as i32 + 1).await?;
        save_permission(pool, Some(&role_id), &label, &format!("role:{}", action), "action", None, i as i32 + 1).await?;
    }
    save_permission(pool, Some(&perm_id), "Index", "permission:index", "action", None, 1).await?;
    Ok(())
}

async fn seed_root_user(pool: &AnyPool) -> Result<(), sqlx::Error> {
    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = 'root@example.com'")
        .fetch_one(pool).await?;
    if exists.0 > 0 { return Ok(()); }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let pw = hash("password", DEFAULT_COST).unwrap();
    sqlx::query(
        "INSERT INTO users (id, name, email, username, password, is_root, is_active, email_verified, created_at, updated_at) VALUES (?, ?, ?, ?, ?, true, true, true, ?, ?)"
    )
    .bind(&id)
    .bind("Root")
    .bind("root@example.com")
    .bind("root")
    .bind(&pw)
    .bind(now)
    .bind(now)
    .execute(pool).await?;
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
