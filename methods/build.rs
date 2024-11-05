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

use std::{collections::HashMap, env, process::Command};

use risc0_build::{embed_methods_with_options, DockerOptions, GuestOptions};
use risc0_build_ethereum::generate_solidity_files;

// Paths where the generated Solidity files will be written.
const SOLIDITY_IMAGE_ID_PATH: &str = "../contracts/ImageID.sol";
const SOLIDITY_ELF_PATH: &str = "../tests/Elf.sol";

fn main() {
    git_submodule_init();
    check_submodule_state();
    println!("cargo:rerun-if-changed=.gitmodules");

    // Builds can be made deterministic, and thereby reproducible, by using Docker to build the
    // guest. Check the RISC0_USE_DOCKER variable and use Docker to build the guest if set.
    println!("cargo:rerun-if-env-changed=RISC0_USE_DOCKER");
    let use_docker = env::var("RISC0_USE_DOCKER").ok().map(|_| DockerOptions {
        root_dir: Some("../".into()),
    });

    // Generate Rust source files for the methods crate.
    let guests = embed_methods_with_options(HashMap::from([(
        "guests",
        GuestOptions {
            features: Vec::new(),
            use_docker,
        },
    )]));

    // Generate Solidity source files for use with Forge.
    let solidity_opts = risc0_build_ethereum::Options::default()
        .with_image_id_sol_path(SOLIDITY_IMAGE_ID_PATH)
        .with_elf_sol_path(SOLIDITY_ELF_PATH);

    generate_solidity_files(guests.as_slice(), &solidity_opts).unwrap();
}

/// Initializes git submodules by adding their configurations to .git/config.
/// This is a one-time setup step that only needs to run on first clone of the repository.
/// Does not fetch or update submodule contents.
///
/// # Warnings
/// Prints a warning to stderr if the initialization fails, but does not interrupt the build process.
fn git_submodule_init() {
    let output = Command::new("git")
        .args(["submodule", "init"])
        .output()
        .expect("failed to run git submodule init in methods/build.rs");

    if !output.status.success() {
        eprintln!(
            "WARNING: git submodule init failed (methods/build.rs): {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Checks and reports the status of all git submodules in the project.
/// Runs on every build to inform developers about the state of their submodules.
///
/// # Status Indicators
/// - `-`: submodule is not initialized
/// - `+`: submodule has local changes
/// - ` `: submodule is clean (no warning displayed)
///
/// # Warnings
/// Prints warnings for any non-clean states, but does not modify submodules
/// or interrupt the build process.
fn check_submodule_state() {
    let status = Command::new("git")
        .args(["submodule", "status"])
        .output()
        .expect("failed to run git submodule status");

    if !status.status.success() {
        println!(
            "cargo:warning=failed to check git submodule status: {}",
            String::from_utf8_lossy(&status.stderr)
        );
        return;
    }

    let output = String::from_utf8_lossy(&status.stdout);
    let mut has_uninitialized = false;
    let mut has_local_changes = false;

    for line in output.lines() {
        let path = line
            .split_whitespace()
            .nth(1)
            .unwrap_or("unknown path")
            .replace("../", "");

        if let Some(first_char) = line.chars().next() {
            match first_char {
                '-' => {
                    println!("cargo:warning=git submodule not initialized: {}", path);
                    has_uninitialized = true;
                }
                '+' => {
                    println!("cargo:warning=git submodule has local changes, this may cause unexpected behaviour: {}", path);
                    has_local_changes = true;
                }
                _ => (),
            }
        }
    }

    if has_uninitialized {
        println!(
            "cargo:warning=to initialize missing submodules, run: git submodule update --init"
        );
    }

    if has_local_changes {
        println!("cargo:warning=to reset submodules to their expected versions, run: git submodule update --recursive");
    }
}
