use std::env;
use verifier::{verify_proof, verify_proof_from_json};

fn main() {
    // Print the current working directory to see where the program is looking
    println!("Current directory: {:?}", env::current_dir().unwrap());
    match verify_proof("fixture/simple_proof.json") {
        Ok(result) => {
            println!("-------------------------------------------------------------------");
            println!(
                "Successfully verified that the bytes below came from a session with {:?} at {}.",
                result.server_name, result.time
            );
            println!("Note that the bytes which the Prover chose not to disclose are shown as X.");
            println!();
            println!("Bytes sent:");
            println!();
            println!("{}", result.sent_data);
            println!();
            println!("Bytes received:");
            println!();
            println!("{}", result.received_data);
            println!("-------------------------------------------------------------------");
        }
        Err(e) => eprintln!("Verification failed: {}", e),
    }
    // replace empty string with raw json string
    match verify_proof_from_json("") {
        Ok(result) => {
            println!("-------------------------------------------------------------------");
            println!(
                "Successfully verified that the bytes below came from a session with {:?} at {}.",
                result.server_name, result.time
            );
            println!("Note that the bytes which the Prover chose not to disclose are shown as X.");
            println!();
            println!("Bytes sent:");
            println!();
            println!("{}", result.sent_data);
            println!();
            println!("Bytes received:");
            println!();
            println!("{}", result.received_data);
            println!("-------------------------------------------------------------------");
        }
        Err(e) => eprintln!("Verification failed: {}", e),
    }
}
