use crate::error::MainProcessError;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProofRequest {
    transcript_proof: String,
    schema: String,
}

pub async fn register_ip_from_transcript(
    Json(payload): Json<ProofRequest>,
) -> Result<impl IntoResponse, MainProcessError> {
    let transcript_proof = payload.transcript_proof;
    let schema = payload.schema;

    Ok((
        StatusCode::OK,
        "IP sucessfully registered from transcript proof.",
    ))
}
