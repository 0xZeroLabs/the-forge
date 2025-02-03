use alloy::{
    consensus::Transaction,
    hex,
    network::EthereumWallet,
    primitives::{Address, FixedBytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::{Log, TransactionReceipt},
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
    let _ = dotenv().map_err(|e| format!("Failed to read .env file: {}", e));
    let private_key = std::env::var("PRIVATE_KEY")
        .map_err(|e| format!("Failed to get PRIVATE_KEY: {}", e))
        .unwrap();
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;
    let contract_address = std::env::var("CONTRACT_ADDRESS").map_err(|e| {
        println!("Failed to get CONTRACT_ADDRESS: {}", e);
        e
    })?;

    let signer: PrivateKeySigner = private_key.parse().expect("should parse private key");
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    let contract = IPARegistrar::new(Address::from_str(&contract_address)?, provider.clone());

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

pub async fn get_transaction_receipt(hash: FixedBytes<32>) -> Result<TransactionReceipt> {
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let tx_receipt = provider
        .get_transaction_receipt(hash)
        .await?
        .ok_or_else(|| eyre::eyre!("Receipt not found"))?;

    Ok(tx_receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;
    use alloy::providers::Provider;
    use mockall::mock;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_register_ip() {
        // Set up test environment variables
        std::env::set_var(
            "PRIVATE_KEY",
            "0x5837da14afbb1229eae18d07700b0e6ec2b6407384a08ef25fde3d55ea846962",
        );
        std::env::set_var(
            "CONTRACT_ADDRESS",
            "0xCf02ba0ed580f4184f70a9F430f1663597462011",
        );

        let address = Address::from_str("0x37ad3634C2fA851847d19256F42ec0eD5ad6e7b4").unwrap();
        println!("address: {:?}", address);
        let name = "Test IP".to_string();
        let ip_metadata_uri =
            "https://ipfs.io/ipfs/QmZHfQdFA2cb3ASdmeGS5K6rZjz65osUddYMURDx21bT73".to_string();
        let ip_metadata = "{'title':'My IP Asset','description':'This is a test IP asset','ipType':'','relationships':[],'createdAt':'','watermarkImg':'https://picsum.photos/200','creators':[],'media':[],'attributes':[{'key':'Rarity','value':'Legendary'}],'tags':[]}".to_string();
        let nft_metadata_uri =
            "https://ipfs.io/ipfs/QmRL5PcK66J1mbtTZSw1nwVqrGxt98onStx6LgeHTDbEey".to_string();
        let nft_metadata = "{'name':'Test NFT','description':'This is a test NFT','image':'https://picsum.photos/200'}".to_string();

        // Add a pre-check to verify the contract
        let rpc_url = "https://rpc.odyssey.storyrpc.io".parse().unwrap();
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let code = provider.get_code_at(address).await.unwrap();
        println!("Contract code exists: {}", !code.is_empty());

        // Execute register_ip function
        let result = register_ip(
            address,
            name,
            ip_metadata_uri,
            ip_metadata,
            nft_metadata_uri,
            nft_metadata,
        )
        .await;

        // Assert the result
        assert!(result.is_ok(), "register_ip should return Ok result");

        if let Ok(ip_data) = result {
            // Verify the returned data structure
            assert_ne!(
                ip_data.ipid,
                Address::ZERO,
                "IPID should not be zero address"
            );
            assert_ne!(
                ip_data.hash,
                FixedBytes::ZERO,
                "Transaction hash should not be zero"
            );
        }
    }
}
