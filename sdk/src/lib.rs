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

use anyhow::{anyhow, Result};
use risc0_build::GuestListEntry;

pub mod ethers;
pub mod prover;
pub mod snark;

pub fn resolve_guest_entry<'a>(
    guest_list: &[GuestListEntry<'a>],
    guest_binary: &String,
) -> Result<Vec<u8>> {
    // Search list for requested binary name
    let potential_guest_image_id: [u8; 32] =
        match hex::decode(guest_binary.to_lowercase().trim_start_matches("0x")) {
            Ok(byte_vector) => byte_vector.try_into().unwrap_or([0u8; 32]),
            Err(_) => [0u8; 32],
        };
    let guest_entry = guest_list
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
        .cloned()?;
    Ok(guest_entry.elf.to_vec())
}
