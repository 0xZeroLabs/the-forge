use alloy::{primitives::address, providers::ProviderBuilder, sol};
use eyre::Result;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IPARegistrar,
    "fixture/abi/IPARegistrar.json"
);

pub async fn register_ip() -> eyre::Result<()> {
    let rpc_url = "https://rpc.odyssey.storyrpc.io".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let contract = IPARegistrar::new(address!("registrar_address"), provider);

    contract.register().send().await?;
    Ok(())
}
