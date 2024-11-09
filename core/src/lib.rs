use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofInput {
    pub root: [u8; 32],         // root of the merkle tree (public)
    pub k: [u8; 32],            // nullifier (first part of spending key)
    pub r: [u8; 32],            // randomness (second part of spending key)
    pub leaf_index: u32,        // note leaf index in the merkle tree
    pub opening: Vec<[u8; 32]>, // merkle tree opening proof (merkle path)
}

impl ProofInput {
    pub fn new(
        root: [u8; 32],
        k: [u8; 32],
        r: [u8; 32],
        leaf_index: u32,
        opening: Vec<[u8; 32]>,
    ) -> Self {
        Self {
            root,
            k,
            r,
            leaf_index,
            opening,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(&bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self)
    }
}
