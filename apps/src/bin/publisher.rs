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

// This application demonstrates how to send an off-chain proof request
// to the Bonsai proving service and publish the received proofs directly
// to your deployed app contract.

use alloy_primitives::{hex, U256};
use alloy_sol_types::{sol, SolValue};
use anyhow::Result;
use clap::Parser;
use methods::IS_EVEN_ELF;
use risc0_ethereum_contracts::groth16;
use risc0_zkp::{core::digest::Digest, verify::VerificationError};
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv, ProverOpts, VerifierContext};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fs::File;
use std::io::Write;

sol! {
    struct Seal {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JsonVerificationKey {
    pub alpha: [String; 2],
    pub beta: [[String; 2]; 2],
    pub gamma: [[String; 2]; 2],
    pub delta: [[String; 2]; 2],
    pub s: Vec<[String; 2]>,
    pub h1: Vec<[[String; 2]; 2]>,
    pub h2: Vec<[[String; 2]; 2]>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonGroth16Proof {
    pi_a: [String; 2],
    pi_b: [[String; 2]; 2],
    pi_c: [String; 2],
    m: Vec<String>,
    pok: Vec<String>,
}

#[derive(Serialize)]
struct JsonGroth16VkProofInputs {
    proof: JsonGroth16Proof,
    inputs: [String; 5],
    vk: JsonVerificationKey,
}

// Modification of `split_digest` function that doesn't convert to Fr
pub fn split_digest_to_hex(d: Digest) -> Result<(String, String)> {
    let big_endian: Vec<u8> = d.as_bytes().to_vec().iter().rev().cloned().collect();
    let middle = big_endian.len() / 2;
    let (b, a) = big_endian.split_at(middle);
    Ok((hex::encode(a), hex::encode(b)))
}

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The input to provide to the guest binary
    #[clap(short, long)]
    input: U256,
}

fn main() -> Result<()> {
    env_logger::init();
    // Parse CLI Arguments: The application starts by parsing command-line arguments provided by the user.
    let args = Args::parse();

    // ABI encode input: Before sending the proof request to the Bonsai proving service,
    // the input number is ABI-encoded to match the format expected by the guest code running in the zkVM.
    let input = args.input.abi_encode();

    let env = ExecutorEnv::builder().write_slice(&input).build()?;

    let ctx = &VerifierContext::default();

    let receipt = default_prover()
        .prove_with_ctx(env, ctx, IS_EVEN_ELF, &ProverOpts::groth16())?
        .receipt;

    receipt.verify_integrity_with_context(ctx).unwrap();

    // Extract the public inputs used in the above verification.
    let params = ctx
        .groth16_verifier_parameters
        .as_ref()
        .ok_or(VerificationError::VerifierParametersMissing)?;

    let (a0, a1) = split_digest_to_hex(params.control_root)
        .map_err(|_| VerificationError::ReceiptFormatError)?;
    let (c0, c1) = split_digest_to_hex(receipt.inner.claim().unwrap().digest())
        .map_err(|_| VerificationError::ReceiptFormatError)?;
    let mut id_bn554: Digest = params.bn254_control_id;
    id_bn554.as_mut_bytes().reverse();
    let id_bn254_hex = hex::encode(id_bn554);

    let public_inputs = [
        format!("0x{}", a0),
        format!("0x{}", a1),
        format!("0x{}", c0),
        format!("0x{}", c1),
        format!("0x{}", id_bn254_hex),
    ];

    // Encode the seal with the selector.
    let seal = groth16::encode(receipt.inner.groth16()?.seal.clone())?;
    let truncated_seal = &seal[4..];
    let decoded = Seal::abi_decode(truncated_seal, true).unwrap();
    let pi_a = [
        format!("0x{:x}", decoded.a[0]),
        format!("0x{:x}", decoded.a[1]),
    ];
    let pi_b = [
        [
            format!("0x{:x}", decoded.b[0][1]),
            format!("0x{:x}", decoded.b[0][0]),
        ],
        [
            format!("0x{:x}", decoded.b[1][1]),
            format!("0x{:x}", decoded.b[1][0]),
        ],
    ];
    let pi_c = [
        format!("0x{:x}", decoded.c[0]),
        format!("0x{:x}", decoded.c[1]),
    ];

    // Taken from the Groth16Verifier contract.
    // TODO: get verifying key from `params.verifying_key`
    let risc0_verifying_key = JsonVerificationKey {
        alpha: [
            "20491192805390485299153009773594534940189261866228447918068658471970481763042"
                .to_string(),
            "9383485363053290200918347156157836566562967994039712273449902621266178545958"
                .to_string(),
        ],
        beta: [
            [
                "6375614351688725206403948262868962793625744043794305715222011528459656738731"
                    .to_string(),
                "4252822878758300859123897981450591353533073413197771768651442665752259397132"
                    .to_string(),
            ],
            [
                "10505242626370262277552901082094356697409835680220590971873171140371331206856"
                    .to_string(),
                "21847035105528745403288232691147584728191162732299865338377159692350059136679"
                    .to_string(),
            ],
        ],
        gamma: [
            [
                "10857046999023057135944570762232829481370756359578518086990519993285655852781"
                    .to_string(),
                "11559732032986387107991004021392285783925812861821192530917403151452391805634"
                    .to_string(),
            ],
            [
                "8495653923123431417604973247489272438418190587263600148770280649306958101930"
                    .to_string(),
                "4082367875863433681332203403145435568316851327593401208105741076214120093531"
                    .to_string(),
            ],
        ],
        delta: [
            [
                "12043754404802191763554326994664886008979042643626290185762540825416902247219"
                    .to_string(),
                "1668323501672964604911431804142266013250380587483576094566949227275849579036"
                    .to_string(),
            ],
            [
                "13740680757317479711909903993315946540841369848973133181051452051592786724563"
                    .to_string(),
                "7710631539206257456743780535472368339139328733484942210876916214502466455394"
                    .to_string(),
            ],
        ],
        s: vec![
            [
                "8446592859352799428420270221449902464741693648963397251242447530457567083492"
                    .to_string(),
                "1064796367193003797175961162477173481551615790032213185848276823815288302804"
                    .to_string(),
            ],
            [
                "3179835575189816632597428042194253779818690147323192973511715175294048485951"
                    .to_string(),
                "20895841676865356752879376687052266198216014795822152491318012491767775979074"
                    .to_string(),
            ],
            [
                "5332723250224941161709478398807683311971555792614491788690328996478511465287"
                    .to_string(),
                "21199491073419440416471372042641226693637837098357067793586556692319371762571"
                    .to_string(),
            ],
            [
                "12457994489566736295787256452575216703923664299075106359829199968023158780583"
                    .to_string(),
                "19706766271952591897761291684837117091856807401404423804318744964752784280790"
                    .to_string(),
            ],
            [
                "19617808913178163826953378459323299110911217259216006187355745713323154132237"
                    .to_string(),
                "21663537384585072695701846972542344484111393047775983928357046779215877070466"
                    .to_string(),
            ],
            [
                "6834578911681792552110317589222010969491336870276623105249474534788043166867"
                    .to_string(),
                "15060583660288623605191393599883223885678013570733629274538391874953353488393"
                    .to_string(),
            ],
        ],
        h1: [].to_vec(),
        h2: [].to_vec(),
    };

    let groth16_proof = JsonGroth16Proof {
        pi_a,
        pi_b,
        pi_c,
        m: [].to_vec(),
        pok: [].to_vec(),
    };

    let vk_proof_inputs = JsonGroth16VkProofInputs {
        proof: groth16_proof,
        inputs: public_inputs,
        vk: risc0_verifying_key,
    };

    let json_data = to_string_pretty(&vk_proof_inputs)?;

    // Write the VK, proof, inputs to a file
    let mut file = File::create("proof.json")?;
    file.write_all(json_data.as_bytes())?;

    println!("Proof successfully written to proof.json");
    Ok(())
}
