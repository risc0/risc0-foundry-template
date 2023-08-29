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

#![no_main]

use std::io::Read;

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolType};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn fibonacci(n: u32) -> U256 {
    let (mut prev, mut curr) = (U256::from(1), U256::from(1));
    for _ in 2..=n {
        (prev, curr) = (curr, prev + curr);
    }
    curr
}

fn main() {
    // Read data sent from the application contract.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();
    // Type array passed to `ethabi::decode_whole` should match the types encoded in
    // the application contract.
    let (n,) = <sol!(tuple(uint32,))>::decode_params(&input_bytes, true).unwrap();

    // Run the computation.
    let result = fibonacci(n);

    // Commit the journal that will be received by the application contract.
    // Encoded types should match the args expected by the application callback.
    env::commit_slice(&<sol!(tuple(uint32, uint256))>::encode_params(&(n, result)));
}
