use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Conflict(String),
    Internal(String),
    Db,
}

#[derive(Serialize)]
struct AppErrorBody {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::Db => (StatusCode::INTERNAL_SERVER_ERROR, "db error".to_string()),
        };

        let body = Json(AppErrorBody { message });

        (status, body).into_response()
    }
}
