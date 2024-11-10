// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Generated crate containing the image ID and ELF binary of the build guest.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));

#[cfg(test)]
mod tests {
    use alloy_merkle_tree::incremental_tree::IncrementalMerkleTree;
    use alloy_primitives::{Address, B256};
    use mvm_core::ProofInput;
    use num_bigint::RandBigInt;
    use rand::Rng;
    use risc0_zkvm::{default_executor, ExecutorEnv};
    use sha2::{Digest, Sha256};

    type MerkleTree = IncrementalMerkleTree<10, Sha256>;

    #[test]
    fn can_spend() {
        let mut tree = MerkleTree::new();
        let rng = &mut rand::thread_rng();

        let note_spending_key = rng.gen_biguint(512).to_bytes_be();
        let (k, r) = note_spending_key.split_at(32);
        let l: usize = 11; // position to add our commitment

        let commitment = {
            let mut hasher = Sha256::new();
            hasher.update(&note_spending_key);
            hasher.finalize()
        };

        let nullifier_hash = {
            let mut hasher = Sha256::new();
            hasher.update(&k);
            hasher.finalize()
        };

        let recipient = Address::default();

        // insert some other random values into the merkle tree to pad
        for _ in 0..l {
            let value = rng.gen::<[u8; 32]>();
            tree.append(value.into()).expect("failed to append to tree");
        }

        // insert our commitment at position `l` and get opening proof
        tree.append(B256::from_slice(&commitment))
            .expect("failed to append commitment to tree");

        let opening = tree
            .proof_at_index(l)
            .expect("failed to generate proof")
            .into_iter()
            .collect();

        let input = ProofInput {
            root: tree.root(),
            leaf_index: l as u32,
            opening,
            k: B256::from_slice(&k),
            r: B256::from_slice(&r),
            recipient,
        };

        let env = ExecutorEnv::builder()
            .write_slice(&input.to_bytes().expect("failed to serialize proof input"))
            .build()
            .unwrap();

        // NOTE: Use the executor to run tests without proving.
        let session_info = default_executor()
            .execute(env, super::CAN_SPEND_ELF)
            .unwrap();

        println!("Session info: {:?}", session_info);
    }
}
