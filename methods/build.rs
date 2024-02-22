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

use std::{collections::HashMap, env};

use risc0_build::{embed_methods_with_options, DockerOptions, GuestOptions};
use risc0_build_ethereum::generate_solidity_files;

const SOLIDITY_IMAGE_ID_PATH: &str = "../contracts/ImageID.sol";
const SOLIDITY_ELF_PATH: &str = "../tests/Elf.sol";

fn main() {
    let use_docker = env::var("RISC0_USE_DOCKER").ok().map(|_| DockerOptions {
        root_dir: Some("../".into()),
    });

    let guests = embed_methods_with_options(HashMap::from([(
        "bonsai-starter-methods-guest",
        GuestOptions {
            features: Vec::new(),
            use_docker,
        },
    )]));

    let solidity_opts = risc0_build_ethereum::Options::default()
        .with_image_id_sol_path(SOLIDITY_IMAGE_ID_PATH)
        .with_elf_sol_path(SOLIDITY_ELF_PATH);

    generate_solidity_files(guests.as_slice(), &solidity_opts).unwrap();
}
