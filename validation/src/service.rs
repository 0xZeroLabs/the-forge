use alloy_primitives::{Address, FixedBytes};
use axum::{response::IntoResponse, Json};
use eigenda::retrieve_blob;
use execution::utils::{get_content_data, ContentSchema, Input};
use registrar::get_transaction_data;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use verifier::{verify_proof_from_json, VerificationResult};

use crate::error::MainProcessError;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IPCreator {
    name: String,
    address: Address,
    contribution_percent: i32,
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
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct NFTMeta {
    name: String,
    description: String,
    image_url: Option<String>,
    animation_url: Option<String>,
    audio_url: Option<String>,
    text_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofofTask {
    pub transcript_proof: String,
    pub transaction_hash: FixedBytes<32>,
    pub ip_id: Address,
    pub content_schema: ContentSchema,
}

pub async fn verify_ip_from_proof(
    Json(body): Json<String>,
) -> Result<impl IntoResponse, MainProcessError> {
    // Parse and verify proof
    let proof_of_task = retrieve_blob(body).await.unwrap();
    let proof = serde_json::from_str::<ProofofTask>(&proof_of_task).unwrap();
    let verification_result = verify(Input {
        transcript_proof: proof.transcript_proof.clone(),
    })
    .map_err(|_| MainProcessError::BadTranscriptProof("Transcript proof is invalid".to_string()));

    // Verify transaction data
    let ip_data = get_transaction_data(proof.transaction_hash).await?;
    if ip_data.owner != proof.content_schema.address {
        return Err(MainProcessError::WrongTransactionData(
            "The owner of the IP doesn't follow what was given in the content schema".to_string(),
        ));
    }

    // Fetch metadata
    let ipameta = fetch_ipameta(&ip_data.ipMetadataURI).await?;
    let nftmeta = fetch_nftmeta(&ip_data.nftMetadataURI).await?;

    // Validate metadata consistency
    validate_metadata_consistency(&proof, &ipameta, &nftmeta, &verification_result?)?;

    // TODO: handle other cases. rn these are satisfactory and can verify the execution was handled correctly
    Ok("nothing yet".to_string())
}

async fn fetch_ipameta(uri: &str) -> Result<IPAMeta, MainProcessError> {
    reqwest::get(&format!("https://ipfs.io/ipfs/{}", uri))
        .await
        .map_err(|e| MainProcessError::BadRequest(e.to_string()))?
        .json()
        .await
        .map_err(|e| MainProcessError::BadRequest(e.to_string()))
}

async fn fetch_nftmeta(uri: &str) -> Result<NFTMeta, MainProcessError> {
    reqwest::get(&format!("https://ipfs.io/ipfs/{}", uri))
        .await
        .map_err(|e| MainProcessError::BadRequest(e.to_string()))?
        .json()
        .await
        .map_err(|e| MainProcessError::BadRequest(e.to_string()))
}

fn validate_metadata_consistency(
    proof: &ProofofTask,
    ipameta: &IPAMeta,
    nftmeta: &NFTMeta,
    verification_result: &VerificationResult,
) -> Result<(), MainProcessError> {
    if proof.content_schema.name != ipameta.title {
        return Err(MainProcessError::BadContentSchema(
            "The name of the content schema doesn't match the IP metadata".to_string(),
        ));
    }

    if ipameta.title != nftmeta.name {
        return Err(MainProcessError::BadContentSchema(
            "The name of the IP metadata doesn't match the NFT metadata".to_string(),
        ));
    }

    if ipameta.attributes[0].value
        != Value::String(verification_result.server_name.as_str().to_string())
    {
        return Err(MainProcessError::BadContentSchema(
            "The server name in the IP metadata doesn't match the server name in the transcript proof".to_string(),
        ));
    }

    if ipameta.creators[0].name
        != get_content_data(verification_result, &proof.content_schema.metadata.owner).unwrap()
    {
        return Err(MainProcessError::BadContentSchema(
            "The server name in the IP metadata doesn't match the server name in the transcript proof".to_string(),
        ));
    }

    if ipameta.creators[0].address != proof.content_schema.address {
        return Err(MainProcessError::BadContentSchema(
            "The server name in the IP metadata doesn't match the server name in the transcript proof".to_string(),
        ));
    }

    Ok(())
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
