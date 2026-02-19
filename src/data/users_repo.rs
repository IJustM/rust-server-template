use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
}

pub async fn create_user(
    state: &AppState,
    id: &Uuid,
    email: &str,
    password_hash: &str,
) -> anyhow::Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .execute(&state.pool)
    .await?;

    Ok(())
}
