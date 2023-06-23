use std::sync::Arc;

use bonsai_proxy_contract::CallbackRequestFilter;
use bonsai_sdk::{
    client::{Client, ClientError as BonsaiClientError},
    types::ProofID,
};
use snafu::prelude::*;
use tokio::sync::Notify;
use tracing::info;

use crate::{
    downloader::common::event_processor::{EventProcessor, EventProcessorError},
    storage::{Error as StorageError, ProofRequstInformation, Storage},
};

#[derive(Debug, Snafu)]
pub enum ProxyCallbackProofRequestProcessorError {
    #[snafu(display("Error creating proof request in Bonsai: source: {}", source))]
    ParseReceiptIDError { source: uuid::Error },
    #[snafu(display("Error creating proof request in Bonsai: source: {}", source))]
    PostSessionRequestBonsaiError { source: BonsaiClientError },
    #[snafu(display("Error storing pending proof request: source: {}", source))]
    StorePendingProofError { source: StorageError },
}

impl From<ProxyCallbackProofRequestProcessorError> for EventProcessorError {
    fn from(err: ProxyCallbackProofRequestProcessorError) -> EventProcessorError {
        match err {
            ProxyCallbackProofRequestProcessorError::PostSessionRequestBonsaiError { .. } => {
                EventProcessorError::ProcessEventError {
                    source: Box::new(err),
                }
            }
            ProxyCallbackProofRequestProcessorError::StorePendingProofError { .. } => {
                EventProcessorError::ProcessEventError {
                    source: Box::new(err),
                }
            }
            ProxyCallbackProofRequestProcessorError::ParseReceiptIDError { .. } => {
                EventProcessorError::ProcessEventError {
                    source: Box::new(err),
                }
            }
        }
    }
}

pub struct ProxyCallbackProofRequestProcessor<S: Storage> {
    bonsai_client: Client,
    storage: S,
    notifier: Option<Arc<Notify>>,
}

impl<S: Storage> ProxyCallbackProofRequestProcessor<S> {
    pub fn new(bonsai_client: Client, storage: S, notifier: Option<Arc<Notify>>) -> Self {
        Self {
            bonsai_client,
            storage,
            notifier,
        }
    }
}

#[async_trait::async_trait]
impl<S: Storage + Sync + Send> EventProcessor for ProxyCallbackProofRequestProcessor<S> {
    type Event = CallbackRequestFilter;

    async fn process_event(&self, event: CallbackRequestFilter) -> Result<(), EventProcessorError> {
        let receipt_id = if event.image_id.eq(&[0u8; 32]) {
            let receipt_id = ProofID::from_slice(&event.input).context(ParseReceiptIDSnafu)?;
            self.bonsai_client
                .get_receipt_info(receipt_id)
                .await
                .context(PostSessionRequestBonsaiSnafu)?;
            info!(?receipt_id, "requested existing receipt from bonsai");
            receipt_id
        } else {
            let bonsai_proof = self
                .bonsai_client
                .request_receipt(event.image_id, event.input.to_vec())
                .await
                .context(PostSessionRequestBonsaiSnafu)?;
            info!(?bonsai_proof, "requested a new receipt from bonsai");
            bonsai_proof.receipt_id
        };

        // Store the request in storage
        self.storage
            .add_new_bonsai_proof_request(ProofRequstInformation {
                proof_request_id: receipt_id,
                callback_proof_request_event: event,
            })
            .await
            .context(StorePendingProofSnafu)?;

        match self.notifier.clone() {
            Some(notifier) => notifier.notify_one(),
            None => (),
        }

        info!(?receipt_id, "processed callback event");
        Ok(())
    }
}
