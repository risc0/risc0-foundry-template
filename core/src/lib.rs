use alloy_primitives::B256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofInput {
    pub root: B256,         // root of the merkle tree (public)
    pub k: B256,            // nullifier (first part of spending key)
    pub r: B256,            // randomness (second part of spending key)
    pub leaf_index: u32,    // note leaf index in the merkle tree
    pub opening: Vec<B256>, // merkle tree opening proof (merkle path)
}

impl ProofInput {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(&bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self)
    }
}
