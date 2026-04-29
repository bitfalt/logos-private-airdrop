use crate::{
    domain::{CLAIM_PUBKEY_DOMAIN, LEAF_DOMAIN, RECIPIENT_DOMAIN},
    hash::{Hash32, sha256_parts},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimSecret(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimPublicKey(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafSalt(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipientBinding(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EligibilityLeafInput {
    pub distribution_id: Hash32,
    pub claim_pubkey: ClaimPublicKey,
    pub allocation: u128,
    pub leaf_salt: LeafSalt,
    pub recipient_binding: RecipientBinding,
}

#[must_use]
pub fn derive_claim_pubkey(secret: &ClaimSecret) -> ClaimPublicKey {
    ClaimPublicKey(sha256_parts(&[CLAIM_PUBKEY_DOMAIN, &secret.0]))
}

#[must_use]
pub fn derive_recipient_binding(raw: &[u8]) -> RecipientBinding {
    RecipientBinding(sha256_parts(&[RECIPIENT_DOMAIN, raw]))
}

#[must_use]
pub fn compute_leaf(input: &EligibilityLeafInput) -> Hash32 {
    let allocation = input.allocation.to_be_bytes();
    sha256_parts(&[
        LEAF_DOMAIN,
        &input.distribution_id,
        &input.claim_pubkey.0,
        &allocation,
        &input.leaf_salt.0,
        &input.recipient_binding.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf_input() -> EligibilityLeafInput {
        EligibilityLeafInput {
            distribution_id: [7; 32],
            claim_pubkey: derive_claim_pubkey(&ClaimSecret([11; 32])),
            allocation: 100,
            leaf_salt: LeafSalt([13; 32]),
            recipient_binding: derive_recipient_binding(b"Public/demo-recipient"),
        }
    }

    #[test]
    fn derives_claim_pubkey_without_revealing_secret() {
        let secret = ClaimSecret([42; 32]);
        let pubkey = derive_claim_pubkey(&secret);

        assert_ne!(pubkey.0, secret.0);
        assert_eq!(pubkey, derive_claim_pubkey(&secret));
    }

    #[test]
    fn recipient_binding_is_deterministic() {
        assert_eq!(
            derive_recipient_binding(b"recipient"),
            derive_recipient_binding(b"recipient")
        );
        assert_ne!(
            derive_recipient_binding(b"recipient"),
            derive_recipient_binding(b"other")
        );
    }

    #[test]
    fn same_input_produces_same_leaf() {
        let input = leaf_input();
        assert_eq!(compute_leaf(&input), compute_leaf(&input));
    }

    #[test]
    fn changing_distribution_changes_leaf() {
        let baseline = leaf_input();
        let mut changed = leaf_input();
        changed.distribution_id = [8; 32];

        assert_ne!(compute_leaf(&baseline), compute_leaf(&changed));
    }

    #[test]
    fn changing_allocation_changes_leaf() {
        let baseline = leaf_input();
        let mut changed = leaf_input();
        changed.allocation = 101;

        assert_ne!(compute_leaf(&baseline), compute_leaf(&changed));
    }

    #[test]
    fn changing_salt_changes_leaf() {
        let baseline = leaf_input();
        let mut changed = leaf_input();
        changed.leaf_salt = LeafSalt([14; 32]);

        assert_ne!(compute_leaf(&baseline), compute_leaf(&changed));
    }
}
