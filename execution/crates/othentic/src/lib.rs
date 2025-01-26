use alloy::providers::Provider;
use alloy::signers::Signer;
use alloy::{
    network::{EthereumWallet, NetworkWallet},
    primitives::{keccak256, Address, Bytes},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol_types::sol,
};
use alloy_sol_types::SolValue;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::error::Error;

sol! {
    struct TaskMessage {
        string proof_of_task;
        bytes data;
        address performer;
        uint16 task_definition_id;
    }
}

pub async fn send_task(
    proof_of_task: String,
    data: String,
    task_definition_id: u16,
) -> Result<(), Box<dyn Error>> {
    dotenv().map_err(|e| format!("Failed to read .env file: {}", e));
    let private_key = std::env::var("PRIVATE_KEY")
        .map_err(|e| format!("Failed to get PRIVATE_KEY: {}", e))
        .unwrap();
    let rpc_url = "".parse()?;

    let signer: PrivateKeySigner = private_key.parse().expect("should parse private key");
    let wallet = EthereumWallet::from(signer.clone());
    let address = wallet.default_signer().address();

    let data_bytes = Bytes::from(data.as_bytes().to_vec());

    let message = TaskMessage {
        proof_of_task: proof_of_task.clone(),
        data: data_bytes.clone(),
        performer: address,
        task_definition_id,
    };

    let encoded = message.abi_encode();
    let message_hash = keccak256(&encoded);

    let signature = signer.sign_message(&message_hash[..]).await?;
    let serialized_sig = format!("{:?}", signature);

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let params = vec![
        serde_json::to_value(proof_of_task)?,
        serde_json::to_value(data_bytes)?,
        serde_json::to_value(task_definition_id)?,
        serde_json::to_value(address)?,
        serde_json::to_value(serialized_sig)?,
    ];

    let response = provider.raw_request("sendTask".into(), params).await?;

    println!("API response: {:?}", response);
    Ok(())
}
