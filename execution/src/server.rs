use crate::{error::MainProcessError, service::register_ip_from_transcript};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use eyre::Report;

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Ok")
}

async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        format!(
            "Hello, from this Forge! The time on this server is: {:?}",
            chrono::Utc::now()
        ),
    )
}

pub async fn run_server() -> Result<(), MainProcessError> {
    let router = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/register", post(register_ip_from_transcript));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2077")
        .await
        .map_err(|e| MainProcessError::Unexpected(Report::new(e)))?;

    println!(
        "Forge server running at: {:?}.",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, router)
        .await
        .map_err(|e| MainProcessError::Unexpected(Report::new(e)))?;

    Ok(())
}
