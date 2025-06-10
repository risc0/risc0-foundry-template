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

// This application demonstrates how to send an off-chain proof request
// to the Bonsai proving service and publish the received proofs directly
// to your deployed app contract.

use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
    sol_types::SolValue,
};
use anyhow::{Context, Result};
use clap::Parser;
use methods::{IS_EVEN_ELF, IS_EVEN_ID};
use risc0_ethereum_contracts::IRiscZeroVerifier;
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv, ProverOpts, VerifierContext};
use url::Url;

/// Arguments of the publisher CLI.
/// ```sh
/// RPC_URL=https://ethereum-sepolia-rpc.publicnode.com \
/// VERIFIER_ADDRESS=0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
/// cargo run
/// ```
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum Node endpoint.
    #[clap(long, env)]
    rpc_url: Url,

    /// Address of the verifier to test against. Should be set to the address of the
    /// RiscZeroVerifierRouter on the given chain.
    // NOTE: The verifier address can be different on each chain but does not change between
    // versions and so could be part of a static config. One option would be to pull it from the
    // `contracts/deployment.toml` file in the risc0-ethereum repo.
    #[clap(long, env)]
    verifier_address: Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI Arguments: The application starts by parsing command-line arguments provided by the user.
    let args = Args::parse();

    let receipt = tokio::task::spawn_blocking(move || {
        // NOTE: What is proven does not matter for the purposes of this test. The is_even guest is
        // used because the starting point was the Foundry template.
        let env = ExecutorEnv::builder()
            .write_slice(&U256::ZERO.abi_encode())
            .build()?;
        default_prover().prove_with_ctx(
            env,
            &VerifierContext::default(),
            IS_EVEN_ELF,
            &ProverOpts::groth16(),
        )
    })
    .await??
    .receipt;

    // Create an alloy provider with the given RPC URL.
    let provider = ProviderBuilder::new()
        .connect(args.rpc_url.as_str())
        .await?;

    // Encode the seal with the selector.
    let seal = risc0_ethereum_contracts::encode_seal(&receipt)?;

    // IRiscZeroVerifier::verify has no return, and the Solidity implementations revert on
    // verification failure. If it reverts, the call result will be an error.
    let contract = IRiscZeroVerifier::new(args.verifier_address, provider);
    contract
        .verify(
            seal.into(),
            bytemuck::cast::<_, [u8; 32]>(IS_EVEN_ID).into(),
            <[u8; 32]>::from(receipt.journal.digest()).into(),
        )
        .call()
        .await
        .context("IRiscZeroVerifier::verify call failed")?;

    Ok(())
}
