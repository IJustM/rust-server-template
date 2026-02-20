use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{data::users_repo, infrastructure::security, state::AppState};

#[derive(Deserialize)]
struct RegisterDto {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginDto {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    id: Uuid,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
}

enum AppError {
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

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDto>,
) -> anyhow::Result<Json<RegisterResponse>, AppError> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password;
    if email.is_empty() || password.len() < 6 {
        return Err(AppError::BadRequest(
            "invalid email or password".to_string(),
        ));
    }

    tracing::info!("register user = {}", email);

    let user_id = uuid::Uuid::new_v4();
    let password_hash = security::hash_password(&password)
        .map_err(|_| AppError::Internal("hash error".to_string()))?;

    let res = users_repo::create_user(&state, &user_id, &email, &password_hash).await;

    match res {
        Ok(_) => Ok(Json(RegisterResponse { id: user_id })),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(AppError::Conflict("email already exist".to_string()))
        }
        Err(e) => {
            error!("SQL create_user error: {:?}", e);
            Err(AppError::Db)
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> anyhow::Result<Json<LoginResponse>, AppError> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password;

    let user = match users_repo::find_by_email(&state, &email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(AppError::Unauthorized("user not found".to_string()));
        }
        Err(_) => {
            return Err(AppError::Db);
        }
    };

    let ok = security::verify_password(&password, &user.password_hash)
        .map_err(|_| AppError::Internal("verify error".to_string()))?;

    if !ok {
        return Err(AppError::Unauthorized("not correct password".to_string()));
    }

    let access_token = security::generate_jwt(&state.config.jwt_secret, &user.id)
        .map_err(|_| AppError::Internal("jwt error".to_string()))?;

    Ok(Json(LoginResponse { access_token }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
}
