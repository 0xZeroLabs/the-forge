mod sindri;

use dotenvy::dotenv;
use serde_json::Value;
use sindri::{compile_guest_code, headers_json, prove_guest_code};

pub struct ZKProofVerificationResult {
    pub is_valid: bool,
    pub public_data: Value,
}

// Function to generate the proof
pub async fn generate_zk_proof(input: &str) -> Result<Value, String> {
    // Obtain the user's API key from the .env file.
    dotenv().map_err(|e| format!("Failed to read .env file: {}", e))?;
    let api_key: String = std::env::var("SINDRI_API_KEY")
        .map_err(|e| format!("Failed to get SINDRI_API_KEY: {}", e))?;

    // Create a headers map with the API key.
    let header = headers_json(&api_key);

    // Generate proof using the input
    let proof_json = match prove_guest_code(input, header).await {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to generate proof: {}", e)),
    };

    Ok(proof_json)
}

// Function to verify the proof directly from JSON
pub fn verify_zk_proof(proof_json: Value) -> Result<ZKProofVerificationResult, String> {
    let json_data: JsonProofData = serde_json::from_value(proof_json["proof"].clone())
        .map_err(|e| format!("Failed to parse proof data: {}", e))?;
}
