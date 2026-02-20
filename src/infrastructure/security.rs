use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{extract::FromRequestParts, http::header};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub fn hash_password(plain: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(plain.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

pub fn verify_password(plain: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(plain.as_bytes(), &parsed).is_ok())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
}

pub fn generate_jwt(secret: &str, user_id: &Uuid, user_email: &str) -> anyhow::Result<String> {
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_string(),
        email: user_email.to_string(),
        exp: exp.unix_timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("encode error");

    Ok(token)
}

pub fn verify_jwt(secret: &str, token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}

pub struct AuthUser(pub Claims);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AppError::Unauthorized(
                "missing authorization header".to_string(),
            ))?;

        let header = header
            .to_str()
            .map_err(|_| AppError::Unauthorized("invalid header".to_string()))?;

        if !header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("invalid scheme".to_string()));
        }

        let token = &header[7..];

        let claims = verify_jwt(&state.config.jwt_secret, token)
            .map_err(|_| AppError::Unauthorized("invalid token".to_string()))?;

        Ok(AuthUser(claims))
    }
}
