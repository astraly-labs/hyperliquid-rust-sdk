use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;
use std::env;

/// I didn't test this code yet, might need some adjustments
#[tokio::main]
async fn main() {
    env_logger::init();

    let private_key =
        env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable must be set");

    let wallet: LocalWallet = private_key.parse().unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // Reserve 1000 additional API request weight (costs 0.0005 USDC per request)
    let weight = 1000;
    let response = exchange_client
        .reserve_request_weight(weight, None)
        .await
        .unwrap();

    info!("Reserve request weight response: {response:?}");
}
