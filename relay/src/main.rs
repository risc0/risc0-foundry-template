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

use anyhow::{Context, Error, Result};
use bonsai_ethereum_relay::{resolve_guest_entry, resolve_image_output};
use bonsai_sdk_alpha::alpha::{Client, SdkErr};
use bonsai_starter_methods::GUEST_LIST;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Command {
    /// Runs the RISC-V ELF binary.
    Query {
        /// The name of the guest binary
        guest_binary: String,

        /// The input to provide to the guest binary
        input: Option<String>,
    },
    /// Upload the RISC-V ELF binary to Bonsai.
    Upload {
        /// The name of the guest binary
        #[arg(long)]
        guest_binary: String,
        /// Bonsai API URL
        #[arg(long, env)]
        bonsai_api_url: String,
        /// Bonsai API URL
        #[arg(long, env)]
        bonsai_api_key: String,
    },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    match args.command {
        Command::Query {
            guest_binary,
            input,
        } => {
            // Search list for requested binary name
            let guest_entry = resolve_guest_entry(GUEST_LIST, &guest_binary)
                .context("failed to resolve guest entry")?;

            // Execute or return image id
            let output_bytes = match &input {
                Some(input) => resolve_image_output(input, guest_entry).await,
                None => Ok(Vec::from(bytemuck::cast::<[u32; 8], [u8; 32]>(
                    guest_entry.image_id,
                ))),
            }
            .context("failed to compute output")?;

            let output = hex::encode(output_bytes);
            print!("{output}");
            std::io::stdout()
                .flush()
                .context("Failed to flush stdout buffer")?;
        }
        Command::Upload {
            guest_binary,
            bonsai_api_url,
            bonsai_api_key,
        } => {
            // Search list for requested binary name
            let guest_entry = resolve_guest_entry(GUEST_LIST, &guest_binary)
                .context("failed to resolve guest entry")?;
            let image_id = hex::encode(Vec::from(bytemuck::cast::<[u32; 8], [u8; 32]>(
                guest_entry.image_id,
            )));
            let bonsai_client = tokio::task::spawn_blocking(move || {
                Client::from_parts(bonsai_api_url, bonsai_api_key)
            })
            .await
            .context("could not initialize a Bonsai client")??;
            let img_id = image_id.clone();
            match tokio::task::spawn_blocking(move || {
                bonsai_client.upload_img(&image_id, guest_entry.elf.to_vec())
            })
            .await?
            {
                Ok(()) => (),
                Err(SdkErr::ImageIdExists) => (),
                Err(err) => return Err(err.into()),
            }

            println!("Uploaded image id: {}", img_id);
            std::io::stdout()
                .flush()
                .context("Failed to flush stdout buffer")?;
        }
    }
    Ok(())
}
