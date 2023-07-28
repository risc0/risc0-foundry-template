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

use std::{io::Write, sync::Arc};

use anyhow::{Context, Error, Result};
use bonsai_ethereum_relay::Relayer;
use bonsai_ethereum_relay_cli::{resolve_guest_entry, resolve_image_output};
use bonsai_sdk_alpha::{
    alpha::SdkErr,
    alpha_async::{get_client_from_parts, put_image},
};
use bonsai_starter_methods::GUEST_LIST;
use clap::{Parser, Subcommand};
use ethers::{
    core::k256::{ecdsa::SigningKey, SecretKey},
    prelude::*,
    types::Address,
};

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
        guest_binary: String,
        /// Bonsai API URL
        bonsai_api_url: String,
        /// Bonsai API URL
        bonsai_api_key: String,
    },
    /// Upload the RISC-V ELF binary to Bonsai.
    Run {
        /// Bonsai API URL
        /// Defaults to localhost
        #[arg(long, env)]
        bonsai_api_url: Option<String>,
        /// Bonsai API URL
        /// Defaults to empty
        #[arg(long, env)]
        bonsai_api_key: Option<String>,
        /// Bonsai Relay contract address on Ethereum
        #[arg(long, env)]
        relay_address: Address,
        /// Ethereum Node endpoint.
        /// Defaults to Anvil
        #[arg(long, env)]
        eth_node: Option<String>,
        /// Ethereum chain ID
        #[arg(long, default_value_t = 31337)]
        eth_chain_id: u64,
        /// Wallet Key Identifier. Can be a private key as a hex string, or an
        /// AWS KMS key identifier.
        /// Defaults to
        /// ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
        #[arg(short, long, env)]
        private_key: Option<String>,
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

            // upload binary to Bonsai
            let bonsai_client = get_client_from_parts(bonsai_api_url, bonsai_api_key).await?;
            let img_id = image_id.clone();
            match put_image(
                bonsai_client.clone(),
                img_id.clone(),
                guest_entry.elf.to_vec(),
            )
            .await
            {
                Ok(()) => (),
                Err(SdkErr::ImageIdExists) => (),
                Err(err) => return Err(err.into()),
            }

            print!("{image_id}");
            std::io::stdout()
                .flush()
                .context("Failed to flush stdout buffer")?;
        }
        Command::Run {
            bonsai_api_url,
            bonsai_api_key,
            relay_address,
            eth_node,
            eth_chain_id,
            private_key,
        } => {
            let ethers_client = create_ethers_client_private_key(
                &eth_node.unwrap_or_else(|| "ws://localhost:8545".to_string()),
                &private_key.unwrap_or_else(|| {
                    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()
                }),
                eth_chain_id,
            )
            .await?;

            let relayer = Relayer {
                publish_mode: true,
                publish_port: "8080".to_string(),
                bonsai_api_url: bonsai_api_url
                    .unwrap_or_else(|| "http://localhost:8081".to_string()),
                bonsai_api_key: bonsai_api_key.unwrap_or_else(|| "".to_string()),
                relay_contract_address: relay_address,
            };
            let _ = tokio::spawn(relayer.run(ethers_client.clone())).await;
        }
    }
    Ok(())
}

async fn create_ethers_client_private_key(
    eth_node: &str,
    private_key: &str,
    eth_chain_id: u64,
) -> Result<Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>> {
    let web3_provider = Provider::<Ws>::connect(eth_node)
        .await
        .context("unable to connect to websocket")?;
    let web3_wallet_sk_bytes =
        hex::decode(private_key).context("wallet_key_identifier should be valid hex string")?;
    let web3_wallet_secret_key =
        SecretKey::from_slice(&web3_wallet_sk_bytes).context("invalid private key")?;
    let web3_wallet_signing_key = SigningKey::from(web3_wallet_secret_key);
    let web3_wallet = LocalWallet::from(web3_wallet_signing_key);
    Ok(Arc::new(SignerMiddleware::new(
        web3_provider,
        web3_wallet.with_chain_id(eth_chain_id),
    )))
}
