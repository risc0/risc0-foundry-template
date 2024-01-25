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

use std::time::Duration;

use alloy_primitives::{FixedBytes, U256};
use anyhow::{ensure, Context, Result};
use bonsai_sdk::alpha as bonsai_sdk;
use clap::Parser;
use risc0_zkvm::{serde::to_vec, Receipt};

mod ethers;
mod contract_interface;
mod seal;

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

    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;

    // Compute the image_id, then upload the ELF with the image_id as its key.
    let image_id = {
        let program = risc0_zkvm::Program::load_elf(
            methods::IS_EVEN_ELF,
            risc0_zkvm::GUEST_MAX_MEM as u32,
        )
        .expect("Could not load ELF");
        let image = risc0_zkvm::MemoryImage::new(&program, risc0_zkvm::PAGE_SIZE as u32)
            .expect("Could not create memory image");
        hex::encode(image.compute_id())
    };
    client.upload_img(&image_id, methods::IS_EVEN_ELF.to_vec())?;
    eprintln!("Image ID: 0x{}", image_id);

    // Prepare input data and upload it.
    let input_id = {
        let input_data = to_vec(&args.number).unwrap();
        let input_data = bytemuck::cast_slice(&input_data).to_vec();
        client.upload_input(input_data)?
    };

    // Start a session running the prover
    let session = client.create_session(image_id, input_id)?;
    eprintln!("Created session: {}", session.uuid);
    let receipt = loop {
        let res = session.status(&client)?;
        if res.status == "RUNNING" {
            eprintln!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");

            let receipt_buf = client.download(&receipt_url)?;
            let receipt: Receipt = bincode::deserialize(&receipt_buf)?;

            break receipt;
        } else {
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }
    };

    // Verify the receipt
    {
        receipt
            .verify(methods::IS_EVEN_ID)
            .expect("Receipt verification failed");
        println!("Journal digest: {:?}", receipt.get_metadata()?.output);

        let committed_number = U256::from_be_slice(receipt.journal.bytes.as_slice());
        ensure!(
            args.number == committed_number,
            "Commitment mismatch: {} != {}",
            args.number,
            committed_number
        );

        eprintln!("Receipt verified");
    }

    // Fetch the snark
    let snark_session = client.create_snark(session.uuid)?;
    eprintln!("Created snark session: {}", snark_session.uuid);
    let snark_receipt = loop {
        let res = snark_session.status(&client)?;
        match res.status.as_str() {
            "RUNNING" => {
                eprintln!("Current status: {} - continue polling...", res.status,);
                std::thread::sleep(Duration::from_secs(15));
                continue;
            }
            "SUCCEEDED" => {
                break res.output.expect("No snark generated :(");
            }
            _ => {
                panic!(
                    "Workflow exited: {} err: {}",
                    res.status,
                    res.error_msg.unwrap_or_default()
                );
            }
        }
    };

    eprintln!("Snark proof!: {snark_receipt:?}");

    let seal = seal::Seal::abi_encode(snark_receipt.snark).context("Read seal")?;
    let post_state_digest: FixedBytes<32> = snark_receipt
        .post_state_digest
        .as_slice()
        .try_into()
        .context("Read post_state_digest")?;

    print!("seal: ");
    for b in &seal {
        print!("\\x{:02x}", b);
    }
    println!("");
    println!("post_state_digest: {}", post_state_digest);

    if let Some(tx_sender) = tx_sender {
        tx_sender
            .send(contract_interface::set(args.number, seal, post_state_digest))
            .await?;
    }

    Ok(())
}
