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
    use alloy_primitives::U256;
    use risc0_zkvm::{default_prover, ExecutorEnv};

    #[test]
    fn proves_even_number() {
        let even_number = U256::from(1304);

        let env = ExecutorEnv::builder()
            .write(&even_number)
            .unwrap()
            .build()
            .unwrap();

        let receipt = default_prover().prove(env, super::IS_EVEN_ELF).unwrap();

        let x: U256 = receipt.journal.decode().unwrap();
        assert_eq!(x, even_number);
    }

    #[test]
    #[should_panic(expected = "number is not even")]
    fn rejects_odd_number() {
        let odd_number = U256::from(75);

        let env = ExecutorEnv::builder()
            .write(&odd_number)
            .unwrap()
            .build()
            .unwrap();

        default_prover().prove(env, super::IS_EVEN_ELF).unwrap();
    }
}
