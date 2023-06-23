use std::sync::Arc;

use bonsai_common_log::init_logging;
use clap::Parser;
use ethereum_relay::{run_with_ethers_client, Config};
use ethers::{
    core::{
        k256::{ecdsa::SigningKey, SecretKey},
        types::Address,
    },
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Provider, Ws},
    signers::AwsSigner,
};
use rusoto_core::Region;
use rusoto_kms::KmsClient;

const DEFAULT_BONSAI_URL: &str = "http://localhost:8080";
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The Bonsai endpoint to connect to
    #[arg(short, long, default_value_t = DEFAULT_BONSAI_URL.to_string())]
    bonsai_url: String,

    /// The Bonsai API KEY
    #[arg(long, env = "API_KEY")]
    api_key: Option<String>,

    /// Ethereum Proxy address
    #[arg(short, long)]
    proxy_address: Address,

    /// Ethereum Node endpoint
    #[arg(long)]
    eth_node_url: String,

    // TODO(Cardosaum): Which is the best description for this value?
    #[arg(long, default_value_t = 5)]
    eth_chain_id: u64,

    // Wallet Key Identifier. Can be a private key as a hex string, or an AWS KMS key identifier
    #[arg(short, long)]
    wallet_key_identifier: String,

    #[arg(short, long)]
    use_kms: bool,

    // /// Bonsai contract address
    // #[arg(long)]
    // bonsai_contract_address: Address,
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

    init_logging(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        args.log_to_file.clone(),
        args.log_url.clone(),
    )
    .await
    .expect("error setting up logging");

    let config = Config {
        bonsai_url: args.bonsai_url,
        bonsai_api_key: args.api_key.expect("API_KEY not set"),
        proxy_address: args.proxy_address,
        log_status_interval: args.log_status_interval,
    };

    if args.use_kms {
        let kms_client = KmsClient::new(Region::default());
        let ethers_client = create_ethers_client_proxy_kms(
            &args.eth_node_url,
            &args.wallet_key_identifier,
            kms_client,
            args.eth_chain_id,
        )
        .await;
        run_with_ethers_client(config, ethers_client).await
    } else {
        let ethers_client = create_ethers_client_private_key(
            &args.eth_node_url,
            &args.wallet_key_identifier,
            args.eth_chain_id,
        )
        .await;

        run_with_ethers_client(config, ethers_client).await
    }
}

async fn create_ethers_client_private_key(
    eth_node_url: &str,
    wallet_key_identifier: &str,
    eth_chain_id: u64,
) -> Arc<SignerMiddleware<Provider<Ws>, LocalWallet>> {
    let web3_provider = Provider::<Ws>::connect(eth_node_url)
        .await
        .expect("unable to connect to websocket");
    let web3_wallet_sk_bytes = hex::decode(wallet_key_identifier)
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

async fn create_ethers_client_proxy_kms(
    eth_node_url: &str,
    wallet_key_identifier: &str,
    kms_client: KmsClient,
    eth_chain_id: u64,
) -> Arc<SignerMiddleware<Provider<Ws>, AwsSigner>> {
    let web3_provider = Provider::<Ws>::connect(eth_node_url)
        .await
        .expect("unable to connect to websocket");
    let aws_signer = AwsSigner::new(kms_client, wallet_key_identifier, eth_chain_id)
        .await
        .expect("error creating aws signer");

    Arc::new(SignerMiddleware::new(web3_provider, aws_signer))
}
