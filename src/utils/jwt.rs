use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub is_root: bool,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

fn pad_secret(secret: &str) -> Vec<u8> {
    let bytes = secret.as_bytes();
    if bytes.len() >= 32 {
        bytes.to_vec()
    } else {
        let mut padded = bytes.to_vec();
        padded.resize(32, b'0');
        padded
    }
}

pub fn generate_access_token(user_id: &str, is_root: bool, secret: &str, expire_minutes: i64) -> AppResult<String> {
    let now = Utc::now();
    let claims = AccessClaims {
        sub: user_id.to_string(),
        is_root,
        exp: (now + Duration::minutes(expire_minutes)).timestamp(),
        iat: now.timestamp(),
    };
    let key = EncodingKey::from_secret(&pad_secret(secret));
    encode(&Header::default(), &claims, &key).map_err(AppError::Jwt)
}

pub fn generate_refresh_token(user_id: &str, secret: &str, expire_minutes: i64) -> AppResult<String> {
    let now = Utc::now();
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp: (now + Duration::minutes(expire_minutes)).timestamp(),
        iat: now.timestamp(),
    };
    let key = EncodingKey::from_secret(&pad_secret(secret));
    encode(&Header::default(), &claims, &key).map_err(AppError::Jwt)
}

pub fn parse_access_token(token: &str, secret: &str) -> AppResult<AccessClaims> {
    let key = DecodingKey::from_secret(&pad_secret(secret));
    let mut val = Validation::new(Algorithm::HS256);
    val.validate_exp = true;
    decode::<AccessClaims>(token, &key, &val)
        .map(|d| d.claims)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))
}
