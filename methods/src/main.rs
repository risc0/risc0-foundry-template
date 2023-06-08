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

use std::{env, io, io::Write, time::Duration};

use bonsai_sdk::Client;
use bonsai_starter_methods::GUEST_LIST;
use clap::Parser;
use risc0_zkvm::{Executor, ExecutorEnv};

/// Runs the RISC-V ELF binary.
#[derive(Parser)]
#[clap(about, version, author)]
struct Args {
    /// The name of the guest binary
    guest_binary: String,

    /// The input to provide to the guest binary
    input: Option<String>,
}

fn prove_locally(elf: &[u8], input: Vec<u8>) -> Vec<u8> {
    let env = ExecutorEnv::builder().add_input(&input).build();
    let mut exec = Executor::from_elf(env, elf).expect("Failed to instantiate executor");
    let session = exec.run().expect("Failed to run executor");
    // Locally prove resulting journal
    if env::var("PROVE_LOCALLY").is_ok() {
        session.prove().expect("Failed to prove session");
    }
    session.journal
}

async fn prove_remotely(api_url: String, elf: &[u8], input: Vec<u8>) -> Vec<u8> {
    let api_key = match env::var("API_KEY") {
        Ok(api_key) => api_key,
        _ => "test_key".to_string(),
    };
    let client = Client::new(api_url, api_key).expect("Failed to instantiate Bonsai client");
    let image_id = client
        .put_image_from_elf(elf)
        .await
        .expect("Failed to upload elf to Bonsai")
        .image_id;
    let receipt_id = client
        .request_receipt(image_id, input)
        .await
        .expect("Failed to request receipt from Bonsai")
        .receipt_id;
    loop {
        match client.get_receipt(receipt_id).await {
            Ok(receipt) => return receipt.journal,
            Err(_) => std::thread::sleep(Duration::from_secs(15)),
        }
    }
}

#[tokio::main]
pub async fn main() {
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
            match env::var("BONSAI_ENDPOINT") {
                Ok(api_url) => prove_remotely(api_url, guest_entry.elf, input).await,
                Err(_) => prove_locally(guest_entry.elf, input),
            }
        }
        None => Vec::from(bytemuck::cast::<[u32; 8], [u8; 32]>(guest_entry.image_id)),
    };
    let output = hex::encode(&output_bytes);
    print!("{output}");
    io::stdout().flush().unwrap();
}
