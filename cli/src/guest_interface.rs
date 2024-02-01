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

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::Result;
use risc0_ethereum_sdk::{cli::GuestInterface, serialize, snark::Proof};

// You can modify this file to implement the `GuestInterface` trait
// that lets you define how to parse and serialize the guest input and calldata
// so that your contract can interact with the RISC Zero zkVM and Bonsai.

// `IEvenNumber`` interface automatically generated via the alloy `sol!` macro.
// The `set` function is then used as part of the `calldata` function of the
// `EvenNumberInterface`.
sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes32 post_state_digest, bytes calldata seal);
    }
}

// `EvenNumberInterface` implementing the `GuestInterface` trait.
pub struct EvenNumberInterface {}
impl GuestInterface for EvenNumberInterface {
    // Parses a `String` as the guest input returning its serialization,
    // encoded as `Vec<u8>`, compatible with the zkVM and Bonsai.
    fn serialize_input(&self, input: String) -> Result<Vec<u8>> {
        serialize(U256::from_str(&input)?)
    }

    // Extracts the calldata ABI encoded from a proof.
    fn calldata(&self, proof: Proof) -> Result<Vec<u8>> {
        let x = U256::abi_decode(&proof.journal, true)?;
        let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
            x,
            post_state_digest: proof.post_state_digest,
            seal: proof.seal,
        });

        Ok(calldata.abi_encode())
    }
}
