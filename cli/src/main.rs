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
use anyhow::Result;
use bonsai_starter_sdk::{bonsai, ethers};
use clap::Parser;

mod contract_interface;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, require_equals = true)]
    chain_id: u64,

    #[clap(short, long, require_equals = true)]
    rpc_url: String,

    #[clap(short, long, require_equals = true)]
    contract: String,

    #[clap(short, long, require_equals = true)]
    number: U256,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let tx_sender = match std::env::var("ETH_WALLET_PRIVATE_KEY") {
        Ok(private_key) => Some(ethers::TxSender::new(
            args.chain_id,
            &args.rpc_url,
            &private_key,
            &args.contract,
        )?),
        _ => None,
    };

    if tx_sender.is_some() {
        println!("Private key is set; transaction will be sent");
    }

    println!("Number: {}", args.number);

    let (seal, post_state_digest, journal) = bonsai::prove(methods::IS_EVEN_ELF, args.number)?;

    print!("seal: ");
    for b in &seal {
        print!("\\x{:02x}", b);
    }
    println!("");
    println!("post_state_digest: {}", post_state_digest);
    print!("journal: ");
    for b in &journal {
        print!("\\x{:02x}", b);
    }
    println!("");

    let x = U256::from_be_slice(journal.as_slice());

    if let Some(tx_sender) = tx_sender {
        tx_sender
            .send(contract_interface::set(x, seal, post_state_digest))
            .await?;
    }

    Ok(())
}
