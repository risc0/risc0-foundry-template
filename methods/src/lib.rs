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

//! Generated crate containing the image ID and ELF binary of the build guest.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));

use std::{env, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use bonsai_sdk_alpha::alpha::Client;
use risc0_zkvm::{recursion::SessionRollupReceipt, Executor, ExecutorEnv};

/// Execute and prove the guest locally, on this machine, as opposed to sending
/// the proof request to the Bonsai service.
pub fn prove_locally(elf: &[u8], input: Vec<u8>, prove: bool) -> Result<Vec<u8>> {
    // Execute the guest program, generating the session trace needed to prove the
    // computation.
    let env = ExecutorEnv::builder().add_input(&input).build();
    let mut exec = Executor::from_elf(env, elf).context("Failed to instantiate executor")?;
    let session = exec.run().context("Failed to run executor")?;

    // Locally prove resulting journal
    if prove {
        session.prove().context("Failed to prove session")?;
        // eprintln!("Completed proof locally");
    } else {
        // eprintln!("Completed execution without a proof locally");
    }
    Ok(session.journal)
}

pub const POLL_INTERVAL_SEC: u64 = 4;

pub fn prove_alpha(elf: &[u8], input: Vec<u8>) -> Result<Vec<u8>> {
    let client = Client::from_env().context("Failed to create client from env var")?;

    let img_id = client
        .upload_img(elf.to_vec())
        .context("Failed to upload ELF image")?;

    let input_id = client
        .upload_input(input)
        .context("Failed to upload input data")?;

    let session = client
        .create_session(img_id, input_id)
        .context("Failed to create remote proving session")?;

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
                            .context("Missing 'receipt_url' on status response")?,
                    )
                    .context("Failed to download receipt")?;
                let receipt: SessionRollupReceipt = bincode::deserialize(&receipt_buf)
                    .context("Failed to deserialize SessionRollupReceipt")?;
                // eprintln!("Completed proof on bonsai alpha backend!");
                return Ok(receipt.journal);
            }
            _ => {
                bail!("Proving session exited with bad status: {}", res.status);
            }
        }
    }
}

pub fn resolve_guest_entry<'a>(
    guest_list: &'a [GuestListEntry],
    guest_binary: &String,
) -> Result<&'a GuestListEntry> {
    // Search list for requested binary name
    let potential_guest_image_id: [u8; 32] =
        match hex::decode(guest_binary.to_lowercase().trim_start_matches("0x")) {
            Ok(byte_vector) => byte_vector.try_into().unwrap_or([0u8; 32]),
            Err(_) => [0u8; 32],
        };
    guest_list
        .iter()
        .find(|entry| {
            entry.name == guest_binary.to_uppercase()
                || bytemuck::cast::<[u32; 8], [u8; 32]>(entry.image_id) == potential_guest_image_id
        })
        .ok_or_else(|| {
            let found_guests: Vec<String> = guest_list
                .iter()
                .map(|g| hex::encode(bytemuck::cast::<[u32; 8], [u8; 32]>(g.image_id)))
                .collect();
            anyhow!(
                "Unknown guest binary {}, found: {:?}",
                guest_binary,
                found_guests
            )
        })
}

pub fn resolve_image_output(input: &String, guest_entry: &GuestListEntry) -> Result<Vec<u8>> {
    let input = hex::decode(&input[2..]).context("Failed to decode input")?;
    let prover = env::var("BONSAI_PROVING").unwrap_or("".to_string());

    match prover.as_str() {
        "bonsai" => prove_alpha(guest_entry.elf, input),
        "local" => prove_locally(guest_entry.elf, input, true),
        _ => prove_locally(guest_entry.elf, input, false),
    }
}
