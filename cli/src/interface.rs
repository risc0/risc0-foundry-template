// Copyright 2024 RISC Zero, Inc.
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

use std::str::FromStr;

use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolInterface};
use anyhow::Result;
use risc0_ethereum_sdk::{serialize, snark::Proof};

sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes32 post_state_digest, bytes calldata seal);
    }
}

/// Rust interface of the `set` contract's function.
pub fn set(x: U256, post_state_digest: FixedBytes<32>, seal: Vec<u8>) -> Vec<u8> {
    let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
        x,
        seal,
        post_state_digest,
    });

    calldata.abi_encode()
}

/// Input parser from string to an encoded `Vec<u8>` compatible with the zkVM and Bonsai.
pub fn parse_input(input: String) -> Result<Vec<u8>> {
    serialize(U256::from_str(&input)?)
}

/// Parses a proof to extract the required output from the journal.
pub fn parse_output(proof: Proof) -> Result<Vec<u8>> {
    let output = U256::from_be_slice(proof.journal.as_slice());
    let calldata = set(output, proof.post_state_digest, proof.seal);
    Ok(calldata)
}
