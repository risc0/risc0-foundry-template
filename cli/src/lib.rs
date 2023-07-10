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

use std::path::Path;
use std::sync::Arc;

use ethers::abi::Tokenize;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::k256::SecretKey;
use ethers::prelude::*;
use ethers::providers::Provider;
use ethers::signers::{LocalWallet, Signer};

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(5);

pub type Client<P> = Arc<SignerMiddleware<Provider<P>, LocalWallet>>;

pub fn get_wallet(private_key: &str, chain_id: u64) -> Wallet<SigningKey> {
    // Derive wallet
    let wallet_sk_bytes = hex::decode(private_key.trim_start_matches("0x"))
        .expect("Could not decode input wallet secret key.");
    let wallet_secret_key = SecretKey::from_slice(&wallet_sk_bytes)
        .expect("Failed to derive SecretKey instance from input.");
    let wallet_signing_key = SigningKey::from(wallet_secret_key);
    LocalWallet::from(wallet_signing_key).with_chain_id(chain_id)

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

pub fn get_http_provider(endpoint: &str) -> Provider<Http> {
    Provider::<Http>::try_from(endpoint)
        .unwrap()
        .interval(POLL_INTERVAL)
}

pub async fn get_ws_provider(endpoint: &str) -> Provider<Ws> {
    Provider::<Ws>::connect(endpoint)
        .await
        .unwrap()
        .interval(POLL_INTERVAL)
}

pub async fn deploy_starter_contract<M: Middleware, S: Signer>(
    signer: Arc<SignerMiddleware<M, S>>,
) -> ethers::contract::Contract<SignerMiddleware<M, S>> {
    deploy_contract((), "BonsaiStarter".to_string(), compile_test_contracts(), signer).await
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
