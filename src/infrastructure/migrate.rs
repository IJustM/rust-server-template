use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
};

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn run(pool: &PgPool) -> anyhow::Result<(), MigrateError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
