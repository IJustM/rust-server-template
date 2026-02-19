mod infrastructure;
mod presentation;

use axum::{Router, serve};
use tokio::net::TcpListener;

use crate::presentation::{cors, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    infrastructure::tracing::init_tracing();

    let cfg = infrastructure::config::Config::from_env().expect("invalid config");

    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("starting server on {}", addr);

    let listener = TcpListener::bind(addr).await.expect("bind listener error");

    let app = Router::new().layer(cors::cors());
    let app = routes::with_routes(app);

    serve(listener, app).await.expect("serve error");

    Ok(())
}
