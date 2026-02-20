use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
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
    status: &'static str,
}

#[derive(Serialize)]
struct RegisterResponse {
    id: Uuid,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
}

type ErrorResponse = (StatusCode, &'static str);

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDto>,
) -> anyhow::Result<Json<RegisterResponse>, ErrorResponse> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password;
    if email.is_empty() || password.len() < 6 {
        return Err((StatusCode::BAD_REQUEST, "invalid email or password"));
    }

    tracing::info!("register user = {}", email);

    let user_id = uuid::Uuid::new_v4();
    let password_hash = security::hash_password(&password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "hash error"))?;

    let res = users_repo::create_user(&state, &user_id, &email, &password_hash).await;

    match res {
        Ok(_) => Ok(Json(RegisterResponse { id: user_id })),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "email already exist"))
        }
        Err(e) => {
            error!("SQL create_user error: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "db error"))
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> anyhow::Result<Json<LoginResponse>, ErrorResponse> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password;

    let user = match users_repo::find_by_email(&state, &email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((StatusCode::UNAUTHORIZED, "user not found"));
        }
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "db error"));
        }
    };

    let ok = security::verify_password(&password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "verify error"))?;

    if !ok {
        return Err((StatusCode::UNAUTHORIZED, "not correct password"));
    }

    let access_token = security::generate_jwt(&state.config.jwt_secret, &user.id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "jwt error"))?;

    Ok(Json(LoginResponse { access_token }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
}
