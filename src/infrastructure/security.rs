use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

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
struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
}

pub fn generate_jwt(secret: &str, user_id: &Uuid) -> anyhow::Result<String> {
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::minutes(15);

    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.unix_timestamp(),
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

#[allow(dead_code)]
pub fn verify_jwt(secret: &str, token: &str) -> anyhow::Result<Uuid> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    let id = Uuid::parse_str(&data.claims.sub)?;
    Ok(id)
}
