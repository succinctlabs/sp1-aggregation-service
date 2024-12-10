use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tiny_keccak::{Hasher, Keccak};

pub struct MerkleProof {
    pub proof: Vec<[u8; 32]>,
}
pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    pub tree: Vec<[u8; 32]>,
    pub root: [u8; 32],
    leaf_indices: HashMap<[u8; 32], usize>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        // if leaves is empty, return empty tree
        if leaves.is_empty() {
            return Self {
                leaves: Vec::new(),
                tree: Vec::new(),
                root: [0; 32],
                leaf_indices: HashMap::new(),
            };
        }

        let mut tree = Vec::new();
        let mut current_layer = leaves.clone();
        let mut leaf_indices = HashMap::new();
        for (i, leaf) in leaves.iter().enumerate() {
            leaf_indices.insert(*leaf, i);
        }

        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            for chunk in current_layer.chunks(2) {
                if chunk.len() == 1 {
                    next_layer.push(chunk[0]); // Add the leaf on its own
                } else {
                    let left = chunk[0];
                    let right = chunk[1];
                    let hash: [u8; 32] = if left > right {
                        let mut hasher = Keccak::v256();
                        let mut output = [0u8; 32];
                        hasher.update(&[right, left].concat());
                        hasher.finalize(&mut output);
                        output
                    } else {
                        let mut hasher = Keccak::v256();
                        let mut output = [0u8; 32];
                        hasher.update(&[left, right].concat());
                        hasher.finalize(&mut output);
                        output
                    };
                    next_layer.push(hash);
                }
            }
            tree.extend(current_layer);
            current_layer = next_layer;
        }
        // Add the root to the tree
        tree.extend(current_layer);
        let root = *tree.last().unwrap();
        Self {
            leaves,
            tree,
            root,
            leaf_indices,
        }
    }

    pub fn generate_proof(&self, leaf: [u8; 32]) -> Option<Vec<[u8; 32]>> {
        // Find the index of the leaf in the leaves vector
        let index = match self.leaf_indices.get(&leaf) {
            Some(&idx) => idx,
            None => return None, // Return None if leaf is not found
        };

        // Generate the proof
        let mut proof = Vec::new();
        let mut current_index = index;
        let mut level_start = 0;
        let mut level_size = self.leaves.len();

        while level_size > 1 {
            let pair_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if pair_index < level_size {
                proof.push(self.tree[level_start + pair_index]);
            }

            current_index /= 2;
            level_start += level_size;
            level_size = (level_size + 1) / 2;
        }

        Some(proof)
    }

    pub fn verify_proof(&self, proof: Vec<[u8; 32]>, leaf: [u8; 32]) -> bool {
        let mut current_hash = leaf;
        for sibling in proof {
            let mut hasher = Keccak::v256();
            let mut output = [0u8; 32];
            if current_hash < sibling {
                hasher.update(&[current_hash, sibling].concat());
            } else {
                hasher.update(&[sibling, current_hash].concat());
            }
            hasher.finalize(&mut output);
            current_hash = output;
        }
        current_hash == self.root
    }
}
