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

#![allow(clippy::expect_used)]

use std::{env, io, io::Write, time::Duration};

use bonsai_sdk_alpha::alpha::Client as AlphaClient;
use bonsai_starter_methods::GUEST_LIST;
use clap::Parser;
use risc0_zkvm::{recursion::SessionRollupReceipt, Executor, ExecutorEnv};

/// Runs the RISC-V ELF binary.
#[derive(Parser)]
#[clap(about, version, author)]
struct Args {
    /// The name of the guest binary
    guest_binary: String,

    /// The input to provide to the guest binary
    input: Option<String>,
}

fn prove_locally(elf: &[u8], input: Vec<u8>, prove: bool) -> Vec<u8> {
    let env = ExecutorEnv::builder().add_input(&input).build();
    let mut exec = Executor::from_elf(env, elf).expect("Failed to instantiate executor");
    let session = exec.run().expect("Failed to run executor");
    if prove {
        session.prove().expect("Failed to prove session");
        // eprintln!("Completed proof locally");
    } else {
        // eprintln!("Completed execution without a proof locally");
    }
    session.journal
}

const POLL_INTERVAL_SEC: u64 = 4;

fn prove_alpha(elf: &[u8], input: Vec<u8>) -> Vec<u8> {
    let client = AlphaClient::from_env().expect("Failed to create client from env var");

    let img_id = client
        .upload_img(elf.to_vec())
        .expect("Failed to upload ELF image");

    let input_id = client
        .upload_input(input)
        .expect("Failed to upload input data");

    let session = client
        .create_session(img_id, input_id)
        .expect("Failed to create remote proving session");

    loop {
        let res = match session.status(&client) {
            Ok(res) => res,
            Err(err) => {
                eprint!("Failed to get session status: {err}");
                std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
                continue;
            }
        };
        match res.status.as_str() {
            "RUNNING" => {
                std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
            }
            "SUCCEEDED" => {
                let receipt_buf = client
                    .download(
                        &res.receipt_url
                            .expect("Missing 'receipt_url' on status response"),
                    )
                    .expect("Failed to download receipt");
                let receipt: SessionRollupReceipt = bincode::deserialize(&receipt_buf)
                    .expect("Failed to deserialize SessionRollupReceipt");
                // eprintln!("Completed proof on bonsai alpha backend!");
                return receipt.journal;
            }
            _ => {
                panic!("Proving session exited with bad status: {}", res.status);
            }
        }
    }
}

fn main() {
    // Parse arguments
    let args = Args::parse();
    // Search list for requested binary name
    let potential_guest_image_id: [u8; 32] =
        match hex::decode(args.guest_binary.to_lowercase().trim_start_matches("0x")) {
            Ok(byte_vector) => byte_vector.try_into().unwrap_or([0u8; 32]),
            Err(_) => [0u8; 32],
        };
    let guest_entry = GUEST_LIST
        .iter()
        .find(|entry| {
            entry.name == args.guest_binary.to_uppercase()
                || bytemuck::cast::<[u32; 8], [u8; 32]>(entry.image_id) == potential_guest_image_id
        })
        .expect("Unknown guest binary");
    // Execute or return image id
    let output_bytes = match &args.input {
        Some(input) => {
            let input = hex::decode(&input[2..]).expect("Failed to decode input");
            let prover = env::var("PROVE_MODE").unwrap_or("".to_string());

            match prover.as_str() {
                "bonsai" => prove_alpha(guest_entry.elf, input),
                "local" => prove_locally(guest_entry.elf, input, true),
                "none" => prove_locally(guest_entry.elf, input, false),
                _ => prove_locally(guest_entry.elf, input, false),
            }
        }
        None => Vec::from(bytemuck::cast::<[u32; 8], [u8; 32]>(guest_entry.image_id)),
    };
    let output = hex::encode(output_bytes);
    print!("{output}");
    io::stdout().flush().expect("Failed to flush stdout buffer");
}
