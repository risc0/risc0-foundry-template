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

use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use anyhow::{Context, Result};
use clap::Parser;
use methods::GUEST_LIST;
use risc0_ethereum_sdk::{
    ethers::{self, Calldata},
    prover, resolve_guest_entry,
};
use risc0_zkvm::compute_image_id;

mod contract_interface;

// #[derive(Subcommand)]
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
        number: U256,
    },
}

// #[derive(Parser, Debug)]
// #[clap(author, version, about, long_about = None)]
// struct Args {
//     #[clap(short, long, require_equals = true)]
//     chain_id: u64,

//     #[clap(short, long, require_equals = true)]
//     rpc_url: String,

//     #[clap(short, long, require_equals = true)]
//     contract: String,

//     #[clap(short, long, require_equals = true)]
//     number: U256,
// }

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    match Command::parse() {
        Command::Query {
            guest_binary,
            input,
        } => {
            let elf = resolve_guest_entry(GUEST_LIST, &guest_binary)?;
            let image_id = compute_image_id(&elf)?;
            let output = match &input {
                Some(input) => {
                    let input = hex::decode(input.trim_start_matches("0x"))
                        .context("Failed to decode input")?;
                    let number = U256::f(&input);
                    let proof = prover::generate_proof(&elf, number)?;
                    let x = U256::from_be_slice(proof.journal.as_slice());
                    // println!("{:?}", proof.journal);
                    // let mut journal_le = proof.journal;
                    // journal_le.reverse();
                    let calldata = Calldata {
                        journal: x.abi_encode(),
                        post_state_digest: proof.post_state_digest,
                        seal: proof.seal,
                    };
                    hex::encode(calldata.abi_encode())
                }
                None => format!("0x{}", image_id.to_string()),
            };
            print!("{output}");
            std::io::stdout()
                .flush()
                .context("failed to flush stdout buffer")?;
        }
        Command::Publish {
            chain_id,
            rpc_url,
            contract,
            number,
        } => {
            let tx_sender = match std::env::var("ETH_WALLET_PRIVATE_KEY") {
                Ok(private_key) => Some(ethers::TxSender::new(
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

            println!("Number: {}", number);

            let proof = prover::generate_proof(methods::IS_EVEN_ELF, number)?;
            let x = U256::from_be_slice(proof.journal.as_slice());
            let calldata = contract_interface::set(x, proof.post_state_digest, proof.seal);
            if let Some(tx_sender) = tx_sender {
                tx_sender.send(calldata).await?;
            }
        }
    }

    Ok(())
}
