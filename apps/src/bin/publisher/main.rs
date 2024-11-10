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
    network::EthereumWallet,
    providers::{ProviderBuilder, WalletProvider},
    signers::local::PrivateKeySigner,
};
use alloy_primitives::{Address, U256};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use url::Url;

mod abi;
mod deposit;
mod withdraw;

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum chain ID
    #[clap(long)]
    chain_id: u64,

    /// Ethereum Node endpoint.
    #[clap(long, env)]
    eth_wallet_private_key: PrivateKeySigner,

    /// Ethereum Node endpoint.
    #[clap(long)]
    rpc_url: Url,

    /// Application's contract address on Ethereum
    #[clap(long)]
    contract: Address,

    /// The height at which the contract was deployed
    #[clap(long)]
    contract_deploy_height: u64,

    /// The note size, N, used by this contract in wei
    #[clap(long)]
    #[clap(default_value = "1000000000000000")] // 1 eth
    note_size: U256,

    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Deposit N eth into the contract and generate the withdrawal key
    Deposit,

    /// Withdraw N eth from the contract using the withdrawal key
    Withdraw { spending_key: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Create an alloy provider for that private key and URL.
    let wallet = EthereumWallet::from(args.eth_wallet_private_key);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(args.rpc_url);
    let contract = abi::ITornado::new(args.contract, provider);

    match args.command {
        SubCommand::Deposit => deposit::deposit(&contract, args.note_size).await?,
        SubCommand::Withdraw { spending_key } => {
            withdraw::withdraw(
                &contract,
                args.contract_deploy_height,
                contract.provider().default_signer_address(),
                decode_spending_key(&spending_key)?,
            )
            .await?
        }
    }

    Ok(())
}

fn decode_spending_key(spending_key: &str) -> Result<[u8; 64]> {
    let spending_key = hex::decode(spending_key.trim_start_matches("0x"))
        .context("failed to decode spending key hex")?;
    spending_key
        .as_slice()
        .try_into()
        .context("spending key must be exactly 64 bytes")
}
