use crate::abi::ITornado::ITornadoInstance;

use alloy::{
    network::Network,
    primitives::{B256, U256},
    providers::Provider,
};
use anyhow::Result;
use num_bigint::RandBigInt;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

pub(crate) async fn deposit<T, P, N>(
    contract: &ITornadoInstance<T, P, N>,
    note_size: U256,
) -> Result<()>
where
    T: alloy::transports::Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    // generate the nullifier (k) and secret (r) that make up the spending key
    let mut rng = OsRng;
    let note_spending_key = rng.gen_biguint(512).to_bytes_be(); // this is comprised of two 256 bit values (k, r)

    // hash them to produce the public commitment
    let commitment = {
        let mut hasher = Sha256::new();
        hasher.update(&note_spending_key);
        hasher.finalize()
    };

    // submit this to the contract along with the Eth to deposit
    // this will error if the caller has insufficient eth
    let call_builder = contract
        .deposit(B256::from_slice(&commitment))
        .value(note_size);
    let pending_tx = call_builder.send().await?;
    pending_tx.get_receipt().await?;

    println!(
        "Deposit successful. Spending key:({})",
        hex::encode(note_spending_key)
    );

    Ok(())
}
