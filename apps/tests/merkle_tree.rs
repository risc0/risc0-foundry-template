use alloy::hex::FromHex;
use alloy_merkle_tree::incremental_tree::IncrementalMerkleTree;
use alloy_primitives::B256;
use sha2::{Digest, Sha256};

type MerkleTree = IncrementalMerkleTree<10, Sha256>;

#[test]
fn merkle_tree_insertion() {
    let mut tree = MerkleTree::new();

    println!("root 0: {:?}", tree.root());
    assert_eq!(
        tree.root(),
        B256::from_hex("0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b")
            .unwrap()
    );

    let leaf = [0x00; 32];
    tree.append(leaf.into()).unwrap();

    println!("root 1: {:?}", tree.root());
    assert_eq!(
        tree.root(),
        B256::from_hex("0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b")
            .unwrap()
    );

    let leaf = [0xff; 32];
    tree.append(leaf.into()).unwrap();

    println!("root 2: {:?}", tree.root());
    assert_eq!(
        tree.root(),
        B256::from_hex("0x606f20333f7003fc7839a11ddbf8a3d85d3f35c5b993889dde79aa2caf13d61d")
            .unwrap()
    );
}
