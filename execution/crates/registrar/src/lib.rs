use alloy::{primitives::Address, providers::ProviderBuilder, rpc::types::TransactionReceipt, sol};
use eyre::Result;
use IPARegistrar::IPMetadata;

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
) -> Result<TransactionReceipt> {
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let contract = IPARegistrar::new(address, provider);

    let imetadata = IPMetadata {
        name,
        ipMetadataURI: ip_metatdata_uri,
        ipMetadata: ip_metadata,
        nftMetadataURI: nft_metadata_uri,
        nftMetadata: nft_metadata,
    };

    let result = contract
        .register(address, imetadata)
        .send()
        .await?
        .get_receipt()
        .await?;
    Ok(result)
}
