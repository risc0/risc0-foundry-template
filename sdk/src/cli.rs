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

use crate::{eth, prover, resolve_guest_entry, snark::Proof};
use anyhow::{Context, Result};
use clap::Parser;
use methods::GUEST_LIST;
use risc0_zkvm::compute_image_id;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Command {
    /// Runs the RISC-V ELF binary.
    Query {
        /// The name of the guest binary
        guest_binary: String,

        /// The input to provide to the guest binary
        input: Option<String>,
    },
    /// Upload the RISC-V ELF binary to Bonsai.
    Publish {
        #[clap(short, long, require_equals = true)]
        chain_id: u64,

        #[clap(short, long, require_equals = true)]
        rpc_url: String,

        #[clap(short, long, require_equals = true)]
        contract: String,

        #[clap(short, long, require_equals = true)]
        input: String,
    },
}

pub fn query<T: serde::Serialize + Sized>(
    guest_binary: String,
    input: Option<T>,
    parser: fn(input: T) -> Result<Vec<u8>>,
) -> Result<()> {
    let elf = resolve_guest_entry(GUEST_LIST, &guest_binary)?;
    let image_id = compute_image_id(&elf)?;
    let output = match input {
        Some(input) => {
            let proof = prover::generate_proof(&elf, parser(input)?)?;
            hex::encode(proof.abi_encode())
        }
        None => format!("0x{}", image_id.to_string()),
    };
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;
    Ok(())
}

pub fn publish<T: serde::Serialize + Sized>(
    elf: &[u8],
    chain_id: u64,
    rpc_url: String,
    contract: String,
    input: T,
    parse_input: fn(input: T) -> Result<Vec<u8>>,
    parse_output: fn(proof: Proof) -> Result<Vec<u8>>,
) -> Result<()> {
    let tx_sender = match std::env::var("ETH_WALLET_PRIVATE_KEY") {
        Ok(private_key) => Some(eth::TxSender::new(
            chain_id,
            &rpc_url,
            &private_key,
            &contract,
        )?),
        _ => None,
    };

    if tx_sender.is_some() {
        println!("Private key is set; transaction will be sent");
    }
    let proof = prover::generate_proof(elf, parse_input(input)?)?;
    let calldata = parse_output(proof)?;

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        if let Some(tx_sender) = tx_sender {
            tx_sender.send(calldata).await.unwrap();
        }
    });

    Ok(())
}

pub fn run(
    elf: &[u8],
    parse_input: fn(input: String) -> Result<Vec<u8>>,
    parse_output: fn(proof: Proof) -> Result<Vec<u8>>,
) -> Result<()> {
    match Command::parse() {
        Command::Query {
            guest_binary,
            input,
        } => query(guest_binary, input, parse_input)?,
        Command::Publish {
            chain_id,
            rpc_url,
            contract,
            input,
        } => publish(
            elf,
            chain_id,
            rpc_url,
            contract,
            input,
            parse_input,
            parse_output,
        )?,
    }

    Ok(())
}
