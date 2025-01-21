use crate::{error::MainProcessError, service::verify_ip_from_proof};

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
            "Hello! This is a validation node for The Forge. The time on this server is: {:?}",
            chrono::Utc::now()
        ),
    )
}

pub async fn run_server() -> Result<(), MainProcessError> {
    let router = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/verify", post(verify_ip_from_proof));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2078")
        .await
        .map_err(|e| MainProcessError::Unexpected(Report::new(e)))?;

    println!(
        "Validation server running at: {:?}.",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, router)
        .await
        .map_err(|e| MainProcessError::Unexpected(Report::new(e)))?;

    Ok(())
}
