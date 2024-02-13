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

use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use ethers::abi::Token;
use risc0_ethereum_sdk::prover;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The hex encoded guest binary
    guest_binary: String,

    /// The hex encoded input to provide to the guest binary
    input: String,
}

/// Prints on stdio the Ethereum ABI and hex encoded proof.
fn query(elf: Vec<u8>, input: Vec<u8>) -> Result<()> {
    let (journal, post_state_digest, seal) = prover::prove(&elf, &input)?;
    let calldata = vec![
        Token::Bytes(journal),
        Token::FixedBytes(post_state_digest.to_vec()),
        Token::Bytes(seal),
    ];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;
    Ok(())
}

/// Run the CLI.
pub fn main() -> Result<()> {
    let args = Args::parse();
    query(
        hex::decode(
            args.guest_binary
                .strip_prefix("0x")
                .unwrap_or(&args.guest_binary),
        )?,
        hex::decode(args.input.strip_prefix("0x").unwrap_or(&args.input))?,
    )?;

    Ok(())
}
