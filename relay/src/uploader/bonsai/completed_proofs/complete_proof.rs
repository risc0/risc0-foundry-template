use bonsai_proxy_contract::{CallbackRequestFilter, EthereumCallback};
use bonsai_sdk::{client::Client, types::ProofID};
use ethers::{contract::abigen, types::H256};

use crate::uploader::bonsai::completed_proofs::error::CompleteProofError;

abigen!(
    BonsaiContract,
    r#"[
        function latest_block_height() public returns (uint256)
    ]"#,
);

#[derive(Debug, Clone)]
pub struct CompleteProof {
    pub bonsai_proof_id: ProofID,
    pub ethereum_callback: EthereumCallback,
    pub block_height: u128,
}

pub async fn get_complete_proof(
    client: Client,
    bonsai_proof_id: ProofID,
    callback_request: CallbackRequestFilter,
) -> Result<CompleteProof, CompleteProofError> {
    let receipt = client.get_receipt(bonsai_proof_id).await.or_else(|_| {
        Err(CompleteProofError::ReceiptNotFound {
            id: bonsai_proof_id,
        })
    })?;

    // Note: right now there are no blocks or proofs
    let journal_inclusion_proof: Vec<H256> = Vec::new();

    // TODO(joby): no blocks right now
    let block_height = 0;

    let gas_limit = callback_request.gas_limit.clone();

    Ok(CompleteProof {
        bonsai_proof_id,
        ethereum_callback: EthereumCallback {
            journal_inclusion_proof,
            journal: receipt.journal,
            callback_request,
            gas_limit,
        },
        block_height,
    })
}
