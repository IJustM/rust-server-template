use sqlx::{Pool, Postgres};

use crate::infrastructure::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub config: Config,
}
