use crate::{
    domain::NULLIFIER_DOMAIN,
    eligibility::ClaimSecret,
    hash::{Hash32, sha256_parts},
};

#[must_use]
pub fn derive_nullifier(distribution_id: &Hash32, claim_secret: &ClaimSecret) -> Hash32 {
    sha256_parts(&[NULLIFIER_DOMAIN, distribution_id, &claim_secret.0])
}

#[cfg(test)]
mod tests {
    use crate::{
        eligibility::{
            ClaimSecret, EligibilityLeafInput, LeafSalt, compute_leaf, derive_claim_pubkey,
            derive_recipient_binding,
        },
        hash::Hash32,
    };

    use super::*;

    fn distribution(byte: u8) -> Hash32 {
        [byte; 32]
    }

    fn claim_secret(byte: u8) -> ClaimSecret {
        ClaimSecret([byte; 32])
    }

    #[test]
    fn same_distribution_and_secret_produce_same_nullifier() {
        assert_eq!(
            derive_nullifier(&distribution(1), &claim_secret(2)),
            derive_nullifier(&distribution(1), &claim_secret(2))
        );
    }

    #[test]
    fn different_distribution_changes_nullifier() {
        assert_ne!(
            derive_nullifier(&distribution(1), &claim_secret(2)),
            derive_nullifier(&distribution(3), &claim_secret(2))
        );
    }

    #[test]
    fn different_secret_changes_nullifier() {
        assert_ne!(
            derive_nullifier(&distribution(1), &claim_secret(2)),
            derive_nullifier(&distribution(1), &claim_secret(4))
        );
    }

    #[test]
    fn nullifier_is_not_claim_pubkey_or_leaf() {
        let distribution_id = distribution(1);
        let secret = claim_secret(2);
        let claim_pubkey = derive_claim_pubkey(&secret);
        let leaf = compute_leaf(&EligibilityLeafInput {
            distribution_id,
            claim_pubkey,
            allocation: 100,
            leaf_salt: LeafSalt([5; 32]),
            recipient_binding: derive_recipient_binding(b"recipient"),
        });
        let nullifier = derive_nullifier(&distribution_id, &secret);

        assert_ne!(nullifier, claim_pubkey.0);
        assert_ne!(nullifier, leaf);
    }
}
