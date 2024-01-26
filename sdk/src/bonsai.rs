use std::time::Duration;

use alloy_primitives::FixedBytes;
use anyhow::{Context, Result};
use bonsai_sdk::alpha as bonsai_sdk;
use risc0_zkvm::{compute_image_id, serde::to_vec, Receipt};

use crate::seal;

pub fn prove<T: serde::Serialize + Sized>(
    elf: &[u8],
    input: T,
) -> Result<(Vec<u8>, FixedBytes<32>, Vec<u8>)> {
    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;

    // Compute the image_id, then upload the ELF with the image_id as its key.
    let image_id = compute_image_id(elf)?.to_string();
    client.upload_img(&image_id, elf.to_vec())?;
    eprintln!("Image ID: 0x{}", image_id);

    // Prepare input data and upload it.
    let input_id = {
        let input_data = to_vec(&input).unwrap();
        let input_data = bytemuck::cast_slice(&input_data).to_vec();
        client.upload_input(input_data)?
    };

    // Start a session running the prover
    let session = client.create_session(image_id, input_id, vec![])?;
    eprintln!("Created session: {}", session.uuid);
    let _receipt = loop {
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

    // Verify receipt
    {
        // TODO
    }

    eprintln!("Snark proof!: {snark_receipt:?}");

    let seal = seal::Seal::abi_encode(snark_receipt.snark).context("Read seal")?;
    let post_state_digest: FixedBytes<32> = snark_receipt
        .post_state_digest
        .as_slice()
        .try_into()
        .context("Read post_state_digest")?;
    let journal = snark_receipt.journal;

    Ok((seal, post_state_digest, journal))
}
