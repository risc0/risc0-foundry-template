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

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::{Context, Result};
use clap::Parser;
use risc0_ethereum_sdk::{eth::TxSender, prover};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum chain ID
    #[clap(long)]
    chain_id: u64,

    /// Ethereum Node endpoint.
    #[clap(long, env)]
    eth_wallet_private_key: String,

    /// Ethereum Node endpoint.
    #[clap(long)]
    rpc_url: String,

    /// Application's contract address on Ethereum
    #[clap(long)]
    contract: String,

    /// The path of the guest binary
    #[clap(long)]
    guest_binary: String,

    /// The input to provide to the guest binary
    #[clap(short, long)]
    input: String,
}

// `IEvenNumber`` interface automatically generated via the alloy `sol!` macro.
// The `set` function is then used as part of the `calldata` function of the
// `EvenNumberInterface`.
sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes32 post_state_digest, bytes calldata seal);
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let elf = std::fs::read(args.guest_binary)?;
    let tx_sender = TxSender::new(
        args.chain_id,
        &args.rpc_url,
        &args.eth_wallet_private_key,
        &args.contract,
    )?;

    let input = hex::decode(args.input.strip_prefix("0x").unwrap_or(&args.input))?;
    let (journal, post_state_digest, seal) = prover::prove(&elf, &input)?;

    // Decode the journal. Must match what was written in the guest with `env::commit_slice`
    let x = U256::abi_decode(&journal, true).context("decoding journal data")?;

    // Encode the function call for `IEvenNumber.set(x)`
    let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
        x,
        post_state_digest,
        seal,
    })
    .abi_encode();

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(tx_sender.send(calldata))?;

    Ok(())
}
