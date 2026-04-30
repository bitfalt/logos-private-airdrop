use crate::{
    domain::NODE_DOMAIN,
    errors::AirdropCoreError,
    hash::{Hash32, sha256_parts},
};

pub const DEFAULT_TREE_DEPTH: usize = 20;

const EMPTY_LEAF: Hash32 = [0; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf: Hash32,
    pub index: u64,
    pub siblings: Vec<Hash32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleTree {
    leaves: Vec<Hash32>,
    layers: Vec<Vec<Hash32>>,
}

impl MerkleTree {
    pub fn from_leaves(leaves: Vec<Hash32>, depth: usize) -> Result<Self, AirdropCoreError> {
        if leaves.is_empty() {
            return Err(AirdropCoreError::EmptyMerkleTree);
        }

        let capacity = capacity_for_depth(depth)?;
        if leaves.len() > capacity {
            return Err(AirdropCoreError::TooManyMerkleLeaves {
                depth,
                capacity,
                actual: leaves.len(),
            });
        }

        let mut current = leaves.clone();
        current.resize(capacity, EMPTY_LEAF);

        let mut layers = Vec::with_capacity(depth + 1);
        layers.push(current.clone());

        while current.len() > 1 {
            current = current
                .chunks_exact(2)
                .map(|pair| node_hash(pair[0], pair[1]))
                .collect();
            layers.push(current.clone());
        }

        Ok(Self { leaves, layers })
    }

    #[must_use]
    pub fn root(&self) -> Hash32 {
        self.layers
            .last()
            .and_then(|layer| layer.first())
            .copied()
            .unwrap_or(EMPTY_LEAF)
    }

    pub fn proof(&self, index: usize) -> Result<MerkleProof, AirdropCoreError> {
        let leaf = self.leaves.get(index).copied().ok_or(
            AirdropCoreError::MerkleProofIndexOutOfBounds {
                index,
                leaf_count: self.leaves.len(),
            },
        )?;

        let mut siblings = Vec::with_capacity(self.layers.len().saturating_sub(1));
        let mut node_index = index;
        for layer in self.layers.iter().take(self.layers.len().saturating_sub(1)) {
            let sibling_index = if node_index % 2 == 0 {
                node_index + 1
            } else {
                node_index - 1
            };
            siblings.push(layer[sibling_index]);
            node_index /= 2;
        }

        Ok(MerkleProof {
            leaf,
            index: index as u64,
            siblings,
        })
    }
}

#[must_use]
pub fn verify_merkle_proof(root: Hash32, proof: &MerkleProof) -> bool {
    let mut index = proof.index;
    let mut current = proof.leaf;

    for sibling in &proof.siblings {
        current = if index % 2 == 0 {
            node_hash(current, *sibling)
        } else {
            node_hash(*sibling, current)
        };
        index /= 2;
    }

    current == root
}

fn capacity_for_depth(depth: usize) -> Result<usize, AirdropCoreError> {
    1usize
        .checked_shl(depth as u32)
        .ok_or(AirdropCoreError::MerkleDepthTooLarge(depth))
}

fn node_hash(left: Hash32, right: Hash32) -> Hash32 {
    sha256_parts(&[NODE_DOMAIN, &left, &right])
}

#[cfg(test)]
mod tests {
    use crate::hash::Hash32;

    use super::*;

    fn leaf(byte: u8) -> Hash32 {
        [byte; 32]
    }

    #[test]
    fn empty_leaves_are_rejected() {
        let err = MerkleTree::from_leaves(vec![], 2).unwrap_err();
        assert_eq!(err.to_string(), "merkle tree requires at least one leaf");
    }

    #[test]
    fn too_many_leaves_are_rejected() {
        let err = MerkleTree::from_leaves(vec![leaf(1), leaf(2), leaf(3)], 1).unwrap_err();
        assert_eq!(
            err.to_string(),
            "merkle tree depth 1 supports 2 leaves, got 3"
        );
    }

    #[test]
    fn single_leaf_tree_proof_verifies() {
        let tree = MerkleTree::from_leaves(vec![leaf(1)], 0).unwrap();
        let proof = tree.proof(0).unwrap();

        assert_eq!(proof.leaf, leaf(1));
        assert_eq!(proof.index, 0);
        assert!(proof.siblings.is_empty());
        assert!(verify_merkle_proof(tree.root(), &proof));
    }

    #[test]
    fn multi_leaf_proofs_verify_for_every_leaf() {
        let leaves = vec![leaf(1), leaf(2), leaf(3), leaf(4)];
        let tree = MerkleTree::from_leaves(leaves.clone(), 2).unwrap();

        for (index, expected_leaf) in leaves.into_iter().enumerate() {
            let proof = tree.proof(index).unwrap();
            assert_eq!(proof.leaf, expected_leaf);
            assert_eq!(proof.index, index as u64);
            assert_eq!(proof.siblings.len(), 2);
            assert!(verify_merkle_proof(tree.root(), &proof));
        }
    }

    #[test]
    fn tampered_sibling_fails() {
        let tree = MerkleTree::from_leaves(vec![leaf(1), leaf(2)], 1).unwrap();
        let mut proof = tree.proof(0).unwrap();
        proof.siblings[0] = leaf(9);

        assert!(!verify_merkle_proof(tree.root(), &proof));
    }

    #[test]
    fn tampered_index_fails() {
        let tree = MerkleTree::from_leaves(vec![leaf(1), leaf(2)], 1).unwrap();
        let mut proof = tree.proof(0).unwrap();
        proof.index = 1;

        assert!(!verify_merkle_proof(tree.root(), &proof));
    }

    #[test]
    fn tampered_root_fails() {
        let tree = MerkleTree::from_leaves(vec![leaf(1), leaf(2)], 1).unwrap();
        let proof = tree.proof(0).unwrap();

        assert!(!verify_merkle_proof(leaf(9), &proof));
    }

    #[test]
    fn deterministic_root_for_committed_fixture() {
        let tree = MerkleTree::from_leaves(vec![leaf(1), leaf(2), leaf(3)], 3).unwrap();
        assert_eq!(
            tree.root(),
            [
                172, 188, 44, 166, 94, 72, 22, 179, 136, 188, 49, 113, 245, 76, 135, 41, 233, 123,
                212, 1, 211, 92, 207, 81, 109, 66, 100, 154, 137, 254, 73, 211,
            ]
        );
    }
}
