use crate::{
    error::MainProcessError,
    service::{
        __path_register_ip_from_transcript, register_ip_from_transcript, IPAMeta, IPAttribute,
        IPCreator, IPMedia, NFTMeta, ProofRequest, ProofofTask,
    },
};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use eyre::Report;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Define API documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        health_check,
        register_ip_from_transcript
    ),
    components(
        schemas(
            ProofRequest,
            ProofofTask,
            IPCreator,
            IPMedia,
            IPAttribute,
            IPAMeta,
            NFTMeta
        )
    ),
    tags(
        (name = "General", description = "Get data about this execution node."),
        (name = "Task", description = "Perform action on this execution node.")
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/",
    tag = "General",
    responses(
      (status = 200, description = "Welcome message with current server time", body = String, example = json!("Hello, from this Forge! The time on this server is: 2025-02-06T14:13:12.297568201Z"), content_type = "text/plain")
    )
)]
async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        format!(
            "Hello, from this Forge! The time on this server is: {:?}",
            chrono::Utc::now()
        ),
    )
}

#[utoipa::path(
    get,
    path = "/healthcheck",
    tag = "General",
    responses(
        (status = 200, description = "Health check endpoint", body = String, example = json!("Ok"), content_type = "text/plain")
    )
)]
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Ok")
}

pub async fn run_server() -> Result<(), MainProcessError> {
    let router = Router::new()
        .route("/", get(root))
        .route("/healthcheck", get(health_check))
        .route("/register", post(register_ip_from_transcript))
        // Add Swagger UI routes
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

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
