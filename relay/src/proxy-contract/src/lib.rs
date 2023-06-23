// Copyright 2023 Risc0, Inc.

//! Specifies the BPN Proxy Ethereum contract and provides
//! structures and procedures for programmatic deployment of and interaction
//! with a `Proxy` contract instance on Ethereum.

use ethers::prelude::*;
use tracing::trace;

abigen!(ProxyContract, "artifacts/proxy.sol/Proxy.json");

#[derive(Clone, Debug)]
pub struct EthereumCallback {
    pub journal_inclusion_proof: Vec<H256>,
    pub journal: Vec<u8>,
    pub gas_limit: u64,
    pub callback_request: CallbackRequestFilter,
}

impl From<EthereumCallback> for Callback {
    fn from(value: EthereumCallback) -> Self {
        let payload = [
            value.callback_request.function_selector.as_slice(),
            value.journal.as_slice(),
            value.callback_request.image_id.as_slice(),
        ]
        .concat();
        Self {
            callback_contract: value.callback_request.callback_contract,
            journal_inclusion_proof: value
                .journal_inclusion_proof
                .into_iter()
                .map(|p| p.0)
                .collect(),
            payload: payload.into(),
            gas_limit: value.gas_limit,
        }
    }
}

/// Fetches the `CallbackRequest` events of a `Proxy` instance since a
/// given block number and returns the equivalent `CallbackRequestFilter`
/// instances.
pub async fn read_request_logs<M: Middleware>(
    contract: &ProxyContract<M>,
    from_block: u64,
    to_block: u64,
) -> Result<Vec<CallbackRequestFilter>, ContractError<M>> {
    trace!(
        from_block = ?from_block,
        to_block = ?to_block,
        "Reading request logs from proxy contract.");
    let filter = contract
        .callback_request_filter()
        .from_block(from_block)
        .to_block(to_block);
    filter.query().await
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use ethers::{
        abi::{ethereum_types::Secret, AbiEncode},
        prelude::{
            k256::{ecdsa::SigningKey, SecretKey},
            *,
        },
        utils::{hex, Ganache, GanacheInstance},
    };
    use risc0_zkvm::SessionReceipt;

    use crate::{CallbackRequestFilter, EthereumCallback, ProxyContract};

    abigen!(BPNDummy, "artifacts/bpn_dummy.sol/BPNDummy.json");
    abigen!(
        CallbackDummy,
        "artifacts/callback_dummy.sol/CallbackDummy.json"
    );

    fn get_client() -> (
        Option<GanacheInstance>,
        Arc<SignerMiddleware<Provider<ethers::providers::Http>, Wallet<SigningKey>>>,
    ) {
        match (
            std::env::var("GANACHE_ENDPOINT"),
            std::env::var("TEST_PRIVATE_KEY"),
        ) {
            (Ok(endpoint), Ok(test_private_key)) => {
                // Connect to provider
                let provider = Provider::<Http>::try_from(endpoint)
                    .expect("Could not connect to HTTP RPC endpoint.");

                // Derive wallet
                let wallet_sk_bytes = hex::decode(test_private_key.trim_start_matches("0x"))
                    .expect("Could not decode input wallet secret key.");
                let wallet_secret_key = SecretKey::from_slice(&wallet_sk_bytes)
                    .expect("Failed to derive SecretKey instance from input.");
                let wallet_signing_key = SigningKey::from(wallet_secret_key);
                let wallet = LocalWallet::from(wallet_signing_key);

                (
                    None,
                    Arc::new(SignerMiddleware::new(
                        provider,
                        wallet.with_chain_id(1337u64),
                    )),
                )
            }
            _ => {
                // Launch ganache instance
                let ganache = Ganache::new().spawn();

                // Instantiate wallet
                let wallet: LocalWallet = ganache.keys()[0].clone().into();

                // Connect to network
                let provider = Provider::<Http>::try_from(ganache.endpoint())
                    .unwrap()
                    .interval(Duration::from_millis(10u64));

                // Instantiate client as wallet on network
                (
                    Some(ganache),
                    Arc::new(SignerMiddleware::new(
                        provider,
                        wallet.with_chain_id(1337u64),
                    )),
                )
            }
        }
    }

    #[tokio::test]
    pub async fn test_happy_path() {
        let (_ganache, client) = get_client();
        let wallet_address = client.address();
        // Deploy dummy BPN contract
        let dummy_bpn = BPNDummy::deploy(client.clone(), ())
            .unwrap()
            .send()
            .await
            .unwrap();

        // Deploy Proxy
        let proxy_contract = ProxyContract::deploy(client.clone(), (dummy_bpn.address(),))
            .expect("Failed to create Proxy deployment tx")
            .send()
            .await
            .expect("Failed to send Proxy deployment tx");
        assert_eq!(
            client
                .get_balance(proxy_contract.address(), None)
                .await
                .unwrap(),
            U256::zero()
        );

        let image_id: [u8; 32] = [0xf2; 32];

        // Deploy dummy Callback contract
        let dummy_callback =
            CallbackDummy::deploy(client.clone(), (image_id, proxy_contract.address()))
                .unwrap()
                .send()
                .await
                .unwrap();

        let call_me_selector = CallMeCall::selector();
        // Create some dummy callback requests
        let callback_requests = vec![
            CallbackRequestFilter {
                account: wallet_address.into(),
                image_id: image_id.clone(),
                input: Vec::new().into(),
                callback_contract: dummy_callback.address(),
                function_selector: call_me_selector.clone(),
                gas_limit: 50000,
            },
            CallbackRequestFilter {
                account: wallet_address.into(),
                image_id: image_id.clone(),
                input: Vec::new().into(),
                callback_contract: dummy_callback.address(),
                function_selector: call_me_selector.clone(),
                gas_limit: 50000,
            },
        ];

        // Send both proof requests to the proxy
        for request in callback_requests.clone() {
            proxy_contract
                .request_callback(
                    request.image_id,
                    request.input,
                    request.callback_contract,
                    request.function_selector,
                    request.gas_limit,
                )
                .send()
                .await
                .expect("Failed to submit proof request");
        }
        // Ensure callback contract is not affected yet
        assert_eq!(
            dummy_callback
                .counter()
                .call()
                .await
                .expect("Failed to get counter value"),
            U256::zero()
        );

        let call_me_call = CallMeCall {
            number: U256::from(1),
            guess: true,
            callback_image_id: Secret::zero().0,
        };

        let journal = call_me_call.encode()[4..4 + 32 + 32].to_vec();

        // Create dummy responses
        let ethereum_callbacks = vec![
            EthereumCallback {
                journal_inclusion_proof: vec![],
                journal: journal.clone(),
                gas_limit: 50000,
                callback_request: callback_requests[0].clone(),
            },
            EthereumCallback {
                journal_inclusion_proof: vec![],
                journal,
                gas_limit: 50000,
                callback_request: callback_requests[1].clone(),
            },
        ];
        dbg!(&ethereum_callbacks);

        // Submit responses
        let callbacks = ethereum_callbacks.into_iter().map(|p| p.into()).collect();
        let invocation_transaction = proxy_contract
            .invoke_callbacks(callbacks)
            .send()
            .await
            .expect("Failed to submit proof bundle")
            .await
            .expect("Failed to submit proof bundle")
            .expect("Failed to retrieve transaction receipt");
        dbg!(invocation_transaction);
        // Ensure callback contract counter has been increased twice
        assert_eq!(
            dummy_callback
                .counter()
                .call()
                .await
                .expect("Failed to get counter value"),
            U256::from(2)
        );
    }
}
