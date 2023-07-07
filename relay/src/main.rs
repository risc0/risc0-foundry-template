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

use std::io::Write;

use bonsai_ethereum_relay::{
    create_ethers_client_private_key, resolve_guest_entry, resolve_image_output,
    run_with_ethers_client, Config,
};
use bonsai_starter_methods::GUEST_LIST;
use clap::{Parser, Subcommand};
use ethers::core::types::Address;

#[derive(Subcommand)]
pub enum Command {
    /// Runs the RISC-V ELF binary.
    Query {
        /// The name of the guest binary
        guest_binary: String,

        /// The input to provide to the guest binary
        input: Option<String>,
    },
    Relay {
        /// Ethereum Proxy address
        #[arg(short, long)]
        relay_contract_address: Address,

        /// Ethereum Node endpoint
        #[arg(long)]
        eth_node_url: String,

        /// Ethereum Chain ID
        #[arg(long, default_value_t = 31337)]
        eth_chain_id: u64,

        /// Wallet private key.
        #[arg(
            short,
            long,
            default_value = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        )]
        private_key: String,
    },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Command::Query {
            guest_binary,
            input,
        } => {
            // Search list for requested binary name
            let guest_entry = resolve_guest_entry(GUEST_LIST, &guest_binary)
                .expect("failed to resolve guest entry");

            // Execute or return image id
            let output_bytes = match &input {
                Some(input) => resolve_image_output(input, guest_entry).await,
                None => Ok(Vec::from(bytemuck::cast::<[u32; 8], [u8; 32]>(
                    guest_entry.image_id,
                ))),
            }
            .expect("failed to compute output");

            let output = hex::encode(output_bytes);
            print!("{output}");
            std::io::stdout()
                .flush()
                .expect("Failed to flush stdout buffer");
        }
        Command::Relay {
            relay_contract_address,
            eth_node_url,
            eth_chain_id,
            private_key,
        } => {
            let config = Config {
                proxy_address: relay_contract_address,
            };

            let ethers_client =
                create_ethers_client_private_key(&eth_node_url, &private_key, eth_chain_id).await;

            run_with_ethers_client(config, ethers_client).await
        }
    }
}
