use axum::{
    Json, Router,
    extract::State,
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
struct TokenResponse {
    access_token: String,
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
struct ErrorResponse {
    error: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDto>,
) -> Result<Json<RegisterResponse>, Json<ErrorResponse>> {
    let email = payload.email.trim().to_lowercase();
    let password = payload.password;
    if email.is_empty() || password.len() < 6 {
        return Err(Json(ErrorResponse {
            error: "invalid email or password",
        }));
    }

    tracing::info!("register user = {}", email);

    let user_id = uuid::Uuid::new_v4();
    let password_hash = security::hash_password(&password).map_err(|_| {
        Json(ErrorResponse {
            error: "hash error",
        })
    })?;

    let res = users_repo::create_user(&state, &user_id, &email, &password_hash).await;

    match res {
        Ok(_) => Ok(Json(RegisterResponse { id: user_id })),
        Err(e) => {
            error!("SQL create_user error: {:?}", e);
            Err(Json(ErrorResponse { error: "db error" }))
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
}
