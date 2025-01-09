mod sindri;
mod utils;

use ark_bn254::{Bn254, Fr};
use dotenvy::dotenv;
use jolt::{
    jolt::vm::{rv32i_vm::RV32IJoltVM, Jolt},
    poly::commitment::hyperkzg::HyperKZG,
};
use serde_json::Value;
use sindri::{compile_guest_code, headers_json, prove_guest_code};
use utils::{deserialize_jolt_proof_data_from_base64, JsonProofData};

pub struct ZKProofVerificationResult {
    pub is_valid: bool,
    pub public_data: Value,
}

// Function to generate the proof
pub async fn generate_zk_proof(input_path: &str) -> Result<Value, String> {
    // Obtain the user's API key from the .env file.
    dotenv().map_err(|e| format!("Failed to read .env file: {}", e))?;
    let api_key: String = std::env::var("SINDRI_API_KEY")
        .map_err(|e| format!("Failed to get SINDRI_API_KEY: {}", e))?;

    // Create a headers map with the API key.
    let header = headers_json(&api_key);

    // Generate proof using the input
    let proof_json = match prove_guest_code(input_path, header).await {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to generate proof: {}", e)),
    };

    Ok(proof_json)
}

// Function to verify the proof directly from JSON
pub fn verify_zk_proof(proof_json: Value) -> Result<ZKProofVerificationResult, String> {
    let json_data: JsonProofData = serde_json::from_value(proof_json["proof"].clone())
        .map_err(|e| format!("Failed to parse proof data: {}", e))?;

    // Separate out the proof and preprocessing components
    let (jolt_proof_struct, jolt_preprocessing_struct) =
        deserialize_jolt_proof_data_from_base64::<Fr, HyperKZG<Bn254>>(json_data);

    let preprocessing = RV32IJoltVM::preprocess(
        jolt_preprocessing_struct.bytecode,
        jolt_preprocessing_struct.memory_init,
        1 << 20,
        1 << 20,
        1 << 22,
    );

    let verification_result = RV32IJoltVM::verify(
        preprocessing,
        jolt_proof_struct.proof,
        jolt_proof_struct.commitments,
        None,
    );

    let public_data = proof_json["public"].clone();

    Ok(ZKProofVerificationResult { is_valid: verification_result.is_ok(), public_data })
}
