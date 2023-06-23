use std::sync::Arc;

use bonsai_proxy_contract::{Callback, ProxyContract};
use bonsai_sdk::client::Client;
use ethers::prelude::*;
use futures::{stream::FuturesUnordered, StreamExt};
use snafu::prelude::*;
use tokio::{sync::Notify, task::JoinHandle};
use tracing::info;

use crate::{
    storage::{ProofRequestState, Storage},
    uploader::bonsai::completed_proofs::{
        complete_proof::{get_complete_proof, CompleteProof},
        error::*,
    },
};

pub struct BonsaiCompleteProofManager<S: Storage, M: Middleware> {
    client: Client,
    storage: S,
    new_complete_proofs_notifier: Arc<Notify>,
    ready_to_send_batch: Vec<CompleteProof>,
    max_batch_size: usize,
    proxy_contract_address: Address,
    ethers_client: Arc<M>,
    send_batch_notifier: Arc<Notify>,
    send_batch_interval: tokio::time::Interval,
    futures_set: FuturesUnordered<JoinHandle<Result<CompleteProof, CompleteProofError>>>,
}

impl<S: Storage, M: Middleware + 'static> BonsaiCompleteProofManager<S, M> {
    pub fn new(
        client: Client,
        storage: S,
        new_complete_proofs_notifier: Arc<Notify>,
        send_batch_notifier: Arc<Notify>,
        max_batch_size: usize,
        proxy_contract_address: Address,
        ethers_client: Arc<M>,
        send_batch_interval: tokio::time::Interval,
    ) -> Self {
        Self {
            client,
            storage,
            new_complete_proofs_notifier,
            ready_to_send_batch: Vec::new(),
            max_batch_size,
            proxy_contract_address,
            ethers_client,
            send_batch_notifier,
            send_batch_interval,
            futures_set: FuturesUnordered::new(),
        }
    }

    async fn send_batch(&mut self) -> Result<(), BonsaiCompleteProofManagerError<M>> {
        if self.ready_to_send_batch.len() == 0 {
            return Ok(());
        }

        let proxy: ProxyContract<M> =
            ProxyContract::new(self.proxy_contract_address, self.ethers_client.clone());
        let proof_batch: Vec<Callback> = self
            .ready_to_send_batch
            .clone()
            .into_iter()
            .map(|complete_proof| complete_proof.ethereum_callback.into())
            .collect();

        info!("sending batch");
        let contract_call = proxy.invoke_callbacks(proof_batch);
        let pending_tx = contract_call.send().await.context(SubmitProofsSnafu)?;
        let tx_hash = pending_tx.tx_hash();

        pending_tx
            .await
            .context(ConfirmationSnafu { tx_hash: tx_hash })?;

        for completed_proof in self.ready_to_send_batch.clone().into_iter() {
            self.storage
                .transition_proof_request(
                    completed_proof.bonsai_proof_id,
                    ProofRequestState::CompletedOnchain(tx_hash.clone()),
                )
                .await
                .context(MarkingProofRequestCompletedOnchainSnafu {
                    id: completed_proof.bonsai_proof_id,
                })?;
        }

        self.ready_to_send_batch.clear();

        Ok(())
    }

    async fn process_new_complete_proof_requests(
        &mut self,
    ) -> Result<(), BonsaiCompleteProofManagerError<M>> {
        let completed_proof_requests = self
            .storage
            .fetch_completed_bonsai_requests(None)
            .await
            .context(FetchCompleteProofRequestsSnafu)?;

        for request in completed_proof_requests.into_iter() {
            let completed_proof_request_handler = tokio::spawn(get_complete_proof(
                self.client.clone(),
                request.proof_request_id,
                request.callback_proof_request_event,
            ));
            self.futures_set.push(completed_proof_request_handler);

            self.storage
                .transition_proof_request(
                    request.proof_request_id,
                    ProofRequestState::PreparingOnchain,
                )
                .await
                .context(MarkingProofRequestPreparingOnchainSnafu {
                    id: request.proof_request_id,
                })?;

            info!(?request.proof_request_id, "processing compeleted proof");
        }

        Ok(())
    }

    pub async fn handle_complete_proof_result(
        &mut self,
        completed_proof_result: Result<CompleteProof, CompleteProofError>,
    ) -> Result<(), BonsaiCompleteProofManagerError<M>> {
        let completed_proof = completed_proof_result.context(ParseCompleteProofRequestSnafu)?;

        self.ready_to_send_batch.push(completed_proof.clone());
        if self.ready_to_send_batch.len() >= self.max_batch_size {
            self.send_batch_notifier.notify_one();
        }

        info!(?completed_proof, "proof added to batch");
        Ok(())
    }

    async fn reset_inflight_proof_requests(
        &mut self,
    ) -> Result<(), BonsaiCompleteProofManagerError<M>> {
        let inflight_requests = self
            .storage
            .fetch_preparing_onchain_proof_requests(None)
            .await
            .context(FetchPreparingOnchainRequestsSnafu)?;

        for request in inflight_requests.into_iter() {
            self.storage
                .transition_proof_request(request.proof_request_id, ProofRequestState::Completed)
                .await
                .context(RevertingProofRequestToCompletedSnafu {
                    id: request.proof_request_id,
                })?;
        }

        Ok(())
    }

    pub async fn step(&mut self) -> Result<(), BonsaiCompleteProofManagerError<M>> {
        tokio::select! {
            Some(completed_proof_handle) = self.futures_set.next() => {
                let completed_proof_result = completed_proof_handle.context(JoinHandleFailedSnafu)?;
                self.handle_complete_proof_result(completed_proof_result).await?
            }
            _ = self.new_complete_proofs_notifier.notified() => {
                self.process_new_complete_proof_requests().await?
            }

            _ = self.send_batch_interval.tick() => {
                self.send_batch_notifier.notify_one();
            }

            _ = self.send_batch_notifier.notified() => {
                self.send_batch().await?
            }
        }

        Ok(())
    }

    pub async fn run(mut self) -> Result<(), BonsaiCompleteProofManagerError<M>> {
        self.reset_inflight_proof_requests().await?;
        self.process_new_complete_proof_requests().await?;

        loop {
            match self.step().await {
                Err(BonsaiCompleteProofManagerError::JoinHandleFailedError { source }) => {
                    // if a task panics, just fail
                    return Err(BonsaiCompleteProofManagerError::JoinHandleFailedError { source });
                }
                Err(err) => {
                    // An error occurred processing the completed proof.
                    println!("error occurred managing pending proof requests: {:?}", err);
                    if let Some(proof_request_id) = err.get_proof_request_id() {
                        // Store the proof as new so that it can be retried.
                        //
                        // What do we do if this call to storage fails?
                        match self
                            .storage
                            .transition_proof_request(proof_request_id, ProofRequestState::New)
                            .await
                        {
                            Err(err) => println!("failed to retry failed proof request: {:?}", err),
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
