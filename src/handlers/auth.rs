use axum::{
    extract::{Query, State},
    response::Response,
    Json,
};
use serde::Deserialize;

use crate::middleware::AuthUser;
use crate::response;
use crate::services::auth_service as svc;
use crate::AppState;
use crate::errors::AppError;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<svc::RegisterRequest>,
) -> Result<Response, AppError> {
    let data = svc::register(&state.pool, &state.config, req).await?;
    Ok(response::created("Registration successful", data))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<svc::LoginRequest>,
) -> Result<Response, AppError> {
    let data = svc::login(&state.pool, &state.config, req).await?;
    Ok(response::ok("Login successful", data))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    svc::logout(&state.pool, &auth.user_id).await?;
    Ok(response::no_data("Logged out successfully"))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<svc::RefreshTokenRequest>,
) -> Result<Response, AppError> {
    let data = svc::refresh(&state.pool, &state.config, req).await?;
    Ok(response::ok("Token refreshed", data))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    Json(req): Json<svc::RevokeTokenRequest>,
) -> Result<Response, AppError> {
    svc::revoke_token(&state.pool, req).await?;
    Ok(response::no_data("Token revoked"))
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<svc::ForgotPasswordRequest>,
) -> Result<Response, AppError> {
    svc::forgot_password(&state.pool, &state.config, req).await?;
    Ok(response::no_data("Password reset email sent if account exists"))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<svc::ResetPasswordRequest>,
) -> Result<Response, AppError> {
    svc::reset_password(&state.pool, req).await?;
    Ok(response::no_data("Password reset successful"))
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

pub async fn verify_email(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> Result<Response, AppError> {
    svc::verify_email(&state.pool, &q.token).await?;
    Ok(response::no_data("Email verified successfully"))
}

pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<svc::ChangePasswordRequest>,
) -> Result<Response, AppError> {
    svc::change_password(&state.pool, &auth.user_id, req).await?;
    Ok(response::no_data("Password changed successfully"))
}

pub async fn oauth_google(
    State(state): State<AppState>,
    Json(req): Json<svc::OAuthRequest>,
) -> Result<Response, AppError> {
    let data = svc::oauth_google(&state.pool, &state.config, req).await?;
    Ok(response::ok("Google OAuth successful", data))
}

pub async fn oauth_facebook(
    State(state): State<AppState>,
    Json(req): Json<svc::OAuthRequest>,
) -> Result<Response, AppError> {
    let data = svc::oauth_facebook(&state.pool, &state.config, req).await?;
    Ok(response::ok("Facebook OAuth successful", data))
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    let data = crate::services::user_service::find_by_id_value(&state.pool, &auth.user_id, &state.config.app_url).await?;
    Ok(response::ok("Profile retrieved", data))
}
