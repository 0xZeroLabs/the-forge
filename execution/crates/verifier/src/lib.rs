use elliptic_curve::pkcs8::DecodePublicKey;
use serde::Serialize;
use std::{str, time::Duration};
use tlsn_core::{
    proof::{SessionProof, TlsProof},
    ServerName,
};

/// Verifies a TLS proof and returns the verified data
pub fn verify_proof(proof_path: &str) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    // Deserialize the proof
    let proof = std::fs::read_to_string(proof_path)?;
    let proof: TlsProof = serde_json::from_str(proof.as_str())?;

    let TlsProof {
        session,
        substrings,
    } = proof;

    // Verify the session proof against the Notary's public key
    session.verify_with_default_cert_verifier(notary_pubkey())?;

    let SessionProof {
        header,
        session_info,
        ..
    } = session;

    // The time at which the session was recorded
    let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(header.time());

    // Verify the substrings proof against the session header
    let (mut sent, mut recv) = substrings.verify(&header)?;

    // Replace the bytes which the Prover chose not to disclose with 'X'
    sent.set_redacted(b'X');
    recv.set_redacted(b'X');

    Ok(VerificationResult {
        server_name: session_info.server_name,
        time,
        sent_data: String::from_utf8(sent.data().to_vec())?,
        received_data: String::from_utf8(recv.data().to_vec())?,
    })
}

/// Verifies a TLS proof from a JSON string and returns the verified data
pub fn verify_proof_from_json(
    proof_json: &str,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    // Deserialize the proof directly from the provided JSON string
    let proof: TlsProof = serde_json::from_str(proof_json)?;

    let TlsProof {
        session,
        substrings,
    } = proof;

    // Verify the session proof against the Notary's public key
    session.verify_with_default_cert_verifier(notary_pubkey())?;

    let SessionProof {
        header,
        session_info,
        ..
    } = session;

    // The time at which the session was recorded
    let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(header.time());

    // Verify the substrings proof against the session header
    let (mut sent, mut recv) = substrings.verify(&header)?;

    // Replace the bytes which the Prover chose not to disclose with 'X'
    sent.set_redacted(b'X');
    recv.set_redacted(b'X');

    Ok(VerificationResult {
        server_name: session_info.server_name,
        time,
        sent_data: String::from_utf8(sent.data().to_vec())?,
        received_data: String::from_utf8(recv.data().to_vec())?,
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
