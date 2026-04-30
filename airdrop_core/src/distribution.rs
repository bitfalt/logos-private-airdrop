use crate::{
    eligibility::{ClaimPublicKey, EligibilityLeafInput, LeafSalt, RecipientBinding, compute_leaf},
    errors::AirdropCoreError,
    hash::Hash32,
    merkle::MerkleTree,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationPolicy {
    Equal,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub claim_pubkey: Hash32,
    pub allocation: u128,
    pub leaf_salt: Hash32,
    pub recipient_binding: Hash32,
}

impl ManifestEntry {
    #[must_use]
    pub fn new(
        claim_pubkey: Hash32,
        allocation: u128,
        leaf_salt: Hash32,
        recipient_binding: Hash32,
    ) -> Self {
        Self {
            claim_pubkey,
            allocation,
            leaf_salt,
            recipient_binding,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionManifest {
    name: String,
    distribution_id: Hash32,
    policy: AllocationPolicy,
    entries: Vec<ManifestEntry>,
    leaves: Vec<Hash32>,
    tree: MerkleTree,
}

impl DistributionManifest {
    pub fn new_variable(
        name: String,
        distribution_id: Hash32,
        entries: Vec<ManifestEntry>,
        tree_depth: usize,
    ) -> Result<Self, AirdropCoreError> {
        Self::new(
            name,
            distribution_id,
            AllocationPolicy::Variable,
            entries,
            tree_depth,
        )
    }

    pub fn new_equal(
        name: String,
        distribution_id: Hash32,
        entries: Vec<ManifestEntry>,
        tree_depth: usize,
    ) -> Result<Self, AirdropCoreError> {
        if let Some(first) = entries.first() {
            if entries
                .iter()
                .any(|entry| entry.allocation != first.allocation)
            {
                return Err(AirdropCoreError::EqualAllocationMixed);
            }
        }

        Self::new(
            name,
            distribution_id,
            AllocationPolicy::Equal,
            entries,
            tree_depth,
        )
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn distribution_id(&self) -> Hash32 {
        self.distribution_id
    }

    #[must_use]
    pub fn policy(&self) -> AllocationPolicy {
        self.policy
    }

    #[must_use]
    pub fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }

    #[must_use]
    pub fn leaves(&self) -> &[Hash32] {
        &self.leaves
    }

    #[must_use]
    pub fn tree(&self) -> &MerkleTree {
        &self.tree
    }

    fn new(
        name: String,
        distribution_id: Hash32,
        policy: AllocationPolicy,
        entries: Vec<ManifestEntry>,
        tree_depth: usize,
    ) -> Result<Self, AirdropCoreError> {
        let leaves = entries
            .iter()
            .map(|entry| {
                compute_leaf(&EligibilityLeafInput {
                    distribution_id,
                    claim_pubkey: ClaimPublicKey(entry.claim_pubkey),
                    allocation: entry.allocation,
                    leaf_salt: LeafSalt(entry.leaf_salt),
                    recipient_binding: RecipientBinding(entry.recipient_binding),
                })
            })
            .collect::<Vec<_>>();
        let tree = MerkleTree::from_leaves(leaves.clone(), tree_depth)?;

        Ok(Self {
            name,
            distribution_id,
            policy,
            entries,
            leaves,
            tree,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DistributionManifest, ManifestEntry};

    #[test]
    fn variable_manifest_builds_ordered_leaves_and_root() {
        let manifest = DistributionManifest::new_variable(
            "demo".to_string(),
            [4; 32],
            vec![
                ManifestEntry::new([11; 32], 100, [21; 32], [31; 32]),
                ManifestEntry::new([12; 32], 250, [22; 32], [32; 32]),
            ],
            2,
        )
        .unwrap();

        assert_eq!(manifest.entries().len(), 2);
        assert_ne!(
            manifest.entries()[0].allocation,
            manifest.entries()[1].allocation
        );
        assert_eq!(manifest.tree().proof(1).unwrap().index, 1);
    }

    #[test]
    fn equal_manifest_rejects_unequal_allocations() {
        let err = DistributionManifest::new_equal(
            "demo".to_string(),
            [4; 32],
            vec![
                ManifestEntry::new([11; 32], 100, [21; 32], [31; 32]),
                ManifestEntry::new([12; 32], 250, [22; 32], [32; 32]),
            ],
            2,
        )
        .unwrap_err();

        assert_eq!(
            err.to_string(),
            "equal allocation manifest contains mixed allocations"
        );
    }
}
