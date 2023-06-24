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

use bonsai_starter_methods::{resolve_guest_entry, resolve_image_output, GUEST_LIST};
use ethers::{core::types::Address, prelude::*};
use tracing::info;

abigen!(ProxyContract, "artifacts/proxy.sol/Proxy.json");

pub struct Config {
    pub proxy_address: Address,
    pub log_status_interval: u64,
}

pub async fn run_with_ethers_client<M: Middleware + 'static>(config: Config, ethers_client: Arc<M>)
where
    <M as ethers::providers::Middleware>::Provider: PubsubClient,
    <<M as ethers::providers::Middleware>::Provider as ethers::providers::PubsubClient>::NotificationStream: Sync,
{
    let event_name = "CallbackRequest(address,bytes32,bytes,address,bytes4,uint64)";
    let filter = ethers::types::Filter::new()
        .address(config.proxy_address)
        .event(event_name);
    let mut proxy_stream = ethers_client
        .subscribe_logs(&filter)
        .await
        .unwrap()
        .map(|log| {
            ethers::contract::parse_log::<CallbackRequestFilter>(log)
                .expect("must be a callback proof request log")
        });

    let proxy: ProxyContract<M> = ProxyContract::new(config.proxy_address, ethers_client.clone());
    while let Some(event) = proxy_stream.next().await {
        // Search list for requested binary name
        let image_id = hex::encode(event.image_id);
        let guest_entry =
            resolve_guest_entry(GUEST_LIST, &image_id).expect("Failed to resolve guest entry");

        // Execute or return image id
        let input = hex::encode(event.input);
        let output_bytes =
            resolve_image_output(&input, guest_entry).expect("Failed to compute journal output");

        // Broadcast callback transaction
        let proof_batch = vec![Callback {
            callback_contract: event.callback_contract,
            journal_inclusion_proof: vec![],
            payload: output_bytes.into(),
            gas_limit: event.gas_limit,
        }];

        info!("sending batch");
        let contract_call = proxy.invoke_callbacks(proof_batch);
        let pending_tx = contract_call
            .send()
            .await
            .expect("failed to send callback transaction");

        pending_tx
            .await
            .expect("Failed to confirm callback transaction");
    }
}
