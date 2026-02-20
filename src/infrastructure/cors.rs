use axum::http::{HeaderValue, Method, header};
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn cors(cors_origin: &str) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<HeaderValue>()
                .expect("CORS origin parse error"),
        )
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .expose_headers([header::CONTENT_TYPE])
        .max_age(Duration::from_secs(600))
}
