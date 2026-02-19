use axum::{
    Json, Router,
    routing::{Route, get},
};
use serde::{Deserialize, Serialize};

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

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn with_routes(router: Router) -> Router {
    router.route("/health", get(health))
}
