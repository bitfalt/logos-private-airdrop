use airdrop_core::{
    eligibility::{ClaimPublicKey, EligibilityLeafInput, LeafSalt, RecipientBinding, compute_leaf},
    hash::Hash32,
    merkle::{MerkleProof, MerkleTree, verify_merkle_proof},
};

use crate::{
    DistributionState,
    errors::{
        ERR_ALLOCATION_MISMATCH, ERR_DISTRIBUTION_PAUSED, ERR_INVALID_MERKLE_PROOF,
        ERR_NULLIFIER_ALREADY_CLAIMED,
    },
    state::NullifierState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimValidationInput {
    pub distribution: DistributionState,
    pub claim_pubkey: Hash32,
    pub allocation: u128,
    pub leaf_salt: Hash32,
    pub recipient_binding: Hash32,
    pub nullifier: Hash32,
    pub merkle_proof: MerkleProof,
    pub existing_nullifier: Option<NullifierState>,
    pub claimed_at_slot_or_time: u64,
}

impl ClaimValidationInput {
    #[must_use]
    pub fn valid_fixture() -> Self {
        let distribution_id = [2; 32];
        let claim_pubkey = [11; 32];
        let allocation = 100;
        let leaf_salt = [21; 32];
        let recipient_binding = [31; 32];
        let leaf = compute_leaf(&EligibilityLeafInput {
            distribution_id,
            claim_pubkey: ClaimPublicKey(claim_pubkey),
            allocation,
            leaf_salt: LeafSalt(leaf_salt),
            recipient_binding: RecipientBinding(recipient_binding),
        });
        let tree = MerkleTree::from_leaves(vec![leaf], 0).unwrap();

        Self {
            distribution: DistributionState {
                version: 1,
                authority: [1; 32],
                distribution_id,
                merkle_root: tree.root(),
                mode: crate::DistributionMode::Airdrop,
                allocation_policy: crate::AllocationPolicy::Variable,
                total_eligible: 1,
                total_claimed: 0,
                created_at_slot_or_time: 42,
                paused: false,
            },
            claim_pubkey,
            allocation,
            leaf_salt,
            recipient_binding,
            nullifier: [9; 32],
            merkle_proof: tree.proof(0).unwrap(),
            existing_nullifier: None,
            claimed_at_slot_or_time: 77,
        }
    }

    #[must_use]
    pub fn invalid_proof_fixture() -> Self {
        let mut input = Self::valid_fixture();
        input.merkle_proof.siblings.push([44; 32]);
        input
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimValidationResult {
    pub distribution: DistributionState,
    pub nullifier: NullifierState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimValidationError {
    DistributionPaused,
    InvalidMerkleProof,
    AllocationMismatch,
    NullifierAlreadyClaimed,
}

impl ClaimValidationError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::DistributionPaused => ERR_DISTRIBUTION_PAUSED,
            Self::InvalidMerkleProof => ERR_INVALID_MERKLE_PROOF,
            Self::AllocationMismatch => ERR_ALLOCATION_MISMATCH,
            Self::NullifierAlreadyClaimed => ERR_NULLIFIER_ALREADY_CLAIMED,
        }
    }
}

pub fn validate_claim(
    input: ClaimValidationInput,
) -> Result<ClaimValidationResult, ClaimValidationError> {
    if input.distribution.paused {
        return Err(ClaimValidationError::DistributionPaused);
    }

    if !verify_merkle_proof(input.distribution.merkle_root, &input.merkle_proof) {
        return Err(ClaimValidationError::InvalidMerkleProof);
    }

    let expected_leaf = compute_leaf(&EligibilityLeafInput {
        distribution_id: input.distribution.distribution_id,
        claim_pubkey: ClaimPublicKey(input.claim_pubkey),
        allocation: input.allocation,
        leaf_salt: LeafSalt(input.leaf_salt),
        recipient_binding: RecipientBinding(input.recipient_binding),
    });
    if input.merkle_proof.leaf != expected_leaf {
        return Err(ClaimValidationError::AllocationMismatch);
    }

    if input.existing_nullifier.is_some() {
        return Err(ClaimValidationError::NullifierAlreadyClaimed);
    }

    let mut distribution = input.distribution;
    distribution.total_claimed = distribution.total_claimed.saturating_add(1);
    let nullifier = NullifierState {
        version: 1,
        distribution_id: distribution.distribution_id,
        nullifier: input.nullifier,
        allocation: input.allocation,
        claimed_at_slot_or_time: input.claimed_at_slot_or_time,
    };

    Ok(ClaimValidationResult {
        distribution,
        nullifier,
    })
}

#[cfg(test)]
mod tests {
    use super::{ClaimValidationInput, validate_claim};

    #[test]
    fn valid_claim_increments_total_claimed_and_writes_nullifier() {
        let result = validate_claim(ClaimValidationInput::valid_fixture()).unwrap();
        assert_eq!(result.distribution.total_claimed, 1);
        assert_eq!(result.nullifier.version, 1);
    }

    #[test]
    fn invalid_merkle_proof_does_not_write_nullifier() {
        let err = validate_claim(ClaimValidationInput::invalid_proof_fixture()).unwrap_err();
        assert_eq!(err.code(), "LP0003_INVALID_MERKLE_PROOF");
    }
}
