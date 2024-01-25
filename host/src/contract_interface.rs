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

use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolInterface};

sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes calldata seal, bytes32 post_state_digest);
    }
}

pub fn set(x: U256, seal: Vec<u8>, post_state_digest: FixedBytes<32>) -> Vec<u8> {
    let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
        x,
        seal,
        post_state_digest,
    });

    calldata.abi_encode()
}