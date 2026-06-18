use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::json;

use crate::AppState;
use crate::utils::jwt::parse_access_token;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub is_root: bool,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"success": false, "message": "Missing or invalid token"})),
            ));
        };

        match parse_access_token(token, &state.config.jwt_secret) {
            Ok(claims) => Ok(AuthUser {
                user_id: claims.sub,
                is_root: claims.is_root,
            }),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"success": false, "message": "Invalid or expired token"})),
            )),
        }
    }
}
