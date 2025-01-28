use std::error::Error;
use verifier::{verify_proof_from_json, VerificationResult};

fn verify_ip_from_proof() {}

fn verify(pre_image: Input) -> Result<VerificationResult, Box<dyn Error>> {
    let result = verify_proof_from_json(pre_image.transcript_proof.as_str())?;
    Ok(VerificationResult {
        server_name: result.server_name,
        time: result.time,
        sent_data: result.sent_data,
        received_data: result.received_data,
    })
}
