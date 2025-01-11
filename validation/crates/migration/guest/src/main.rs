#![no_main]
pub mod utils;
pub use utils::Output;

use getrandom::register_custom_getrandom;
use getrandom::Error;
use rand::{thread_rng, Rng};
use verifier::verify_proof_from_json;

pub fn zkvm_random(dest: &mut [u8]) -> Result<(), Error> {
    let mut rng = thread_rng();
    rng.fill(dest);
    Ok(())
}
register_custom_getrandom!(zkvm_random);

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let transcript = sp1_zkvm::io::read::<String>();

    fn print_verification_result(result: &verifier::VerificationResult) {
        println!("-------------------------------------------------------------------");
        println!(
            "Successfully verified that the bytes below came from a session with {:?} at {}.",
            result.server_name, result.time
        );
        println!("Note that the bytes which the Prover chose not to disclose are shown as X.");
        println!();
        println!("Bytes sent:\n\n{}\n", result.sent_data);
        println!("Bytes received:\n\n{}", result.received_data);
        println!("-------------------------------------------------------------------");
    }

    let result = verify_proof_from_json(transcript.as_str()).expect("Verification failed");

    print_verification_result(&result);

    sp1_zkvm::io::commit(&result);
}
