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
use alloy_sol_types::{sol, SolValue};
use anyhow::{ensure, Result};
use bonsai_sdk::alpha as bonsai_sdk;
use ethers::abi::Token;

sol! {
    #[derive(Debug)]
    struct Seal {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }
}

impl Seal {
    pub fn abi_encode(seal: bonsai_sdk::responses::Groth16Seal) -> Result<Vec<u8>> {
        let seal = Seal::try_from(seal)?;
        Ok(seal.abi_encode())
    }
}

impl TryFrom<bonsai_sdk::responses::Groth16Seal> for Seal {
    type Error = anyhow::Error;

    fn try_from(seal: bonsai_sdk::responses::Groth16Seal) -> Result<Self> {
        ensure!(
            seal.a.len() == 2,
            "seal.a has invalid length: {}",
            seal.a.len()
        );
        ensure!(
            seal.b.len() == 2,
            "seal.b has invalid length: {}",
            seal.b.len()
        );
        ensure!(
            seal.b[0].len() == 2,
            "seal.b[0] has invalid length: {}",
            seal.b[0].len()
        );
        ensure!(
            seal.b[1].len() == 2,
            "seal.b[0] has invalid length: {}",
            seal.b[1].len()
        );
        ensure!(
            seal.c.len() == 2,
            "seal.c has invalid length: {}",
            seal.c.len()
        );

        let a0 = U256::from_be_slice(seal.a[0].as_slice());
        let a1 = U256::from_be_slice(seal.a[1].as_slice());
        let b00 = U256::from_be_slice(seal.b[0][0].as_slice());
        let b01 = U256::from_be_slice(seal.b[0][1].as_slice());
        let b10 = U256::from_be_slice(seal.b[1][0].as_slice());
        let b11 = U256::from_be_slice(seal.b[1][1].as_slice());
        let c0 = U256::from_be_slice(seal.c[0].as_slice());
        let c1 = U256::from_be_slice(seal.c[1].as_slice());

        Ok(Seal {
            a: [a0, a1],
            b: [[b00, b01], [b10, b11]],
            c: [c0, c1],
        })
    }
}

#[derive(Clone, Debug)]
pub struct Proof {
    pub journal: Vec<u8>,
    pub post_state_digest: FixedBytes<32>,
    pub seal: Vec<u8>,
}

impl Proof {
    pub fn new_empty(journal: Vec<u8>) -> Self {
        Self {
            journal,
            post_state_digest: FixedBytes::<32>::default(),
            seal: vec![],
        }
    }

    pub fn abi_encode(self) -> Vec<u8> {
        let calldata = vec![
            Token::Bytes(self.journal),
            Token::FixedBytes(self.post_state_digest.to_vec()),
            Token::Bytes(self.seal),
        ];
        ethers::abi::encode(&calldata)
    }
}
