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

use risc0_build::{embed_methods_with_options, DockerOptions, GuestOptions};
use std::{collections::HashMap, env, fs, process::Command};

const SOL_HEADER: &str = r#"// Copyright 2024 RISC Zero, Inc.
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
//
// SPDX-License-Identifier: Apache-2.0

// This file is automatically generated

"#;

const LIB_HEADER: &str = r#"pragma solidity ^0.8.20;

library ImageID {
"#;

const SOLIDITY_IMAGE_ID_PATH: &str = "../contracts/ImageID.sol";

fn main() {
    let docker_opts = DockerOptions { root_dir: None };

    let use_docker = if env::var("RISC0_USE_DOCKER").is_ok() {
        Some(docker_opts)
    } else {
        None
    };

    let methods = embed_methods_with_options(HashMap::from([(
        "bonsai-starter-methods-guest",
        GuestOptions {
            features: vec![],
            use_docker,
        },
    )]));

    let mut file_content = format!("{SOL_HEADER}{LIB_HEADER}\n");
    let mut image_ids = vec![];
    for method in methods {
        let name = method.name.clone().to_uppercase().replace('-', "_");
        let image_id = hex::encode(method.make_image_id());
        image_ids.push(format!(
            "bytes32 public constant {name}_ID = bytes32(0x{image_id});"
        ));
    }
    for image_id in image_ids {
        file_content.push_str(&image_id)
    }
    file_content.push_str("\n}");
    fs::write(SOLIDITY_IMAGE_ID_PATH, file_content).expect(&format!(
        "failed to save changes to {}",
        SOLIDITY_IMAGE_ID_PATH
    ));

    // Use forge fmt to format the file.
    Command::new("forge")
        .arg("fmt")
        .arg(SOLIDITY_IMAGE_ID_PATH)
        .status()
        .expect("failed to format {SOLIDITY_CONTROL_ID_PATH}");
}
