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
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;

    let signer: PrivateKeySigner = "<PRIVATE_KEY>".parse().expect("should parse private key");
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

    let hash = contract
        .register(address, imetadata)
        .from(signer.address())
        .send()
        .await?
        .get_receipt()
        .await?
        .transaction_hash;
    let input = hex::decode(
        provider
            .get_transaction_by_hash(hash)
            .await?
            .unwrap()
            .inner
            .input(),
    )
    .unwrap();
    Ok(IPData {
        ipid: IPARegistrar::registerCall::abi_decode_returns(&input, true)
            .unwrap()
            ._0
            .clone(),
        hash,
    })
}
