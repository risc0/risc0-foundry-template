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

use bonsai_ethereum_cli::{
    deploy_starter_contract, get_ethers_client, get_wallet, get_ws_provider,
};
use bonsai_starter_methods::{FIBONACCI_ELF, FIBONACCI_ID};
use clap::{Parser, Subcommand};
use ethers::core::types::Address;
use risc0_zkvm::{MemoryImage, Program, MEM_SIZE, PAGE_SIZE};

const DEFAULT_ETH_NODE_URL: &str = "ws://anvil:8545";

#[derive(Subcommand)]
pub enum Command {
    /// Runs the RISC-V ELF binary.
    Execute {
        /// The input to provide to the guest binary
        input: String,
    },
    /// Prints the Image ID of the ELF.
    ImageId {},
    Deploy {
        /// Ethereum Proxy address
        #[arg(short, long)]
        relay_contract_address: Address,

        /// Ethereum Node endpoint
        #[arg(long, default_value_t = DEFAULT_ETH_NODE_URL.to_string())]
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
        Command::ImageId {} => {
            let program =
                Program::load_elf(FIBONACCI_ELF, MEM_SIZE as u32).expect("could not load elf");
            let image =
                MemoryImage::new(&program, PAGE_SIZE as u32).expect("could not get memory image");
            let image_id = hex::encode(image.compute_id());

            println!("image id: {}", image_id);
            std::io::stdout()
                .flush()
                .expect("Failed to flush stdout buffer");
        }
        Command::Execute { input } => {
            std::io::stdout()
                .flush()
                .expect("Failed to flush stdout buffer");
        }
        Command::Deploy {
            relay_contract_address,
            eth_node_url,
            eth_chain_id,
            private_key,
        } => {
            let eth_provider = get_ws_provider(&eth_node_url).await;
            let wallet = get_wallet(&private_key, eth_chain_id);
            let ethers_client = get_ethers_client(eth_provider, wallet).await;

            let starter_contract = deploy_starter_contract(ethers_client.clone()).await;
        }
    }
}
