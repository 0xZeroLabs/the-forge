mod disperser {
    tonic::include_proto!("disperser");
}

mod common {
    tonic::include_proto!("common");
}

use std::time::Duration;

use disperser::disperser_client::DisperserClient;
use disperser::{BlobStatus, BlobStatusRequest, DisperseBlobRequest, RetrieveBlobRequest};

/// Publishes a blob to the Disperser.
/// Returns the request ID after verifying blob status
pub async fn publish_blob(d: String) -> Result<String, Box<dyn std::error::Error>> {
    let endpoint = "https://disperser-holesky.eigenda.xyz:443";
    let mut client = DisperserClient::connect(endpoint).await.unwrap();

    let data = d.as_bytes().to_vec();
    println!("{:?}", data);
    let request = tonic::Request::new(DisperseBlobRequest {
        data: data.clone(),
        custom_quorum_numbers: vec![],
        account_id: "".to_string(),
    });

    let response = client.disperse_blob(request).await.unwrap();
    let request_id = response.into_inner().request_id;
    println!(
        "Blob dispersion completed, request id '{}'",
        base64::encode(&request_id)
    );

    // Poll GetBlobStatus with a timeout of 5 minutes
    let start_time = tokio::time::Instant::now();
    let timeout_duration = Duration::from_secs(5 * 60); // 5 minutes
    let mut blob_status = BlobStatus::Unknown;

    while tokio::time::Instant::now().duration_since(start_time) < timeout_duration {
        let status_request = tonic::Request::new(BlobStatusRequest {
            request_id: request_id.clone(),
        });

        println!("Checking for blob confirmation...");
        let reply = client.get_blob_status(status_request).await?.into_inner();
        blob_status = reply.status();

        match blob_status {
            BlobStatus::Confirmed | BlobStatus::Finalized => {
                println!("Blob processing completed.");
                break;
            }
            _ => {
                println!("Blob not yet confirmed, sleeping for 5 seconds.");
                tokio::time::sleep(Duration::from_secs(5)).await
            }
        }
    }

    if blob_status != BlobStatus::Confirmed && blob_status != BlobStatus::Finalized {
        return Err("Timeout reached without confirmation or finalization of the blob.".into());
    }

    Ok(base64::encode(&request_id))
}

/// Retrieves a blob from the Disperser.
/// Takes a request ID string and returns the blob data as a string.
pub async fn retrieve_blob(request_id: String) -> Result<String, Box<dyn std::error::Error>> {
    let endpoint = "https://disperser-holesky.eigenda.xyz:443";
    let mut client = DisperserClient::connect(endpoint).await.unwrap();

    let request_id_bytes = base64::decode(request_id)?;

    let status_request = tonic::Request::new(BlobStatusRequest {
        request_id: request_id_bytes,
    });

    let reply = client.get_blob_status(status_request).await?.into_inner();
    let info = reply.info.as_ref().ok_or("info is None")?;
    let proof = info
        .blob_verification_proof
        .as_ref()
        .ok_or("blob_verification_proof is None")?;
    let metadata = proof
        .batch_metadata
        .as_ref()
        .ok_or("batch_metadata is None")?;
    let batch_header_hash = metadata.batch_header_hash.clone();
    let blob_index = proof.blob_index;

    let request = tonic::Request::new(RetrieveBlobRequest {
        batch_header_hash,
        blob_index,
    });

    let response = client.retrieve_blob(request).await;
    if let Ok(r) = response {
        Ok(String::from_utf8(r.into_inner().data).unwrap())
    } else {
        Err("Failed to retrieve blob value".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_verify_data() {
        let original = String::from("00") + r#"{ "message": "hello world" }"#;
        let request_id = tokio::runtime::Runtime::new().unwrap().block_on(async {
            println!("Publishing blob: {}", original);
            publish_blob(original.to_string()).await.unwrap()
        });

        println!("Request ID: {}", request_id);

        // Now use request_id to retrieve the blob
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { retrieve_blob(request_id).await.unwrap() });

        assert_eq!(result, original);
    }
}
