use alloy::{
    network::Network,
    primitives::{Address, B256},
    providers::Provider,
    rpc::types::Filter,
};
use alloy_merkle_tree::incremental_tree::IncrementalMerkleTree;
use alloy_sol_types::SolEvent;
use anyhow::Result;
use methods::CAN_SPEND_ELF;
use mvm_core::ProofInput;
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, VerifierContext};
use sha2::{Digest, Sha256};

use crate::abi::Deposit;
use crate::abi::IMixer::IMixerInstance;

type MerkleTree = IncrementalMerkleTree<10, Sha256>;

pub(crate) async fn withdraw<T, P, N>(
    contract: &IMixerInstance<T, P, N>,
    contract_deploy_height: u64,
    recipient: Address,
    note_spending_key: [u8; 64],
) -> Result<()>
where
    T: alloy::transports::Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    let commitment = {
        let mut hasher = Sha256::new();
        hasher.update(&note_spending_key);
        hasher.finalize()
    };

    // extract nullifier and randomness
    let (k, r) = note_spending_key.split_at(32);

    // reconstruct the commitment tree and use this to generate the opening proof (merkle path)
    let (mut tree, index) =
        fetch_tree_and_commitment_position(contract, contract_deploy_height, commitment.into())
            .await?;

    tracing::info!("reconstructed commitment tree with root {}", tree.root());

    let index = index.ok_or_else(|| {
        anyhow::anyhow!("commitment not found in tree, cannot build a spending proof")
    })?;

    tracing::info!("note commitment found at index {}", index);

    let opening: Vec<_> = tree
        .proof_at_index(index.try_into()?)
        .map_err(|_| anyhow::anyhow!("failed to generate proof"))?
        .into_iter()
        .collect();

    let proof_input = ProofInput {
        root: tree.root(),
        k: k.try_into()?,
        r: r.try_into()?,
        leaf_index: index,
        opening,
        recipient,
    };

    tracing::info!("Building spend proof...");

    let env = ExecutorEnv::builder()
        .write_slice(&proof_input.to_bytes()?)
        .build()?;

    // Need to block in place here as the default_prover uses the bonsai blocking API
    // by default which won't work in an async context.
    // Should add a feature flag to risc0_zkvm to enable the non_blocking version
    let receipt = tokio::task::block_in_place(|| {
        default_prover().prove_with_ctx(
            env,
            &VerifierContext::default(),
            CAN_SPEND_ELF,
            &ProverOpts::groth16(),
        )
    })?
    .receipt;

    // Encode the seal with the selector.
    let seal = encode_seal(&receipt)?;

    // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();

    // use the root and nullifier hash from the journal to ensure they match the values we expect
    let root = B256::from_slice(&journal[0..32]);
    let nullifier_hash = B256::from_slice(&journal[32..64]);

    tracing::info!("Submitting withdrawal request to contract");

    // Submit the withdrawal request including the seal
    let call_builder = contract.withdraw(seal.into(), root, nullifier_hash);
    let pending_tx = call_builder.send().await?;
    pending_tx.get_receipt().await?;

    println!("💸 Withdrawal successful 💸");

    Ok(())
}

/// Parse the deposit logs in the contract to reconstruct the commitment Merkle tree locally
/// Also return the index of the given spending commitment in the tree if it is found
pub(crate) async fn fetch_tree_and_commitment_position<T, P, N>(
    contract: &IMixerInstance<T, P, N>,
    contract_deploy_height: u64,
    spending_commitment: [u8; 32],
) -> Result<(MerkleTree, Option<u32>)>
where
    T: alloy::transports::Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    // log filter for deposit events
    let filter = Filter::new()
        .address(*contract.address())
        .event_signature(Deposit::SIGNATURE_HASH)
        .from_block(contract_deploy_height);

    let logs = contract.provider().get_logs(&filter).await?;

    let mut commitment_tree = MerkleTree::new();
    let mut spending_commitment_index = None;

    for log in logs {
        let log = Deposit::decode_log(&log.inner, true)?;
        commitment_tree
            .append(log.commitment)
            .map_err(|_| anyhow::anyhow!("failed to append to tree"))?;
        if log.commitment == spending_commitment {
            spending_commitment_index = Some(log.leafIndex);
        }
    }
    Ok((commitment_tree, spending_commitment_index))
}
