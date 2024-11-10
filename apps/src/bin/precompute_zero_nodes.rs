use alloy_primitives::B256;
use sha2::{Digest, Sha256};

const HEIGHT: usize = 32;

// These are the zero nodes for the incremental merkle tree using a given hasher
// These can be precomputed and hard-coded in the contract to save gas
fn main() {
    let mut zero_hashes = [B256::default(); HEIGHT];
    let mut hash_buf = [0u8; 64];
    (1..HEIGHT).for_each(|height| {
        hash_buf[..32].copy_from_slice(zero_hashes[height - 1].as_slice());
        hash_buf[32..].copy_from_slice(zero_hashes[height - 1].as_slice());
        zero_hashes[height] = hash(hash_buf);
    });
    println!("{:?}", zero_hashes);
}

fn hash(data: [u8; 64]) -> B256 {
    let mut hasher = Sha256::new();
    hasher.update(&data);
    B256::from_slice(&hasher.finalize())
}
