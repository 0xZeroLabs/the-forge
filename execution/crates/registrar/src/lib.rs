use alloy::{
    consensus::Transaction,
    hex,
    network::EthereumWallet,
    primitives::{Address, FixedBytes},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolCall,
};
use dotenvy::dotenv;
use eyre::Result;
use IPARegistrar::IPMetadata;

pub struct IPData {
    pub ipid: Address,
    pub hash: FixedBytes<32>,
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IPARegistrar,
    "fixture/abi/IPARegistrar.json"
);

pub async fn register_ip(
    address: Address,
    name: String,
    ip_metatdata_uri: String,
    ip_metadata: String,
    nft_metadata_uri: String,
    nft_metadata: String,
) -> Result<IPData> {
    dotenv().map_err(|e| format!("Failed to read .env file: {}", e));
    let _ = dotenv().map_err(|e| format!("Failed to read .env file: {}", e));
    let private_key = std::env::var("PRIVATE_KEY")
        .map_err(|e| format!("Failed to get PRIVATE_KEY: {}", e))
        .unwrap();
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;

    let signer: PrivateKeySigner = private_key.parse().expect("should parse private key");
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    let contract = IPARegistrar::new(address, provider.clone());

    let imetadata = IPMetadata {
        name,
        ipMetadataURI: ip_metatdata_uri,
        ipMetadata: ip_metadata,
        nftMetadataURI: nft_metadata_uri,
        nftMetadata: nft_metadata,
    };

    // Step 1: Send the transaction and get initial response
    // The `send()` method submits the transaction to the blockchain and returns
    // a transaction handle that we can use to track its status
    let tx = contract
        .register(address, imetadata.clone())
        .from(signer.address())
        .send()
        .await?;

    // Step 2: Wait for transaction confirmation
    // `get_receipt()` waits for the transaction to be mined and returns the receipt
    // This ensures our transaction has been processed by the network
    let receipt = tx.get_receipt().await?;
    let tx_hash = receipt.transaction_hash;

    // Step 3: Get the transaction receipt
    // The receipt contains important information about the transaction execution
    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or_else(|| eyre::eyre!("Receipt not found"))?;

    // Step 4: Extract the transaction data
    // We'll need to get the transaction to access its input data
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or_else(|| eyre::eyre!("Transaction not found"))?;

    // Step 5: Get the function output from the logs or use a View call
    // Since this is a state-changing function, we need to either:
    // a) Parse event logs if the contract emits an event with the IPID
    // b) Make a separate view call to get the IPID if the contract provides a getter
    // For now, we'll use a view call right after the transaction
    // Get the return value and extract the address from it
    let register_return = contract.register(address, imetadata.clone()).call().await?;

    // The register function returns a tuple with one field (the address)
    // We need to extract that address
    let ipid = register_return._0;

    Ok(IPData {
        ipid,
        hash: tx_hash,
    })
}
