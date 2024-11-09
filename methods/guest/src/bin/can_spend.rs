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

use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use mvm_core::ProofInput;
use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() {
    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    // Decode and parse the input
    let input = ProofInput::try_from_bytes(&input_bytes).unwrap();

    // hash the nullifier so we can include commit it to the journal enforcing the constraint (nullifier_hash = H(k))
    let nullifier_hash = {
        let mut hasher = Sha256::new();
        hasher.update(&input.k);
        hasher.finalize()
    };

    // calculate the commitment and use this when checking the merjle proof
    let commitment = {
        let mut hasher = Sha256::new();
        hasher.update(&input.k);
        hasher.update(&input.r);
        hasher.finalize()
    };

    // Commit the public values (tree root and nullifier hash) to the journal
    env::commit_slice(input.root.abi_encode().as_slice());
    env::commit_slice(nullifier_hash.abi_encode().as_slice());
}
