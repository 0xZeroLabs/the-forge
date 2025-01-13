use std::{path::PathBuf, str::FromStr};

use bundlr_sdk::{bundlr::BundlrBuilder, currency::ethereum::EthereumBuilder, error::BundlrError};
use reqwest::Url;

pub async fn upload_file() -> Result<(), BundlrError> {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let currency = EthereumBuilder::new()
        .wallet("kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb")
        .build()
        .expect("Could not create Ethereum instance");
    let mut bundlr = BundlrBuilder::new()
        .url(url)
        .currency(currency)
        .fetch_pub_info()
        .await?
        .build()?;

    let file = PathBuf::from_str("res/test_image.jpg").unwrap();
    let res = bundlr.upload_file(file).await;
    match res {
        Ok(()) => println!("[ok]"),
        Err(err) => println!("[err] {}", err),
    }
    Ok(())
}
