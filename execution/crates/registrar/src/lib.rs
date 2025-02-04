use alloy::{
    network::EthereumWallet,
    primitives::{Address, FixedBytes},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolEvent,
};
use dotenvy::dotenv;
use eyre::Result;
use std::str::FromStr;
use ForgeRegistrar::{IPMetadata, IPRegistered};

pub struct IPData {
    pub ipid: Address,
    pub hash: FixedBytes<32>,
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ForgeRegistrar,
    "fixture/abi/ForgeRegistrar.json"
);

pub async fn register_ip(
    address: Address,
    name: String,
    ip_metatdata_uri: String,
    ip_metadata: String,
    nft_metadata_uri: String,
    nft_metadata: String,
    app_id: String,
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

    let contract = ForgeRegistrar::new(Address::from_str(&contract_address)?, provider.clone());

    let imetadata = IPMetadata {
        name,
        ipMetadataURI: ip_metatdata_uri,
        ipMetadata: ip_metadata,
        nftMetadataURI: nft_metadata_uri,
        nftMetadata: nft_metadata,
    };

    let tx = contract
        .register(address, imetadata.clone(), app_id.clone())
        .from(signer.address())
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

    let hash = receipt.transaction_hash;
    let logs = receipt.inner.logs();
    let ip_data = logs
        .iter()
        .find_map(|log| IPRegistered::decode_log(log.as_ref(), true).ok())
        .ok_or_else(|| {
            let err = eyre::eyre!("IPRegistered event not found in logs");
            println!("Error: {}", err);
            err
        })?;

    let ipid = ip_data.ipId;

    Ok(IPData { ipid, hash })
}

pub async fn get_transaction_data(hash: FixedBytes<32>) -> Result<IPRegistered> {
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let receipt = provider
        .get_transaction_receipt(hash)
        .await?
        .ok_or_else(|| eyre::eyre!("Receipt not found"))?;
    let ip_data = receipt
        .inner
        .logs()
        .iter()
        .find_map(|log| IPRegistered::decode_log(log.as_ref(), true).ok())
        .ok_or_else(|| {
            let err = eyre::eyre!("IPRegistered event not found in logs");
            println!("Error: {}", err);
            err
        })?;
    Ok(ip_data.data)
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
            "0x1763C69c900A3Bad8BBb476EF0A13e8bb2c2b75B",
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
            "a1b2c3d4e5f6g7h8i9j0k1l2".to_string(),
        )
        .await;

        // Assert the result
        assert!(result.is_ok(), "register_ip should return Ok result");

        if let Ok(ip_data) = result {
            println!("ip_data.ipid: {:?}", ip_data.ipid);
            println!("ip_data.hash: {:?}", ip_data.hash);
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
