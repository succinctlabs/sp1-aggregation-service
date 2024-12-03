//! A simple program that aggregates the proofs of multiple programs proven with the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);

// use alloy_merkle_tree::tree::MerkleTree;
use sha2::Digest;
use sha2::Sha256;

pub fn words_to_bytes_le(words: &[u32; 8]) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for i in 0..8 {
        let word_bytes = words[i].to_le_bytes();
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&word_bytes);
    }
    bytes
}

/// Encode a list of vkeys and public values into a merkle tree. Returns the root of the tree.
pub fn create_merkle_tree(vkeys: &[[u32; 8]], public_values: &[Vec<u8>]) -> Vec<u8> {
    assert_eq!(vkeys.len(), public_values.len());
    let mut leaves = Vec::new(); // Initialize the Merkle tree
    for (vkey, public_value) in vkeys.iter().zip(public_values.iter()) {
        let vkey_bytes = words_to_bytes_le(vkey);
        let pair = [public_value.as_slice(), &vkey_bytes].concat();
        let pair_hash = Sha256::digest(&pair);
        leaves.push(pair_hash.to_vec());
    }
    let mut tree = Vec::new();
    let mut current_layer = leaves.clone();
    while current_layer.len() > 1 {
        let mut next_layer = Vec::new();
        for chunk in current_layer.chunks(2) {
            if chunk.len() == 2 {
                next_layer.push(Sha256::digest(chunk.concat()).to_vec());
            } else {
                next_layer.push(Sha256::digest(chunk[0].clone()).to_vec());
            }
        }
        tree.extend(current_layer);
        current_layer = next_layer;
    }
    let root = tree.clone().last().unwrap().clone();
    root
}

pub fn main() {
    // Read the verification keys.
    let vkeys = sp1_zkvm::io::read::<Vec<[u32; 8]>>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<Vec<u8>>>();

    // Verify the proofs.
    assert_eq!(vkeys.len(), public_values.len());
    for i in 0..vkeys.len() {
        let vkey = &vkeys[i];
        let public_values = &public_values[i];
        let public_values_digest = Sha256::digest(public_values);
        sp1_zkvm::lib::verify::verify_sp1_proof(vkey, &public_values_digest.into());
        // TODO: maybe have an array of bools to indicate whether verification succeeded or not and include this in the merkle tree?
    }

    // Commit to the verified proofs in a merkle tree.
    let commitment = create_merkle_tree(&vkeys, &public_values);
    sp1_zkvm::io::commit_slice(&commitment);
}
