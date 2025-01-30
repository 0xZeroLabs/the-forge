use alloy_primitives::{Address, FixedBytes};
use axum::{response::IntoResponse, Json};
use eigenda::retrieve_blob;
use execution::utils::Input;
use registrar::get_transaction_receipt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use verifier::{verify_proof_from_json, VerificationResult};

use crate::error::MainProcessError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofofTask {
    pub transcript_proof: String,
    pub transaction_hash: FixedBytes<32>,
    pub ip_id: Address,
}

pub async fn verify_ip_from_proof(
    Json(mut body): Json<String>,
) -> Result<impl IntoResponse, MainProcessError> {
    let proof_of_task = retrieve_blob(body).await.unwrap();
    let proof = serde_json::from_str::<ProofofTask>(&proof_of_task).unwrap();

    let verification_result = verify(Input {
        transcript_proof: proof.transcript_proof,
    })
    .map_err(|_| MainProcessError::BadTranscriptProof("Transcript proof is invalid".to_string()));

    let tx_receipt = get_transaction_receipt(proof.transaction_hash).await?;

    Ok("nothing yet".to_string())
}

fn verify(pre_image: Input) -> Result<VerificationResult, Box<dyn Error>> {
    let result = verify_proof_from_json(pre_image.transcript_proof.as_str())?;
    Ok(VerificationResult {
        server_name: result.server_name,
        time: result.time,
        sent_data: result.sent_data,
        received_data: result.received_data,
    })
}
