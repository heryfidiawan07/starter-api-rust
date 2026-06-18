use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Value>,
}

pub fn ok(message: &str, data: Value) -> Response {
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        message: message.into(),
        data: Some(data),
        meta: None,
        errors: None,
    })).into_response()
}

pub fn ok_paged(message: &str, data: Value, meta: Value) -> Response {
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        message: message.into(),
        data: Some(data),
        meta: Some(meta),
        errors: None,
    })).into_response()
}

pub fn created(message: &str, data: Value) -> Response {
    (StatusCode::CREATED, Json(ApiResponse {
        success: true,
        message: message.into(),
        data: Some(data),
        meta: None,
        errors: None,
    })).into_response()
}

pub fn no_data(message: &str) -> Response {
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        message: message.into(),
        data: None,
        meta: None,
        errors: None,
    })).into_response()
}

pub fn page_meta(page: i64, per_page: i64, total: i64) -> Value {
    let total_page = (total as f64 / per_page as f64).ceil() as i64;
    json!({ "page": page, "per_page": per_page, "total": total, "total_page": total_page })
}
