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

use anyhow::{Error, Result};
use bonsai_sdk::alpha::SdkErr;
use bonsai_starter_methods::FIBONACCI_ELF;
use clap::{Parser, Subcommand};
use risc0_zkvm::{MemoryImage, Program, MEM_SIZE, PAGE_SIZE};

#[derive(Subcommand)]
pub enum Command {
    /// Upload the RISC-V ELF binary to Bonsai.
    Upload {
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

pub fn get_digest(elf: &[u8]) -> Result<String> {
    let program = Program::load_elf(elf, MEM_SIZE as u32)?;
    let image = MemoryImage::new(&program, PAGE_SIZE as u32)?;
    Ok(hex::encode(image.compute_id()))
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    match args.command {
        Command::Upload {
            bonsai_api_url,
            bonsai_api_key,
        } => {
            let image_id = get_digest(FIBONACCI_ELF).expect("could not load image id");
            let bonsai_client =
                bonsai_sdk::alpha::Client::from_parts(bonsai_api_url, bonsai_api_key)
                    .expect("could not initialize a Bonsai client");
            match bonsai_client.upload_img(&image_id, FIBONACCI_ELF.to_vec()) {
                Ok(()) => (),
                Err(SdkErr::ImageIdExists) => (),
                Err(err) => return Err(err.into()),
            }

            println!("Uploaded image id: {}", image_id);
            std::io::stdout()
                .flush()
                .expect("Failed to flush stdout buffer");
        }
    }
    Ok(())
}
