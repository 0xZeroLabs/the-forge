use std::error::Error;

use crate::error::MainProcessError;
use crate::utils::{get_content_data, parse_content_json, ContentSchema, Input};

use alloy::primitives::Address;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use verifier::{verify_proof_from_json, VerificationResult};

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

#[derive(Serialize, Deserialize)]
struct NFTMeta {
    name: String,
    description: String,
    image: String,
}

pub async fn register_ip_from_transcript(
    Json(body): Json<ProofRequest>,
) -> Result<impl IntoResponse, MainProcessError> {
    let transcript_proof = body.transcript_proof;
    let schema = body.schema;

fn verify(pre_image: Input) -> Result<VerificationResult, Box<dyn Error>> {
    // fn print_verification_result(result: &verifier::VerificationResult) {
    //     println!("-------------------------------------------------------------------");
    //     println!(
    //         "Successfully verified that the bytes below came from a session with {:?} at {}.",
    //         result.server_name, result.time
    //     );
    //     println!("Note that the bytes which the Prover chose not to disclose are shown as X.");
    //     println!();
    //     println!("Bytes sent:\n\n{}\n", result.sent_data);
    //     println!("Bytes received:\n\n{}", result.received_data);
    //     println!("-------------------------------------------------------------------");
    // }

    let result =
        verify_proof_from_json(pre_image.transcript_proof.as_str()).expect("Verification failed");

    // print_verification_result(&result);

    Ok(VerificationResult {
        server_name: result.server_name,
        time: result.time,
        sent_data: result.sent_data,
        received_data: result.received_data,
    })
}
