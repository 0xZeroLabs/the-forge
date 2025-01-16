use crate::error::MainProcessError;

use alloy::primitives::Address;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct ProofRequest {
    pub transcript_proof: String,
    pub schema: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IPCreator {
    name: String,
    address: Address,
    contribution_percent: i32,
    // description: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IPMedia {
    name: String,
    url: String,
    mimetype: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IPAttribute {
    key: String,
    value: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IPAMeta {
    title: String,
    description: String,
    ip_type: String,
    creators: Vec<IPCreator>,
    media: Vec<IPMedia>,
    attributes: Vec<IPAttribute>,
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
