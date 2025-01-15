use dotenvy::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Debug)]
pub struct FileUploadParams {
    pub file_url: String,
    pub file_name: String,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PinataResponse {
    pub ipfs_hash: String,
    pub pin_size: u64,
    pub timestamp: String,
}

pub async fn upload_file_from_url(
    params: FileUploadParams,
) -> Result<PinataResponse, Box<dyn Error>> {
    let file_url = params.file_url.as_str();
    let file_name = params.file_name.as_str();
    let file_type = params.file_type.as_str();

    dotenv().ok();

    let api_key = std::env::var("PINATA_API_KEY")?;
    let api_secret = std::env::var("PINATA_API_SECRET")?;

    // Fetch the file from the URL
    let response = reqwest::get(file_url).await?;
    let file_bytes = response.bytes().await?;

    // Convert `file_bytes` to `Vec<u8>`
    let file_vec = file_bytes.to_vec();

    // Prepare the request to Pinata
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("pinata_api_key", HeaderValue::from_str(&api_key)?);
    headers.insert("pinata_secret_api_key", HeaderValue::from_str(&api_secret)?);

    // Create the multipart form
    let form = multipart::Form::new()
        .part(
            "file",
            multipart::Part::bytes(file_vec)
                .file_name(file_name) // e.g., picture.png
                .mime_str(file_type)?, // e.g., image/png,
        )
        .text("pinataOptions", r#"{"cidVersion": 1}"#);

    // Send the request to Pinata
    let res = client
        .post("https://api.pinata.cloud/pinning/pinFileToIPFS")
        .headers(headers)
        .multipart(form)
        .send()
        .await?;

    // Check the response
    if res.status().is_success() {
        let response: PinataResponse = res.json().await?;
        println!("File uploaded successfully: {:?}", response);
        Ok(response)
    } else {
        let error_text = res.text().await?;
        eprintln!("Failed to upload file: {}", error_text);
        Err(error_text.into())
    }
}

pub async fn upload_json(json_data: Value) -> Result<PinataResponse, Box<dyn Error>> {
    dotenv().ok();

    let api_key = std::env::var("PINATA_API_KEY")?;
    let api_secret = std::env::var("PINATA_API_SECRET")?;

    // Prepare the request to Pinata
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("pinata_api_key", HeaderValue::from_str(&api_key)?);
    headers.insert("pinata_secret_api_key", HeaderValue::from_str(&api_secret)?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    // Send the request to Pinata
    let res = client
        .post("https://api.pinata.cloud/pinning/pinJSONToIPFS")
        .headers(headers)
        .json(&json_data)
        .send()
        .await?;

    // Check the response
    if res.status().is_success() {
        let response: PinataResponse = res.json().await?;
        println!("JSON uploaded successfully: {:?}", response);
        Ok(response)
    } else {
        let error_text = res.text().await?;
        eprintln!("Failed to upload JSON: {}", error_text);
        Err(error_text.into())
    }
}
