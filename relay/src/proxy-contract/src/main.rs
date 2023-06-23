use std::sync::Arc;

use bonsai_proxy_contract::ProxyContract;
use ethers::prelude::{
    k256::{ecdsa::SigningKey, SecretKey},
    *,
};

const GANACHE_ENDPOINT: &str = "GANACHE_ENDPOINT";
const TEST_PRIVATE_KEY: &str = "TEST_PRIVATE_KEY";
const BONSAI_CONTRACT_ADDRESS: &str = "BONSAI_CONTRACT_ADDRESS";

async fn deploy_contract() {
    let endpoint = match std::env::var(GANACHE_ENDPOINT) {
        Ok(endpoint) => endpoint,
        Err(_) => panic!("GANACHE_ENDPOINT environment variable is not set"),
    };

    let test_private_key = match std::env::var(TEST_PRIVATE_KEY) {
        Ok(private_key) => private_key,
        Err(_) => panic!("TEST_PRIVATE_KEY environment variable is not set"),
    };

    let bonsai_contract_address = match std::env::var(BONSAI_CONTRACT_ADDRESS) {
        Ok(address) => address,
        Err(_) => panic!("BONSAI_CONTRACT_ADDRESS environment variable is not set"),
    };

    // Connect to provider
    let provider =
        Provider::<Http>::try_from(endpoint).expect("Could not connect to HTTP RPC endpoint.");

    // Derive wallet
    let wallet_sk_bytes = hex::decode(test_private_key.trim_start_matches("0x"))
        .expect("Could not decode input wallet secret key.");
    let wallet_secret_key = SecretKey::from_slice(&wallet_sk_bytes)
        .expect("Failed to derive SecretKey instance from input.");
    let wallet_signing_key = SigningKey::from(wallet_secret_key);
    let wallet = LocalWallet::from(wallet_signing_key);

    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.with_chain_id(1337u64),
    ));

    let bonsai_contract_address_bytes: [u8; 20] = hex::decode(
        bonsai_contract_address
            .strip_prefix("0x")
            .expect("should still be valid string"),
    )
    .expect("valid hex encoded address")
    .try_into()
    .expect("valid 20 bytes");

    // Deploy Proxy
    let proxy_contract = ProxyContract::deploy(
        client.clone(),
        (Address::from(bonsai_contract_address_bytes),),
    )
    .expect("Failed to create Proxy deployment tx")
    .send()
    .await
    .expect("Failed to send Proxy deployment tx");

    println!("0x{:x}", proxy_contract.address());
}

#[tokio::main]
async fn main() {
    deploy_contract().await
}
