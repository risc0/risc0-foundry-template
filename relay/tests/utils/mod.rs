use std::{path::Path, sync::Arc};

use bonsai_sdk::{
    routes::RECEIPT_ROUTE,
    types::{ProofID, ReceiptInfo, ReceiptResponse, ReceiptStatus},
};
use ethers::{
    abi::Tokenize,
    prelude::{
        k256::{ecdsa::SigningKey, SecretKey},
        *,
    },
    providers::{Http, Provider, Ws},
    signers::{LocalWallet, Signer},
    utils::AnvilInstance,
};
use reqwest::header;
use risc0_zkvm::recursion::{
    ReceiptMeta, SegmentRecursionReceipt, SessionRollupReceipt, SystemState,
};
use semver::Version;
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

pub mod test_callback_proof_request_processor;

const API_URI: &str = "http://localhost:8080";
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(5);

pub type Client<P> = Arc<SignerMiddleware<Provider<P>, LocalWallet>>;

pub fn get_wallet(anvil: Option<&AnvilInstance>) -> Wallet<SigningKey> {
    match std::env::var("TEST_PRIVATE_KEY") {
        Ok(test_private_key) => {
            // Derive wallet
            let wallet_sk_bytes = hex::decode(test_private_key.trim_start_matches("0x"))
                .expect("Could not decode input wallet secret key.");
            let wallet_secret_key = SecretKey::from_slice(&wallet_sk_bytes)
                .expect("Failed to derive SecretKey instance from input.");
            let wallet_signing_key = SigningKey::from(wallet_secret_key);
            LocalWallet::from(wallet_signing_key)
        }
        _ => {
            let anvil = anvil.expect("Anvil not instantiated.");
            LocalWallet::from(anvil.keys()[0].clone()).with_chain_id(anvil.chain_id())
        }
    }
}

pub fn get_http_provider(anvil: Option<&AnvilInstance>) -> Provider<Http> {
    let endpoint = match std::env::var("ETHEREUM_HOST") {
        Ok(ethereum_host) => format!("http://{ethereum_host}"),
        _ => anvil.expect("Anvil not instantiated.").endpoint(),
    };
    Provider::<Http>::try_from(endpoint)
        .unwrap()
        .interval(POLL_INTERVAL)
}

pub async fn get_ws_provider(anvil: Option<&AnvilInstance>) -> Provider<Ws> {
    let endpoint = match std::env::var("ETHEREUM_HOST") {
        Ok(ethereum_host) => format!("ws://{ethereum_host}"),
        _ => anvil.expect("Anvil not instantiated.").ws_endpoint(),
    };
    Provider::<Ws>::connect(&endpoint)
        .await
        .unwrap()
        .interval(POLL_INTERVAL)
}

pub async fn get_ethers_client<P: JsonRpcClient>(
    provider: Provider<P>,
    wallet: LocalWallet,
) -> Client<P> {
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    Arc::new(SignerMiddleware::new(
        provider,
        wallet.with_chain_id(chain_id),
    ))
}

pub fn get_anvil() -> Option<AnvilInstance> {
    match std::env::var("ETHEREUM_HOST") {
        Ok(_) => None,
        _ => Some(ethers::utils::Anvil::new().spawn()),
    }
}

pub async fn deploy_logger_contract<M: Middleware, S: Signer>(
    signer: Arc<SignerMiddleware<M, S>>,
) -> ethers::contract::Contract<SignerMiddleware<M, S>> {
    deploy_contract((), "Logger".to_string(), compile_test_contracts(), signer).await
}

pub async fn deploy_proxy_contract<M: Middleware, S: Signer>(
    signer: Arc<SignerMiddleware<M, S>>,
) -> ethers::contract::Contract<SignerMiddleware<M, S>> {
    deploy_contract((), "Proxy".to_string(), compile_test_contracts(), signer).await
}

pub async fn deploy_counter_contract<M: Middleware, S: Signer>(
    signer: Arc<SignerMiddleware<M, S>>,
) -> ethers::contract::Contract<SignerMiddleware<M, S>> {
    deploy_contract((), "Counter".to_string(), compile_test_contracts(), signer).await
}

pub async fn deploy_bonsai_contract<M: Middleware, S: Signer>(
    block_height: u128,
    signer: Arc<SignerMiddleware<M, S>>,
) -> ethers::contract::Contract<SignerMiddleware<M, S>> {
    deploy_contract(
        block_height,
        "BonsaiContract".to_string(),
        compile_test_contracts(),
        signer,
    )
    .await
}

pub async fn get_test_bonsai_server() -> (ProofID, MockServer) {
    // Mock API server
    let server = MockServer::start().await;

    let receipt_id = Uuid::new_v4();

    // Mock receipt response
    let receipt_response = ReceiptResponse {
        receipt_id,
        session_id: Default::default(),
    };

    let receipt_info_response = ReceiptInfo {
        version: Version::new(0, 0, 0),
        status: ReceiptStatus::Finished,
    };

    let receipt_data_response = SessionRollupReceipt::new(
        SegmentRecursionReceipt {
            seal: vec![],
            control_id: Default::default(),
            meta: ReceiptMeta {
                input: Default::default(),
                pre: SystemState {
                    pc: 0,
                    image_id: Default::default(),
                },
                post: SystemState {
                    pc: 0,
                    image_id: Default::default(),
                },
                sys_exit: 0,
                user_exit: 0,
                output: Default::default(),
            },
        },
        vec![],
    );
    Mock::given(method("POST"))
        .and(path(format!("{RECEIPT_ROUTE}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&receipt_response))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("{RECEIPT_ROUTE}/{receipt_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&receipt_info_response))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("{RECEIPT_ROUTE}/{receipt_id}/data")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(header::CONTENT_ENCODING, "zstd")
                .set_body_bytes(bincode::serialize(&receipt_data_response).unwrap()),
        )
        .mount(&server)
        .await;

    (receipt_id, server)
}

pub fn get_bonsai_url() -> String {
    let endpoint = match std::env::var("BONSAI_API_ENDPOINT") {
        Ok(endpoint) => endpoint,
        Err(_) => API_URI.to_string(),
    };

    let bonsai_api_endpoint = endpoint
        .is_empty()
        .then(|| API_URI.to_string())
        .unwrap_or(endpoint);

    bonsai_api_endpoint
}

pub fn get_api_key() -> String {
    match std::env::var("API_KEY") {
        Ok(api_key) => {
            dbg!(&api_key);
            api_key
        }
        _ => "test_key".to_string(),
    }
}

pub fn get_bonsai_client(api_key: String) -> bonsai_sdk::Client {
    let bonsai_api_endpoint = get_bonsai_url();
    bonsai_sdk::Client::new(bonsai_api_endpoint, api_key)
        .expect("Failed to create Bonsai client.")
        .clone()
}

async fn deploy_contract<T: Tokenize, M: Middleware, S: Signer>(
    constructor_args: T,
    contract_name: String,
    compiled: CompilerOutput,
    signer: Arc<SignerMiddleware<M, S>>,
) -> ethers::contract::Contract<SignerMiddleware<M, S>> {
    let (abi, bytecode, _runtime_bytecode) = compiled
        .find(contract_name.clone())
        .expect(&format!(
            "could not find contract {} in compiler output",
            contract_name.clone()
        ))
        .into_parts_or_default();

    let client = signer;
    let factory = ContractFactory::new(abi, bytecode, client.clone());
    let contract = factory
        .deploy(constructor_args)
        .expect(&format!(
            "constructing deploy transaction failed for {}",
            contract_name.clone()
        ))
        .send()
        .await
        .expect(&format!("deployed failed for {}", contract_name.clone()));

    contract
}

fn compile_test_contracts() -> CompilerOutput {
    let source = Path::new("tests/solidity/contracts");

    let compiled = Solc::default()
        .compile_source(source)
        .expect("Could not compile contracts");

    // println!("{:?}", compiled);

    compiled
}
