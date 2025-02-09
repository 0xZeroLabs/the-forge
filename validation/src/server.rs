use crate::service::{verify_ip_from_proof, ErrorResponse};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde_json::json;

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

pub async fn run_server() -> Result<(), ErrorResponse> {
    let router = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/task/validate", post(verify_ip_from_proof));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2078")
        .await
        .map_err(|e| {
            ErrorResponse::new(json!({}), &format!("Failed to bind TCP listener: {}", e))
        })?;

    println!(
        "Validation server running at: {:?}.",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, router)
        .await
        .map_err(|e| ErrorResponse::new(json!({}), &format!("Server error: {}", e)))?;

    Ok(())
}
