mod data;
mod domain;
mod infrastructure;
mod presentation;
mod state;

use axum::{Router, serve};
use tokio::net::TcpListener;

use crate::{
    presentation::{cors, routes},
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    infrastructure::tracing::init_tracing();

    let config = infrastructure::config::Config::from_env().expect("invalid config");

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("starting server on {}", addr);

    let pool = infrastructure::db::create_db(&config.database_url).await;

    infrastructure::migrate::run(&pool)
        .await
        .expect("migrations failed");

    let state = AppState { pool, config };

    let listener = TcpListener::bind(addr).await.expect("bind listener error");

    let app = Router::<AppState>::new()
        .merge(routes::router())
        .with_state(state)
        .layer(cors::cors());

    serve(listener, app).await.expect("serve error");

    Ok(())
}
