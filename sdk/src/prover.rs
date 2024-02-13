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

use alloy_primitives::FixedBytes;
use anyhow::{Context, Result};
use bonsai_sdk::alpha as bonsai_sdk;
use risc0_zkvm::{compute_image_id, default_executor, is_dev_mode, ExecutorEnv, Receipt};

use crate::snark::Seal;

/// Generates a snark proof as a triplet (`Vec<u8>`, `FixedBytes<32>`, `Vec<u8>)
/// for the given elf and input.
/// When `RISC0_DEV_MODE` is set, executes the elf locally,
/// as opposed to sending the proof request to the Bonsai service.
pub fn prove(elf: &[u8], input: &[u8]) -> Result<(Vec<u8>, FixedBytes<32>, Vec<u8>)> {
    match is_dev_mode() {
        true => DevModeProver::prove(elf, input),
        false => BonsaiProver::prove(elf, input),
    }
}

trait Prover {
    fn prove(elf: &[u8], input: &[u8]) -> Result<(Vec<u8>, FixedBytes<32>, Vec<u8>)>;
}

struct DevModeProver {}

impl DevModeProver {
    fn prove(elf: &[u8], input: &[u8]) -> Result<(Vec<u8>, FixedBytes<32>, Vec<u8>)> {
        let env = ExecutorEnv::builder()
            .write_slice(&serialize_input_to_bytes(input)?)
            .build()
            .context("Failed to build exec env")?;
        let exec = default_executor();
        let session_info = exec.execute(env, elf).context("Failed to run executor")?;

        Ok((
            session_info.journal.bytes,
            FixedBytes::<32>::default(),
            Vec::new(),
        ))
    }
}

/// Serializes the given input as a `Vec<u8>` compatible with Bonsai.
pub(crate) fn serialize_input_to_bytes(input: impl serde::Serialize) -> Result<Vec<u8>> {
    let input_encoded = risc0_zkvm::serde::to_vec(&input)?;
    Ok(bytemuck::cast_slice(&input_encoded).to_vec())
}

struct BonsaiProver {}
impl BonsaiProver {
    fn prove(elf: &[u8], input: &[u8]) -> Result<(Vec<u8>, FixedBytes<32>, Vec<u8>)> {
        let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;

        // Compute the image_id, then upload the ELF with the image_id as its key.
        let image_id = compute_image_id(elf)?;
        let image_id_hex = image_id.to_string();
        client.upload_img(&image_id_hex, elf.to_vec())?;
        log::info!("Image ID: 0x{}", image_id_hex);

        // Prepare input data and upload it.
        let input_bytes = serialize_input_to_bytes(input)?;
        let input_id = client.upload_input(input_bytes)?;

        // Start a session running the prover.
        let session = client.create_session(image_id_hex, input_id, vec![])?;
        log::info!("Created session: {}", session.uuid);
        let _receipt = loop {
            let res = session.status(&client)?;
            if res.status == "RUNNING" {
                log::info!(
                    "Current status: {} - state: {} - continue polling...",
                    res.status,
                    res.state.unwrap_or_default()
                );
                std::thread::sleep(Duration::from_secs(15));
                continue;
            }
            if res.status == "SUCCEEDED" {
                // Download the receipt, containing the output.
                let receipt_url = res
                    .receipt_url
                    .context("API error, missing receipt on completed session")?;

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

        // Fetch the snark.
        let snark_session = client.create_snark(session.uuid)?;
        log::info!("Created snark session: {}", snark_session.uuid);
        let snark_receipt = loop {
            let res = snark_session.status(&client)?;
            match res.status.as_str() {
                "RUNNING" => {
                    log::info!("Current status: {} - continue polling...", res.status,);
                    std::thread::sleep(Duration::from_secs(15));
                    continue;
                }
                "SUCCEEDED" => {
                    break res.output.context("No snark generated :(")?;
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

        let snark = snark_receipt.snark;
        log::debug!("Snark proof!: {snark:?}");

        // // Verify receipt.
        // let receipt = Receipt {
        //     inner: risc0_zkvm::InnerReceipt::Groth16(Groth16Receipt {
        //         seal: Groth16Seal {
        //             a: snark.a.clone(),
        //             b: snark.b.clone(),
        //             c: snark.c.clone(),
        //         }
        //         .to_vec(),
        //         claim: receipt.get_claim()?,
        //     }),
        //     journal: receipt.journal,
        // };
        // receipt.verify(image_id)?;

        let seal = Seal::abi_encode(snark).context("Read seal")?;
        let post_state_digest: FixedBytes<32> = snark_receipt
            .post_state_digest
            .as_slice()
            .try_into()
            .context("Read post_state_digest")?;
        let journal = snark_receipt.journal;

        Ok((journal, post_state_digest, seal))
    }
}
