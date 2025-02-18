use std::error::Error;
use std::str::FromStr;

use crate::error::MainProcessError;
use crate::utils::{get_content_data, parse_content_json, ContentSchema, Input, PropertyType};

use alloy::primitives::{Address, FixedBytes};
use axum::{http::StatusCode, response::IntoResponse, Json};
use eigenda_adapter::publish_blob;
use othentic::{init_config, send_task};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use verifier::{verify_proof_from_json, VerificationResult};

// Serialization: Convert Address to hex string
fn serialize_address<S>(address: &Address, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex = format!("{:?}", address);
    serializer.serialize_str(&hex)
}

// Deserialization: Parse hex string to Address
fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Address::from_str(&s).map_err(serde::de::Error::custom)
}

// Serialization: Convert FixedBytes<32> to hex string
fn serialize_fixed_bytes<S>(bytes: &FixedBytes<32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex = format!("{:?}", bytes);
    serializer.serialize_str(&hex)
}

// Deserialization: Parse hex string to FixedBytes<32>
fn deserialize_fixed_bytes<'de, D>(deserializer: D) -> Result<FixedBytes<32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    FixedBytes::<32>::from_str(&s).map_err(serde::de::Error::custom)
}

/// Request body for proof registration
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProofRequest {
    /// The transcript proof string
    pub transcript_proof: String,
    /// The schema string
    pub schema: String,
}

/// IP Creator information
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IPCreator {
    /// Name of the creator
    pub name: String,
    /// Blockchain address of the creator
    #[serde(
        serialize_with = "serialize_address",
        deserialize_with = "deserialize_address"
    )]
    pub address: Address,
    /// Contribution percentage
    pub contribution_percent: i32,
}

/// IP Media information
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IPMedia {
    /// Name of the media
    pub name: String,
    /// URL of the media
    pub url: String,
    /// MIME type of the media
    pub mimetype: String,
}

/// IP Attribute information
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IPAttribute {
    /// Attribute key
    pub key: String,
    /// Attribute value
    pub value: Value,
}

/// IP Metadata
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IPAMeta {
    /// Title of the IP
    pub title: String,
    /// Description of the IP
    pub description: String,
    /// Type of IP
    pub ip_type: String,
    /// List of creators
    pub creators: Vec<IPCreator>,
    /// List of media files
    pub media: Vec<IPMedia>,
    /// List of attributes
    pub attributes: Vec<IPAttribute>,
    /// List of tags
    pub tags: Vec<String>,
}

/// NFT Metadata
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct NFTMeta {
    /// Name of the NFT
    pub name: String,
    /// Description of the NFT
    pub description: String,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Optional animation URL
    pub animation_url: Option<String>,
    /// Optional audio URL
    pub audio_url: Option<String>,
    /// Optional text content
    pub text_content: Option<String>,
}

/// Proof of Task response
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProofofTask {
    /// The transcript proof
    pub transcript_proof: String,
    /// Transaction hash
    #[serde(
        serialize_with = "serialize_fixed_bytes",
        deserialize_with = "deserialize_fixed_bytes"
    )]
    pub transaction_hash: FixedBytes<32>,
    /// IP identifier address
    #[serde(
        serialize_with = "serialize_address",
        deserialize_with = "deserialize_address"
    )]
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
        PropertyType::Url => get_content_data(transcript, &content.metadata.property.key)
            .map_err(|e| MainProcessError::BadFileUse(e.to_string()))?,
    };

    match content.metadata.property.property_type {
        PropertyType::File | PropertyType::Json => Ok(format!("https://ipfs.io/ipfs/{}", file_up)),
        PropertyType::Url => Ok(file_up),
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
            address: content.address,
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

#[utoipa::path(
    post,
    path = "/register",
    tag = "Task",
    request_body = ProofRequest,
    responses(
        (status = 200, description = "Successfully registered IP", body = ProofofTask),
        (status = 400, description = "Bad request - Invalid proof or schema"),
        (status = 500, description = "Internal server error")
    )
)]
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

    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY is not set in environment variables");
    let rpc_url = std::env::var("OTHENTIC_CLIENT_RPC_ADDRESS")
        .expect("ETH_RPC_URL is not set in environment variables");
    init_config(private_key, rpc_url);
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
