use bonsai_sdk::{client::ClientError as BonsaiClientError, types::ProofID};
use displaydoc::Display;
use ethers::{
    prelude::{Middleware, ProviderError},
    types::H256,
};
use snafu::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use crate::storage::Error as StorageError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BonsaiCompleteProofManagerError<M: Middleware + 'static> {
    #[snafu(display("Error getting new completed requests: source: {}", source))]
    FetchCompleteProofRequestsError { source: StorageError },
    #[snafu(display("Error getting preparing onchain requests: source: {}", source))]
    FetchPreparingOnchainRequestsError { source: StorageError },
    #[snafu(display("Error marking proof request sending on chain: source: {}", source))]
    MarkingProofRequestPreparingOnchainError { source: StorageError, id: ProofID },
    #[snafu(display("Error marking proof request completed on chain: source: {}", source))]
    MarkingProofRequestCompletedOnchainError { source: StorageError, id: ProofID },
    #[snafu(display("Error reverting proof request back to completed: source: {}", source))]
    RevertingProofRequestToCompletedError { source: StorageError, id: ProofID },
    #[snafu(display("Error marking proof request new: source: {}", source))]
    JoinHandleFailedError { source: JoinError },
    #[snafu(display("Failed to parse proof"))]
    ParseCompleteProofRequestError { source: CompleteProofError },
    #[snafu(display("Failed to convert complete proof to proxy proof format"))]
    ConvertCompleteProofError {
        source: hex::FromHexError,
        id: ProofID,
    },
    #[snafu(display("Failed to submit proofs onchain"))]
    SubmitProofsError {
        source: ethers::contract::ContractError<M>,
    },
    #[snafu(display("Failed to confirm transaction on chain"))]
    ConfirmationError {
        source: ProviderError,
        tx_hash: H256,
    },
}

impl<M: Middleware> BonsaiCompleteProofManagerError<M> {
    pub fn get_proof_request_id(self) -> Option<ProofID> {
        match self {
            BonsaiCompleteProofManagerError::MarkingProofRequestPreparingOnchainError {
                source: _,
                id,
            }
            | BonsaiCompleteProofManagerError::MarkingProofRequestCompletedOnchainError {
                source: _,
                id,
            } => Some(id),
            BonsaiCompleteProofManagerError::ConvertCompleteProofError { source: _, id } => {
                Some(id)
            }
            BonsaiCompleteProofManagerError::ParseCompleteProofRequestError { source } => {
                Some(source.get_proof_request_id())
            }
            _ => None,
        }
    }
}

// Cannot use async functions that return snafu errors with tokio::spawn cleanly
// so we isolate errors that might occur during tokio::spawn without using
// snafu.
#[derive(Debug, Display, Error)]
pub enum CompleteProofError {
    /// bonsai client error for proof `{id}`
    ClientAPI {
        source: BonsaiClientError,
        id: ProofID,
    },
    /// bonsai receipt was not found for proof `{id}`
    ReceiptNotFound { id: ProofID },
    /// bonsai inclusion proof was not found for proof `{id}`
    InclusionProofNotPresent { id: ProofID },
    /// bonsai merkle path was not found for proof `{id}`
    MerklePathNotPresent { id: ProofID },
    /// invalid merkle path for proof `{id}`
    FailedToDecodeMerklePath { id: ProofID },
    /// invalid block height for proof `{id}`
    FailedToConvertBlockHeight { id: ProofID },
    /// invalid hex fields for proof `{id}`
    HexParseError { id: ProofID },
    /// invalid hex length for proof `{id}`
    HexLengthError { id: ProofID },
}

impl CompleteProofError {
    pub fn get_proof_request_id(self) -> ProofID {
        match self {
            CompleteProofError::ReceiptNotFound { id }
            | CompleteProofError::InclusionProofNotPresent { id }
            | CompleteProofError::MerklePathNotPresent { id }
            | CompleteProofError::FailedToDecodeMerklePath { id }
            | CompleteProofError::FailedToConvertBlockHeight { id }
            | CompleteProofError::HexParseError { id }
            | CompleteProofError::HexLengthError { id } => id,
            CompleteProofError::ClientAPI { source: _, id } => id,
        }
    }
}
