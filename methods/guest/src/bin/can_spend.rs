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

use std::io::Read;

use alloy_merkle_tree::incremental_tree::IncrementalMerkleTree;
use alloy_primitives::B256;
use mvm_core::ProofInput;
use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

type MerkleTree = IncrementalMerkleTree<10, Sha256>;

fn main() {
    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    // Decode and parse the input
    let input = ProofInput::try_from_bytes(&input_bytes).unwrap();

    // hash the nullifier so we can include commit it to the journal enforcing the constraint (nullifier_hash = H(k))
    let nullifier_hash = {
        let mut hasher = Sha256::new();
        let preimage = [input.k.as_slice(), input.r.as_slice()].concat();
        hasher.update(&preimage);
        hasher.finalize()
    };

    // calculate the commitment and use this when checking the merkle proof
    let commitment = {
        let mut hasher = Sha256::new();
        hasher.update(&input.k);
        hasher.update(&input.r);
        hasher.finalize()
    };

    // check the opening proof and panic if it is invalid
    assert!(
        MerkleTree::verify_proof_against_root(
            input.root,
            B256::from_slice(&commitment),
            input.leaf_index.try_into().unwrap(),
            &input.opening.try_into().unwrap(),
        ),
        "invalid opening proof"
    );

    // Commit the instance/public values (tree root, nullifier hash, and recipient) to the journal
    // in an ABI friendly way
    env::commit_slice(input.root.as_slice());
    env::commit_slice(B256::from_slice(&nullifier_hash).as_slice());
    env::commit_slice(input.recipient.as_slice());
}
