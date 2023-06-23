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

use std::{io, io::Write};

use anyhow::{Context, Result};
use bonsai_starter_methods::{resolve_guest_entry, resolve_image_output, GUEST_LIST};
use clap::Parser;

/// Runs the RISC-V ELF binary.
#[derive(Parser)]
#[clap(about, version, author)]
struct Args {
    /// The name of the guest binary
    guest_binary: String,

    /// The input to provide to the guest binary
    input: Option<String>,
}

fn main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();
    // Search list for requested binary name
    let guest_entry = resolve_guest_entry(GUEST_LIST, &args.guest_binary)?;

    // Execute or return image id
    let output_bytes = match &args.input {
        Some(input) => resolve_image_output(input, guest_entry),
        None => Ok(Vec::from(bytemuck::cast::<[u32; 8], [u8; 32]>(
            guest_entry.image_id,
        ))),
    }?;

    let output = hex::encode(output_bytes);
    print!("{output}");
    io::stdout()
        .flush()
        .context("Failed to flush stdout buffer")?;
    Ok(())
}
