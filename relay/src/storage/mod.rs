use bonsai_proxy_contract::CallbackRequestFilter;
use ethers::types::H256;

pub mod in_memory;

use bonsai_sdk::types::ProofID;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to transition proof request: {}", source))]
    TransitionProofRequestError {
        source: Box<dyn snafu::Error + Sync + Send>,
    },
    #[snafu(display("Proof not found"))]
    ProofNotFound { id: ProofID },
}

#[derive(Debug, Clone)]
pub struct ProofRequstInformation {
    pub proof_request_id: ProofID,
    pub callback_proof_request_event: CallbackRequestFilter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProofRequestState {
    New,
    Pending,
    // Completed by Bonsai
    Completed,
    PreparingOnchain,
    CompletedOnchain(H256),
}

impl ProofRequestState {
    fn is_valid_state_transition(self, new_state: Self) -> bool {
        match (self, new_state) {
            (ProofRequestState::New, ProofRequestState::Pending)
            | (ProofRequestState::Pending, ProofRequestState::Completed)
            | (ProofRequestState::Completed, ProofRequestState::PreparingOnchain)
            // Allow a revert from PreparingOnchain to Completed. This is useful if the service
            // crashes while preparing a request for sending on chain.
            | (ProofRequestState::PreparingOnchain, ProofRequestState::Completed)
            | (ProofRequestState::PreparingOnchain, ProofRequestState::CompletedOnchain(_)) => true,
            _ => false,
        }
    }
}

#[async_trait::async_trait]
pub trait Storage {
    async fn add_new_bonsai_proof_request(
        &self,
        proof: ProofRequstInformation,
    ) -> Result<(), Error>;
    async fn fetch_new_bonsai_requests(
        &self,
        limit: Option<u64>,
    ) -> Result<Vec<ProofRequstInformation>, Error>;
    async fn fetch_completed_bonsai_requests(
        &self,
        limit: Option<u64>,
    ) -> Result<Vec<ProofRequstInformation>, Error>;
    async fn fetch_preparing_onchain_proof_requests(
        &self,
        limit: Option<u64>,
    ) -> Result<Vec<ProofRequstInformation>, Error>;
    async fn transition_proof_request(
        &self,
        proof_id: ProofID,
        new_state: ProofRequestState,
    ) -> Result<(), Error>;
    async fn get_proof_request_state(&self, proof_id: ProofID) -> Result<ProofRequestState, Error>;
}
