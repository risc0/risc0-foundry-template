// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use bonsai_ethereum_relay::{run_with_ethers_client, Config};
use clap::Parser;
use ethers::{
    core::{
        k256::{ecdsa::SigningKey, SecretKey},
        types::Address,
    },
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Provider, Ws},
};

const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ethereum Proxy address
    #[arg(short, long)]
    relay_contract_address: Address,

    /// Ethereum Node endpoint
    #[arg(long)]
    eth_node_url: String,

    /// Ethereum Chain ID
    #[arg(long, default_value_t = 5)]
    eth_chain_id: u64,

    /// Wallet private key.
    #[arg(short, long)]
    private_key: String,

    /// Log status interval [in seconds]
    #[arg(long, default_value_t = 600)]
    log_status_interval: u64,

    /// Log level
    #[arg(long, default_value_t = DEFAULT_LOG_LEVEL.to_string())]
    log_level: String,

    /// Log to file
    #[arg(short = 'F', long, default_value_t = false)]
    log_to_file: bool,

    /// Push logs to external collector (url Loki)
    #[arg(long)]
    log_url: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = Config {
        proxy_address: args.relay_contract_address,
        log_status_interval: args.log_status_interval,
    };

    let ethers_client = create_ethers_client_private_key(
        &args.eth_node_url,
        &args.private_key,
        args.eth_chain_id,
    )
    .await;

    run_with_ethers_client(config, ethers_client).await
}

async fn create_ethers_client_private_key(
    eth_node_url: &str,
    private_key: &str,
    eth_chain_id: u64,
) -> Arc<SignerMiddleware<Provider<Ws>, LocalWallet>> {
    let web3_provider = Provider::<Ws>::connect(eth_node_url)
        .await
        .expect("unable to connect to websocket");
    let web3_wallet_sk_bytes = hex::decode(private_key)
        .expect("wallet_key_identifier should be valid hex string");
    let web3_wallet_secret_key =
        SecretKey::from_slice(&web3_wallet_sk_bytes).expect("invalid private key");
    let web3_wallet_signing_key = SigningKey::from(web3_wallet_secret_key);
    let web3_wallet = LocalWallet::from(web3_wallet_signing_key);
    Arc::new(SignerMiddleware::new(
        web3_provider,
        web3_wallet.with_chain_id(eth_chain_id),
    ))
}
