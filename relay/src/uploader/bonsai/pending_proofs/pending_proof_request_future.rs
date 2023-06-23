use std::pin::Pin;

use bonsai_sdk::{
    client::{Client, ClientError as BonsaiClientError},
    types::{ProofID, ReceiptInfo, ReceiptStatus},
};
use futures::{
    task::{Context, Poll},
    Future,
};
use pin_project::pin_project;
use snafu::prelude::*;

pub type ProofRequestID = ProofID;

#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to perform operation against Bonsai client: {}", source))]
    ClientAPI {
        source: BonsaiClientError,
        id: ProofRequestID,
    },
}

impl Error {
    pub fn get_proof_request_id(self) -> ProofRequestID {
        match self {
            Error::ClientAPI { source: _, id } => id,
        }
    }
}

type PollingBonsaiFuture = Pin<Box<dyn Future<Output = Result<ReceiptInfo, Error>> + Sync + Send>>;

enum PendingProofRequestState {
    // Inital state. The Proof Request has been submitted to Bonsai
    Pending,

    PollingBonsai(PollingBonsaiFuture),
}

#[pin_project]
pub struct PendingProofRequest {
    bonsai_client: Client,
    pending_proof_id: ProofID,
    state: PendingProofRequestState,
}

impl PendingProofRequest {
    pub fn new(bonsai_client: Client, pending_proof_id: ProofID) -> Self {
        Self {
            bonsai_client,
            pending_proof_id,
            state: PendingProofRequestState::Pending,
        }
    }
}

impl Future for PendingProofRequest {
    type Output = Result<ProofRequestID, Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<ProofRequestID, Error>> {
        let this = self.project();

        loop {
            match this.state {
                PendingProofRequestState::Pending => {
                    // Transition state to ask Bonsai for the proof request's
                    // status
                    let bonsai_get_receipt_fut =
                        get_receipt_info(this.bonsai_client.clone(), this.pending_proof_id.clone());

                    *this.state =
                        PendingProofRequestState::PollingBonsai(Box::pin(bonsai_get_receipt_fut))
                }

                PendingProofRequestState::PollingBonsai(bonsai_get_receipt_fut) => {
                    let response = futures::ready!(bonsai_get_receipt_fut.as_mut().poll(ctx));
                    match response {
                        Ok(receipt_response) => {
                            if let ReceiptStatus::Finished { .. } = receipt_response.status {
                                return Poll::Ready(Ok(*this.pending_proof_id));
                            }

                            // Not done yet, still pending. Transition back to pending
                            *this.state = PendingProofRequestState::Pending;
                            ctx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        Err(err) => {
                            return Poll::Ready(Err(err));
                        }
                    }
                }
            }
        }
    }
}

// Note: called self.bonsai_client.get_receipt in 'poll' causes the
// compiler to error out due to the lifetime of self being dropped but the
// future still needing it. Moving the function outside and not taking &self as
// a parameter fixes the issue
async fn get_receipt_info(client: Client, proof_id: ProofID) -> Result<ReceiptInfo, Error> {
    client
        .get_receipt_info(proof_id)
        .await
        .context(ClientAPISnafu { id: proof_id })
}
