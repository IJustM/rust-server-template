use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{
    data::users_repo,
    error::AppError,
    infrastructure::security::{self, AuthUser},
    state::AppState,
};

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

#[derive(Serialize)]
struct ProfileResponse {
    email: String,
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

    let access_token = security::generate_jwt(&state.config.jwt_secret, &user.id, &user.email)
        .map_err(|_| AppError::Internal("jwt error".to_string()))?;

    Ok(Json(LoginResponse { access_token }))
}

async fn profile(AuthUser(claims): AuthUser) -> anyhow::Result<Json<ProfileResponse>, AppError> {
    Ok(Json(ProfileResponse {
        email: claims.email,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/profile", get(profile))
}
