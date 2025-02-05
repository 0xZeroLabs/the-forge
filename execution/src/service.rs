use std::error::Error;
use std::str::FromStr;

use crate::error::MainProcessError;
use crate::utils::{get_content_data, parse_content_json, ContentSchema, Input, PropertyType};

use alloy::primitives::{Address, FixedBytes};
use axum::{http::StatusCode, response::IntoResponse, Json};
use eigenda_adapter::publish_blob;
use othentic::send_task;
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
}

async fn upload_file_to_ipfs(
    transcript: &VerificationResult,
    content: &ContentSchema,
) -> Result<String, MainProcessError> {
    let file_up = match content.metadata.property.property_type {
        PropertyType::File => {
            pinata::upload_file_from_url(pinata::FileUploadParams {
                file_url: get_content_data(transcript, &content.metadata.property.key).unwrap(),
                file_name: content.metadata.property.key.clone(),
                file_type: content.metadata.property.mime.clone(),
            })
            .await
            .map_err(|e| MainProcessError::BadFileUse(e.to_string()))
            .unwrap()
            .ipfs_hash
        }
        PropertyType::Json => {
            pinata::upload_json(
                Value::from_str(
                    &get_content_data(transcript, &content.metadata.property.key).unwrap(),
                )
                .unwrap(),
            )
            .await
            .map_err(|e| MainProcessError::BadFileUse(e.to_string()))
            .unwrap()
            .ipfs_hash
        }
        PropertyType::URL => get_content_data(transcript, &content.metadata.property.key)
            .map_err(|e| MainProcessError::BadFileUse(e.to_string()))?,
    };

    match content.metadata.property.property_type {
        PropertyType::File | PropertyType::Json => Ok(format!("https://ipfs.io/ipfs/{}", file_up)),
        PropertyType::URL => Ok(file_up),
    }
}

async fn create_and_upload_metadata(
    content: &ContentSchema,
    file_url: &str,
    transcript: &VerificationResult,
) -> Result<(String, String, String, String), MainProcessError> {
    let ipameta = IPAMeta {
        title: content.name.clone(),
        media: vec![IPMedia {
            name: content.name.clone(),
            mimetype: content.metadata.property.mime.clone(),
            url: file_url.to_string(),
        }],
        description: content.metadata.property.description.clone(),
        attributes: vec![IPAttribute {
            key: "Source".to_string(),
            value: Value::String(transcript.server_name.as_str().to_string()),
        }],
        ip_type: content.metadata.property.tags[0].clone(),
        creators: vec![IPCreator {
            address: content.address.clone(),
            contribution_percent: 100,
            name: get_content_data(transcript, &content.metadata.owner).unwrap(),
        }],
        tags: content.metadata.property.tags.clone(),
    };

    let nftmeta = NFTMeta {
        name: content.name.clone(),
        description: content.metadata.property.description.clone(),
        image_url: Some(file_url.to_string()),
        audio_url: None,
        animation_url: None,
        text_content: None,
    };

    let ipameta_up =
        pinata::upload_json(Value::from_str(&serde_json::to_string(&ipameta).unwrap()).unwrap())
            .await
            .map_err(|e| MainProcessError::BadRequest(e.to_string()))?; // same deal
    let nftmeta_up =
        pinata::upload_json(Value::from_str(&serde_json::to_string(&nftmeta).unwrap()).unwrap())
            .await
            .map_err(|e| MainProcessError::BadRequest(e.to_string()))?; // same deal

    Ok((
        serde_json::to_string(&ipameta).unwrap(),
        serde_json::to_string(&nftmeta).unwrap(),
        format!("https://ipfs.io/ipfs/{}", ipameta_up.ipfs_hash),
        format!("https://ipfs.io/ipfs/{}", nftmeta_up.ipfs_hash),
    ))
}

pub async fn register_ip_from_transcript(
    Json(body): Json<ProofRequest>,
) -> Result<impl IntoResponse, MainProcessError> {
    let content = parse_content_json(body.schema.as_str()).unwrap();

    let transcript = verify(Input {
        transcript_proof: body.transcript_proof.clone(),
    })
    .map_err(|_| {
        MainProcessError::BadTranscriptProof("Transcript verification failed".to_string())
    })?;

    if content.url != transcript.server_name.as_str() {
        return Err(MainProcessError::BadContentSchema(
            "Host name does not match".to_string(),
        ));
    }

    let file_url = upload_file_to_ipfs(&transcript, &content).await?;
    let (ipameta, nftmeta, ipameta_url, nftmeta_url) =
        create_and_upload_metadata(&content, &file_url, &transcript).await?;

    let regip = registrar::register_ip(
        content.address,
        content.name,
        ipameta_url,
        ipameta,
        nftmeta_url,
        nftmeta,
        content.app_id,
    )
    .await
    .map_err(|e| MainProcessError::BadRequest(e.to_string()))?;

    let proof = ProofofTask {
        transcript_proof: body.transcript_proof,
        transaction_hash: regip.hash,
        ip_id: regip.ipid,
    };
    let req_id = publish_blob(format!("00{}", serde_json::to_string(&proof).unwrap()))
        .await
        .unwrap();
    send_task(req_id, 0).await.unwrap();

    Ok((StatusCode::OK, serde_json::to_string(&proof).unwrap()))
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
