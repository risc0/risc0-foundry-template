use std::sync::Arc;

use bonsai_proxy_contract::CallbackRequestFilter;
use ethers::{
    providers::{Middleware, PubsubClient},
    types::Address,
};
use futures::StreamExt;
use snafu::prelude::*;

use crate::downloader::common::event_processor::{EventProcessor, EventProcessorError};

#[derive(Debug, Snafu)]
pub enum ProxyRunnerError {
    #[snafu(display("Error getting address: source: {}", source))]
    LoopTerminatedError { source: EventProcessorError },
}

pub struct ProxyCallbackProofRequestStream<
    M: Middleware,
    EP: EventProcessor<Event = CallbackRequestFilter> + Sync + Send,
> {
    ethers_client: Arc<M>,
    proxy_contract_address: Address,
    event_processor: EP,
}

impl<M: Middleware, EP: EventProcessor<Event = CallbackRequestFilter> + Sync + Send>
    ProxyCallbackProofRequestStream<M, EP>
where
    <M as Middleware>::Provider: PubsubClient,
{
    pub fn new(
        ethers_client: Arc<M>,
        proxy_contract_address: Address,
        event_processor: EP,
    ) -> ProxyCallbackProofRequestStream<M, EP> {
        Self {
            ethers_client,
            proxy_contract_address,
            event_processor,
        }
    }

    pub async fn run(self) -> Result<(), ProxyRunnerError> {
        let event_name = "CallbackRequest(address,bytes32,bytes,address,bytes4,uint64)";
        let filter = ethers::types::Filter::new()
            .address(self.proxy_contract_address)
            .event(event_name);
        let mut proxy_stream = self
            .ethers_client
            .subscribe_logs(&filter)
            .await
            .unwrap()
            .map(|log| {
                ethers::contract::parse_log::<CallbackRequestFilter>(log)
                    .expect("must be a callback proof request log")
            });

        while let Some(event) = proxy_stream.next().await {
            self.event_processor
                .process_event(event)
                .await
                .context(LoopTerminatedSnafu)?
        }

        Ok(())
    }
}
