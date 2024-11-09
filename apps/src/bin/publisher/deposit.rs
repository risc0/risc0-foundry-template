use crate::abi::ITornado::ITornadoInstance;

use alloy::{network::Network, providers::Provider};
use anyhow::Result;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

pub(crate) async fn deposit<T, P, N>(contract: &ITornadoInstance<T, P, N>) -> Result<()>
where
    T: alloy::transports::Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    // generate the random values that make up the spending key
    let mut rng = OsRng;
    let (k, r) = (rng.next_u64(), rng.next_u64());

    // hash them to produce the public commitment
    let mut hasher = Sha256::new();
    hasher.update(&k.to_be_bytes());
    hasher.update(&r.to_be_bytes());
    let commitment = hasher.finalize();

    // submit this to the contract along with the Eth to deposit

    let call_builder =
        contract.deposit(alloy_primitives::FixedBytes::<32>::from_slice(&commitment));

    // Send transaction: Finally, send the transaction to the Ethereum blockchain,
    // effectively calling the set function of the EvenNumber contract with the verified number and proof.
    let pending_tx = call_builder.send().await?;
    pending_tx.get_receipt().await?;

    Ok(())
}
