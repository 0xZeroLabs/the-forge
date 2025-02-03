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
use std::str::FromStr;
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
    let _ = dotenv().map_err(|e| {
        println!("Failed to read .env file: {}", e);
        e
    });

    let private_key = std::env::var("PRIVATE_KEY").map_err(|e| {
        println!("Failed to get PRIVATE_KEY: {}", e);
        e
    })?;

    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse().map_err(|e| {
        println!("Failed to parse RPC URL: {}", e);
        e
    })?;

    let signer: PrivateKeySigner = private_key.parse().map_err(|e| {
        println!("Failed to parse private key: {}", e);
        e
    })?;

    let contract_address = std::env::var("CONTRACT_ADDRESS").map_err(|e| {
        println!("Failed to get CONTRACT_ADDRESS: {}", e);
        e
    })?;

    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url);

    let contract = IPARegistrar::new(Address::from_str(&contract_address)?, provider.clone());

    let imetadata = IPMetadata {
        name,
        ipMetadataURI: ip_metatdata_uri,
        ipMetadata: ip_metadata,
        nftMetadataURI: nft_metadata_uri,
        nftMetadata: nft_metadata,
    };

    let tx = contract
        .register(address, imetadata.clone())
        .from(signer.address())
        .gas(5000000)
        .send()
        .await
        .map_err(|e| {
            println!("Failed to send transaction: {}", e);
            e
        })?;

    let receipt = tx.get_receipt().await.map_err(|e| {
        println!("Failed to get initial receipt: {}", e);
        e
    })?;
    let tx_hash = receipt.transaction_hash;

    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .map_err(|e| {
            println!("Failed to get transaction receipt: {}", e);
            e
        })?
        .ok_or_else(|| {
            let err = eyre::eyre!("Receipt not found");
            println!("Error: {}", err);
            err
        })?;

    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await
        .map_err(|e| {
            println!("Failed to get transaction: {}", e);
            e
        })?
        .ok_or_else(|| {
            let err = eyre::eyre!("Transaction not found");
            println!("Error: {}", err);
            err
        })?;

    let register_return = contract
        .register(address, imetadata.clone())
        .call()
        .await
        .map_err(|e| {
            println!("Failed to call register function: {}", e);
            e
        })?;

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
