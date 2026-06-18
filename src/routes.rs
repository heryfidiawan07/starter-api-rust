use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::{auth, permissions, roles, users};
use crate::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        .route("/api/v1/auth/revoke", post(auth::revoke_token))
        .route("/api/v1/auth/forgot-password", post(auth::forgot_password))
        .route("/api/v1/auth/reset-password", post(auth::reset_password))
        .route("/api/v1/auth/verify-email", get(auth::verify_email))
        .route("/api/v1/auth/change-password", post(auth::change_password))
        .route("/api/v1/auth/me", get(auth::me))
        .route("/api/v1/auth/oauth/google", post(auth::oauth_google))
        .route("/api/v1/auth/oauth/facebook", post(auth::oauth_facebook))
        // Profile
        .route("/api/v1/profile", put(users::update_my_profile))
        .route("/api/v1/profile/photo", post(users::upload_my_photo))
        // Users
        .route("/api/v1/users", get(users::index).post(users::create))
        .route("/api/v1/users/:id", get(users::show).put(users::update).delete(users::delete))
        .route("/api/v1/users/:id/photo", post(users::upload_photo))
        // Roles
        .route("/api/v1/roles", get(roles::index).post(roles::create))
        .route("/api/v1/roles/:id", get(roles::show).put(roles::update).delete(roles::delete))
        // Permissions
        .route("/api/v1/permissions", get(permissions::index))
        .route("/api/v1/permissions/tree", get(permissions::tree))
        .route("/api/v1/permissions/by-role/:role_id", get(permissions::by_role))
        .with_state(state)
}
