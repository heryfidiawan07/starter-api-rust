use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::AnyPool;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::{AppError, AppResult};
use crate::models::password_reset_token::PasswordResetToken;
use crate::models::refresh_token::RefreshToken;
use crate::models::social_account::SocialAccount;
use crate::models::user::User;
use crate::models::user::serialize_user;
use crate::utils::jwt::{generate_access_token, generate_refresh_token};
use crate::utils::mail;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RevokeTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthRequest {
    pub token: String,
}

async fn token_pair(pool: &AnyPool, user: &User, cfg: &Config) -> AppResult<Value> {
    let access = generate_access_token(&user.id, user.is_root, &cfg.jwt_secret, cfg.jwt_access_expire)?;
    let refresh = generate_refresh_token(&user.id, &cfg.jwt_secret, cfg.jwt_refresh_expire)?;
    let now = Utc::now();
    let expires_at = now + Duration::minutes(cfg.jwt_refresh_expire);
    let rt_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&rt_id)
    .bind(&user.id)
    .bind(&refresh)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await?;

    let role_val = if let Some(rid) = &user.role_id {
        let r = sqlx::query_as::<_, crate::models::role::Role>(
            "SELECT id, name, description, created_at, updated_at, deleted_at FROM roles WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(rid)
        .fetch_optional(pool)
        .await?
        .map(|r| crate::models::role::serialize_role(&r, None));
        r
    } else {
        None
    };

    Ok(json!({
        "access_token": access,
        "refresh_token": refresh,
        "token_type": "Bearer",
        "expires_in": cfg.jwt_access_expire * 60,
        "user": serialize_user(user, role_val, &cfg.app_url),
    }))
}

pub async fn register(pool: &AnyPool, cfg: &Config, req: RegisterRequest) -> AppResult<Value> {
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
    let email_verified = !cfg.email_verification_required;
    sqlx::query(
        "INSERT INTO users (id, name, email, username, password, is_root, is_active, email_verified, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, false, true, ?, ?, ?)"
    )
    .bind(&id).bind(&req.name).bind(&req.email).bind(&req.username)
    .bind(&pw).bind(email_verified).bind(now).bind(now)
    .execute(pool).await?;

    if cfg.email_verification_required {
        let token = Uuid::new_v4().to_string();
        let expires = now + Duration::hours(24);
        let tid = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO password_reset_tokens (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(&tid).bind(&id).bind(&token).bind(expires).bind(now)
            .execute(pool).await?;
        let _ = mail::send_verification_email(cfg, &req.email, &req.name, &token).await;
        return Ok(json!({"message": "Registration successful. Please verify your email."}));
    }

    let user = find_user_by_id(pool, &id).await?;
    token_pair(pool, &user, cfg).await
}

pub async fn login(pool: &AnyPool, cfg: &Config, req: LoginRequest) -> AppResult<Value> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, username, password, photo, is_root, is_active, email_verified, role_id, created_at, updated_at, deleted_at
         FROM users WHERE email = ? AND deleted_at IS NULL"
    )
    .bind(&req.email)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    let pw = user.password.clone().unwrap_or_default();
    let valid = verify(&req.password, &pw).map_err(|_| AppError::Unauthorized("Invalid email or password".into()))?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }
    if !user.is_active {
        return Err(AppError::Forbidden("Account is inactive".into()));
    }
    if cfg.email_verification_required && !user.email_verified {
        return Err(AppError::Forbidden("Please verify your email first".into()));
    }
    token_pair(pool, &user, cfg).await
}

pub async fn logout(pool: &AnyPool, user_id: &str) -> AppResult<()> {
    let now = Utc::now();
    sqlx::query("UPDATE refresh_tokens SET revoked_at = ? WHERE user_id = ? AND revoked_at IS NULL")
        .bind(now).bind(user_id)
        .execute(pool).await?;
    Ok(())
}

pub async fn refresh(pool: &AnyPool, cfg: &Config, req: RefreshTokenRequest) -> AppResult<Value> {
    let rt = sqlx::query_as::<_, RefreshToken>(
        "SELECT id, user_id, token, expires_at, used_at, revoked_at, created_at FROM refresh_tokens WHERE token = ?"
    )
    .bind(&req.refresh_token)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".into()))?;

    if !rt.is_valid() {
        return Err(AppError::Unauthorized("Refresh token expired or revoked".into()));
    }
    let now = Utc::now();
    sqlx::query("UPDATE refresh_tokens SET used_at = ? WHERE id = ?")
        .bind(now).bind(&rt.id)
        .execute(pool).await?;

    let user = find_user_by_id(pool, &rt.user_id).await?;
    token_pair(pool, &user, cfg).await
}

pub async fn revoke_token(pool: &AnyPool, req: RevokeTokenRequest) -> AppResult<()> {
    let now = Utc::now();
    sqlx::query("UPDATE refresh_tokens SET revoked_at = ? WHERE token = ?")
        .bind(now).bind(&req.refresh_token)
        .execute(pool).await?;
    Ok(())
}

pub async fn forgot_password(pool: &AnyPool, cfg: &Config, req: ForgotPasswordRequest) -> AppResult<()> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, username, password, photo, is_root, is_active, email_verified, role_id, created_at, updated_at, deleted_at
         FROM users WHERE email = ? AND deleted_at IS NULL"
    )
    .bind(&req.email)
    .fetch_optional(pool)
    .await?;

    if let Some(u) = user {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires = now + Duration::hours(1);
        let tid = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO password_reset_tokens (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(&tid).bind(&u.id).bind(&token).bind(expires).bind(now)
            .execute(pool).await?;
        let _ = mail::send_password_reset_email(cfg, &u.email, &u.name, &token).await;
    }
    Ok(())
}

pub async fn reset_password(pool: &AnyPool, req: ResetPasswordRequest) -> AppResult<()> {
    let prt = sqlx::query_as::<_, PasswordResetToken>(
        "SELECT id, user_id, token, expires_at, used_at, created_at FROM password_reset_tokens WHERE token = ?"
    )
    .bind(&req.token)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid reset token".into()))?;

    if !prt.is_valid() {
        return Err(AppError::BadRequest("Reset token expired or already used".into()));
    }
    let pw = hash(&req.new_password, DEFAULT_COST).map_err(|e| AppError::Internal(e.into()))?;
    let now = Utc::now();
    sqlx::query("UPDATE users SET password = ?, updated_at = ? WHERE id = ?")
        .bind(&pw).bind(now).bind(&prt.user_id)
        .execute(pool).await?;
    sqlx::query("UPDATE password_reset_tokens SET used_at = ? WHERE id = ?")
        .bind(now).bind(&prt.id)
        .execute(pool).await?;
    Ok(())
}

pub async fn verify_email(pool: &AnyPool, token: &str) -> AppResult<()> {
    let prt = sqlx::query_as::<_, PasswordResetToken>(
        "SELECT id, user_id, token, expires_at, used_at, created_at FROM password_reset_tokens WHERE token = ?"
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid verification token".into()))?;

    if !prt.is_valid() {
        return Err(AppError::BadRequest("Verification token expired or already used".into()));
    }
    let now = Utc::now();
    sqlx::query("UPDATE users SET email_verified = true, updated_at = ? WHERE id = ?")
        .bind(now).bind(&prt.user_id)
        .execute(pool).await?;
    sqlx::query("UPDATE password_reset_tokens SET used_at = ? WHERE id = ?")
        .bind(now).bind(&prt.id)
        .execute(pool).await?;
    Ok(())
}

pub async fn change_password(pool: &AnyPool, user_id: &str, req: ChangePasswordRequest) -> AppResult<()> {
    let user = find_user_by_id(pool, user_id).await?;
    let pw = user.password.clone().unwrap_or_default();
    let valid = verify(&req.current_password, &pw).map_err(|_| AppError::BadRequest("Current password incorrect".into()))?;
    if !valid {
        return Err(AppError::BadRequest("Current password incorrect".into()));
    }
    let new_pw = hash(&req.new_password, DEFAULT_COST).map_err(|e| AppError::Internal(e.into()))?;
    let now = Utc::now();
    sqlx::query("UPDATE users SET password = ?, updated_at = ? WHERE id = ?")
        .bind(&new_pw).bind(now).bind(user_id)
        .execute(pool).await?;
    Ok(())
}

pub async fn oauth_google(pool: &AnyPool, cfg: &Config, req: OAuthRequest) -> AppResult<Value> {
    if cfg.google_client_id.is_empty() {
        return Err(AppError::BadRequest("Google OAuth not configured".into()));
    }
    let client = Client::new();
    let resp: serde_json::Value = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(&req.token)
        .send().await.map_err(|e| AppError::Internal(e.into()))?
        .json().await.map_err(|e| AppError::Internal(e.into()))?;

    let provider_id = resp["sub"].as_str().ok_or_else(|| AppError::Unauthorized("Invalid Google token".into()))?.to_string();
    let email = resp["email"].as_str().unwrap_or("").to_string();
    let name = resp["name"].as_str().unwrap_or("").to_string();
    handle_oauth(pool, cfg, "google", &provider_id, &email, &name).await
}

pub async fn oauth_facebook(pool: &AnyPool, cfg: &Config, req: OAuthRequest) -> AppResult<Value> {
    if cfg.facebook_client_id.is_empty() {
        return Err(AppError::BadRequest("Facebook OAuth not configured".into()));
    }
    let client = Client::new();
    let url = format!("https://graph.facebook.com/me?fields=id,name,email&access_token={}", req.token);
    let resp: serde_json::Value = client
        .get(&url)
        .send().await.map_err(|e| AppError::Internal(e.into()))?
        .json().await.map_err(|e| AppError::Internal(e.into()))?;

    let provider_id = resp["id"].as_str().ok_or_else(|| AppError::Unauthorized("Invalid Facebook token".into()))?.to_string();
    let email = resp["email"].as_str().unwrap_or("").to_string();
    let name = resp["name"].as_str().unwrap_or("").to_string();
    handle_oauth(pool, cfg, "facebook", &provider_id, &email, &name).await
}

async fn handle_oauth(pool: &AnyPool, cfg: &Config, provider: &str, provider_id: &str, email: &str, name: &str) -> AppResult<Value> {
    let existing_sa = sqlx::query_as::<_, SocialAccount>(
        "SELECT id, user_id, provider, provider_id, created_at, updated_at FROM social_accounts WHERE provider = ? AND provider_id = ?"
    )
    .bind(provider).bind(provider_id)
    .fetch_optional(pool).await?;

    let user_id = if let Some(sa) = existing_sa {
        sa.user_id
    } else {
        let uid = if !email.is_empty() {
            let u = sqlx::query_as::<_, User>(
                "SELECT id, name, email, username, password, photo, is_root, is_active, email_verified, role_id, created_at, updated_at, deleted_at
                 FROM users WHERE email = ? AND deleted_at IS NULL"
            )
            .bind(email).fetch_optional(pool).await?;
            if let Some(existing) = u {
                existing.id
            } else {
                let new_id = Uuid::new_v4().to_string();
                let now = Utc::now();
                sqlx::query(
                    "INSERT INTO users (id, name, email, is_root, is_active, email_verified, created_at, updated_at) VALUES (?, ?, ?, false, true, true, ?, ?)"
                )
                .bind(&new_id).bind(name).bind(email).bind(now).bind(now)
                .execute(pool).await?;
                new_id
            }
        } else {
            let new_id = Uuid::new_v4().to_string();
            let now = Utc::now();
            sqlx::query(
                "INSERT INTO users (id, name, is_root, is_active, email_verified, created_at, updated_at) VALUES (?, ?, false, true, true, ?, ?)"
            )
            .bind(&new_id).bind(name).bind(now).bind(now)
            .execute(pool).await?;
            new_id
        };
        let sa_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query("INSERT INTO social_accounts (id, user_id, provider, provider_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&sa_id).bind(&uid).bind(provider).bind(provider_id).bind(now).bind(now)
            .execute(pool).await?;
        uid
    };

    let user = find_user_by_id(pool, &user_id).await?;
    token_pair(pool, &user, cfg).await
}

async fn find_user_by_id(pool: &AnyPool, id: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, username, password, photo, is_root, is_active, email_verified, role_id, created_at, updated_at, deleted_at
         FROM users WHERE id = ? AND deleted_at IS NULL"
    )
    .bind(id)
    .fetch_optional(pool).await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
}
