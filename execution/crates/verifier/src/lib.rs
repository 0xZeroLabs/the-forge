use elliptic_curve::pkcs8::DecodePublicKey;
use serde::Serialize;
use std::{str, time::Duration};
use tlsn_core::{
    connection::ServerName,
    presentation::{Presentation, PresentationOutput},
    signing::VerifyingKey,
    CryptoProvider,
};

/// Verifies a TLS proof from a JSON string and returns the verified data
pub fn verify_proof_from_json(
    proof_json: &str,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    // Deserialize the proof directly from the provided JSON string
    let presentation: Presentation = serde_json::from_str(proof_json)?;
    let provider = CryptoProvider::default();

    let VerifyingKey {
        alg: _,
        data: key_data,
    } = presentation.verifying_key();

    if hex::encode(key_data) != notary_pubkey().to_string() {
        return Err("Notary public key doesn't match".into());
    }

    // Verify the presentation
    let PresentationOutput {
        server_name,
        connection_info,
        transcript,
        ..
    } = presentation.verify(&provider).unwrap();

    // The time at which the connection was started
    let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(connection_info.time);

    let server_name = server_name.unwrap();
    let mut partial_transcript = transcript.unwrap();
    // Set the unauthenticated bytes so they are distinguishable.
    partial_transcript.set_unauthed(b'~');

    Ok(VerificationResult {
        server_name,
        time,
        sent_data: String::from_utf8_lossy(partial_transcript.sent_unsafe()).to_string(),
        received_data: String::from_utf8_lossy(partial_transcript.received_unsafe()).to_string(),
    })
}

/// Struct to hold the verification results
#[derive(Debug, Serialize)]
pub struct VerificationResult {
    pub server_name: ServerName,
    #[serde(serialize_with = "serialize_datetime")]
    pub time: chrono::DateTime<chrono::Utc>,
    pub sent_data: String,
    pub received_data: String,
}

/// Returns a Notary pubkey trusted by this Verifier
fn notary_pubkey() -> p256::PublicKey {
    let pem_file = str::from_utf8(include_bytes!("../../../fixture/notary/notary.pub")).unwrap();
    p256::PublicKey::from_public_key_pem(pem_file).unwrap()
}

/// Custom serialization function for DateTime
fn serialize_datetime<S>(
    date: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.to_rfc3339())
}
