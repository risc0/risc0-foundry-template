use crate::abi::ITornado::ITornadoInstance;

use alloy::{network::Network, providers::Provider, signers::k256::U256};
use anyhow::Result;
use sha2::{Digest, Sha256};

pub(crate) async fn withdraw<T, P, N>(
    contract: &ITornadoInstance<T, P, N>,
    note_size: U256,
    note_spending_key: [u8; 512],
) -> Result<()>
where
    T: alloy::transports::Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    // nullifier and randomness
    let (k, r) = note_spending_key.split_at(256);

    let nullifier_hash = {
        let mut hasher = Sha256::new();
        hasher.update(&k);
        hasher.finalize()
    };

    // retrieve a recent tree root from the contract and use this to generate the opening proof (merkle path)

    Ok(())
}
