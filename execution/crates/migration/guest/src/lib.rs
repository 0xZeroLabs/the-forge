#![no_main]
pub mod utils;
pub use utils::{Input, Output};

use verifier::verify_proof_from_json;
use bytemuck;
use getrandom::Error;
use getrandom::register_custom_getrandom;
use rand::{Rng, thread_rng};

pub fn zkvm_random(dest: &mut [u8]) -> Result<(), Error> {
    let mut rng = thread_rng();
    rng.fill(dest);
    Ok(())
}
register_custom_getrandom!(zkvm_random);

#[jolt::provable]
fn migrate(pre_image: Input) -> Output {
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

    let output = match verify_proof_from_json(pre_image.transcript.unwrap()) {
        Ok(result) => {
            print_verification_result(&result);
            Output {
                server_name: result.server_name,
                time: result.time,
                sent_data: result.sent_data,
                received_data: result.recieved_data,
            }
        }
        Err(e) => {
            eprintln!("Verification failed: {}", e);
            Output::default()
        }
    };

    output
}
