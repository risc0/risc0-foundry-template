use std::sync::Arc;

use bonsai_sdk::client::Client;
use futures::{stream::FuturesUnordered, StreamExt};
use snafu::prelude::*;
use tokio::{
    sync::Notify,
    task::{JoinError, JoinHandle},
};
use tracing::info;

use crate::{
    storage::{Error as StorageError, ProofRequestState, Storage},
    uploader::bonsai::pending_proofs::pending_proof_request_future::{
        Error as PendingProofError, PendingProofRequest, ProofRequestID,
    },
};

#[derive(Debug, Snafu)]
pub enum BonsaiPendingProofManagerError {
    #[snafu(display("Error getting new pending requests: source: {}", source))]
    NewPendingProofRequestsError { source: StorageError },
    #[snafu(display("Error marking proof request pending: source: {}", source))]
    MarkingProofRequestPendingError { source: StorageError },
    #[snafu(display("Error marking proof request completed: source: {}", source))]
    MarkingProofRequestCompletedError { source: StorageError },
    #[snafu(display("Error marking proof request new: source: {}", source))]
    MarkingProofRequestNewError { source: StorageError },
    #[snafu(display("Error marking proof request new: source: {}", source))]
    JoinHandleFailedError { source: JoinError },
    #[snafu(display("Error marking proof request new: source: {}", source))]
    PendingProofError { source: PendingProofError },
}

pub struct BonsaiPendingProofManager<S: Storage> {
    client: Client,
    storage: S,
    new_pending_proof_request_notifier: Arc<Notify>,
    complete_proof_manager_notifier: Arc<Notify>,
    futures_set: FuturesUnordered<JoinHandle<Result<ProofRequestID, PendingProofError>>>,
}

impl<S: Storage> BonsaiPendingProofManager<S> {
    pub fn new(
        client: Client,
        storage: S,
        new_pending_proof_request_notifier: Arc<Notify>,
        complete_proof_manager_notifier: Arc<Notify>,
    ) -> Self {
        Self {
            client,
            storage,
            new_pending_proof_request_notifier,
            complete_proof_manager_notifier,
            futures_set: FuturesUnordered::new(),
        }
    }

    async fn process_new_pending_proof_requests(
        &mut self,
    ) -> Result<(), BonsaiPendingProofManagerError> {
        let pending_proof_requests = self
            .storage
            .fetch_new_bonsai_requests(None)
            .await
            .context(NewPendingProofRequestsSnafu)?;

        for request in pending_proof_requests.into_iter() {
            let pending_proof_request =
                PendingProofRequest::new(self.client.clone(), request.proof_request_id);
            let pending_proof_request_handler = tokio::spawn(pending_proof_request);
            self.futures_set.push(pending_proof_request_handler);

            self.storage
                .transition_proof_request(request.proof_request_id, ProofRequestState::Pending)
                .await
                .context(MarkingProofRequestPendingSnafu)?;

            info!(?request.proof_request_id, "processing new pending proof");
        }

        Ok(())
    }

    pub async fn handle_pending_proof_result(
        &self,
        pending_proof_result: Result<ProofRequestID, PendingProofError>,
    ) -> Result<(), BonsaiPendingProofManagerError> {
        let completed_proof_id = pending_proof_result.context(PendingProofSnafu)?;

        self.storage
            .transition_proof_request(completed_proof_id, ProofRequestState::Completed)
            .await
            .context(MarkingProofRequestCompletedSnafu)?;

        self.complete_proof_manager_notifier.notify_one();

        info!(?completed_proof_id, "pending proof done");

        Ok(())
    }

    pub async fn step(&mut self) -> Result<(), BonsaiPendingProofManagerError> {
        tokio::select! {
            Some(pending_proof_handle) = self.futures_set.next() => {
                    let pending_proof_result = pending_proof_handle.context(JoinHandleFailedSnafu)?;
                    self.handle_pending_proof_result(pending_proof_result).await?
            }
            _ = self.new_pending_proof_request_notifier.notified() => {
                self.process_new_pending_proof_requests().await?
            }
        }

        Ok(())
    }

    pub async fn run(mut self) -> Result<(), BonsaiPendingProofManagerError> {
        self.process_new_pending_proof_requests().await?;

        loop {
            match self.step().await {
                Err(BonsaiPendingProofManagerError::JoinHandleFailedError { source }) => {
                    // if a task panics, just fail
                    return Err(BonsaiPendingProofManagerError::JoinHandleFailedError { source });
                }
                Err(BonsaiPendingProofManagerError::PendingProofError { source }) => {
                    // An error occurred processing the pending proof.
                    println!(
                        "error occurred managing pending proof requests: {:?}",
                        source
                    );
                    // Store the proof as new so that it can be retried.
                    //
                    // What do we do if this call to storage fails?
                    self.storage
                        .transition_proof_request(
                            source.get_proof_request_id(),
                            ProofRequestState::New,
                        )
                        .await
                        .context(MarkingProofRequestNewSnafu)?
                }

                _ => (),
            }
        }
    }
}
