use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub async fn create_db(database_url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("connect to database error")
}
