use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bonsai_sdk::types::ProofID;
use snafu::prelude::*;

use crate::storage::{Error, ProofRequestState, ProofRequstInformation, Storage};

#[derive(Debug, Clone)]
pub struct InMemoryStorage {
    proof_states: Arc<RwLock<HashMap<ProofID, ProofRequestState>>>,
    new_proofs: Arc<RwLock<HashMap<ProofID, ProofRequstInformation>>>,
    pending_proofs: Arc<RwLock<HashMap<ProofID, ProofRequstInformation>>>,
    completed_proofs: Arc<RwLock<HashMap<ProofID, ProofRequstInformation>>>,
    preparing_onchain_proofs: Arc<RwLock<HashMap<ProofID, ProofRequstInformation>>>,
}

#[derive(Debug, Snafu)]
pub enum InMemoryStorageError {
    #[snafu(display("Invalid proof state transition_proof_request "))]
    InvalidProofStateTransition {
        proof_id: ProofID,
        from_state: ProofRequestState,
        new_state: ProofRequestState,
    },
}

impl From<InMemoryStorageError> for Error {
    fn from(err: InMemoryStorageError) -> Error {
        match err {
            InMemoryStorageError::InvalidProofStateTransition { .. } => {
                Error::TransitionProofRequestError {
                    source: Box::new(err),
                }
            }
        }
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            proof_states: Arc::new(RwLock::new(HashMap::new())),
            new_proofs: Arc::new(RwLock::new(HashMap::new())),
            pending_proofs: Arc::new(RwLock::new(HashMap::new())),
            completed_proofs: Arc::new(RwLock::new(HashMap::new())),
            preparing_onchain_proofs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_proof_request_set_for_state(
        &self,
        state: ProofRequestState,
    ) -> Arc<RwLock<HashMap<ProofID, ProofRequstInformation>>> {
        match state {
            ProofRequestState::New => self.new_proofs.clone(),
            ProofRequestState::Pending => self.pending_proofs.clone(),
            ProofRequestState::Completed => self.completed_proofs.clone(),
            ProofRequestState::PreparingOnchain => self.preparing_onchain_proofs.clone(),
            ProofRequestState::CompletedOnchain(_) => Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Storage for InMemoryStorage {
    async fn add_new_bonsai_proof_request(
        &self,
        proof: ProofRequstInformation,
    ) -> Result<(), Error> {
        self.proof_states
            .write()
            .expect("write lock should not be poisoned")
            .insert(proof.proof_request_id, ProofRequestState::New);
        self.new_proofs
            .write()
            .expect("write lock should not be poisoned")
            .insert(proof.proof_request_id, proof);

        Ok(())
    }

    async fn fetch_new_bonsai_requests(
        &self,
        _limit: Option<u64>,
    ) -> Result<Vec<ProofRequstInformation>, Error> {
        let hashmap = self
            .new_proofs
            .read()
            .expect("read lock should not be poisoned");

        Ok(hashmap.values().cloned().collect())
    }

    async fn fetch_completed_bonsai_requests(
        &self,
        _limit: Option<u64>,
    ) -> Result<Vec<ProofRequstInformation>, Error> {
        let hashmap = self
            .completed_proofs
            .read()
            .expect("read lock should not be poisoned");

        Ok(hashmap.values().cloned().collect())
    }

    async fn fetch_preparing_onchain_proof_requests(
        &self,
        _limit: Option<u64>,
    ) -> Result<Vec<ProofRequstInformation>, Error> {
        let hashmap = self
            .preparing_onchain_proofs
            .read()
            .expect("read lock should not be poisoned");

        Ok(hashmap.values().cloned().collect())
    }

    async fn get_proof_request_state(&self, proof_id: ProofID) -> Result<ProofRequestState, Error> {
        match self
            .proof_states
            .read()
            .expect("read lock should not be poisoned")
            .get(&proof_id)
        {
            Some(state) => Ok(*state),
            None => return Err(Error::ProofNotFound { id: proof_id }.into()),
        }
    }

    async fn transition_proof_request(
        &self,
        proof_id: ProofID,
        new_state: ProofRequestState,
    ) -> Result<(), Error> {
        let mut proof_states_locked = self
            .proof_states
            .write()
            .expect("write lock should not be poisoned");

        let current_state = match proof_states_locked.get(&proof_id) {
            Some(proof_state) => proof_state.clone(),
            None => return Err(Error::ProofNotFound { id: proof_id }.into()),
        };

        if !current_state.is_valid_state_transition(new_state) {
            return Err(InMemoryStorageError::InvalidProofStateTransition {
                proof_id,
                from_state: current_state,
                new_state,
            }
            .into());
        }

        let from_set = self.get_proof_request_set_for_state(current_state);
        let to_set = self.get_proof_request_set_for_state(new_state);

        // remove from the from_set and add to the to_set
        let mut from_set_locked = from_set.write().expect("write lock should not be poisoned");
        let mut to_set_locked = to_set.write().expect("write lock should not be poisoned");

        let proof = {
            let proof = match from_set_locked.get(&proof_id) {
                Some(proof) => proof.clone(),
                None => return Err(Error::ProofNotFound { id: proof_id }.into()),
            };

            from_set_locked.remove(&proof_id);
            proof.clone()
        };

        match new_state {
            ProofRequestState::CompletedOnchain(_) => {
                // We dont need to store onchain transactions in memory
                proof_states_locked.remove(&proof_id);
                return Ok(());
            }
            _ => (),
        };

        to_set_locked.insert(proof.proof_request_id, proof);

        proof_states_locked.insert(proof_id, new_state);

        Ok(())
    }
}
