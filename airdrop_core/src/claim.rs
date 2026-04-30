use crate::{
    eligibility::{ClaimSecret, LeafSalt, RecipientBinding},
    hash::Hash32,
    merkle::MerkleProof,
    nullifier::derive_nullifier,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimPackage {
    public_inputs: ClaimPublicInputs,
    private_witness: ClaimPrivateWitness,
}

impl ClaimPackage {
    #[must_use]
    pub fn new(
        distribution_id: Hash32,
        claim_secret: ClaimSecret,
        allocation: u128,
        leaf_salt: LeafSalt,
        recipient_binding: RecipientBinding,
        merkle_proof: MerkleProof,
    ) -> Self {
        let nullifier = derive_nullifier(&distribution_id, &claim_secret);
        Self {
            public_inputs: ClaimPublicInputs {
                distribution_id,
                nullifier,
                allocation,
                recipient_binding: recipient_binding.0,
            },
            private_witness: ClaimPrivateWitness {
                claim_secret,
                leaf_salt,
                recipient_binding,
                merkle_proof,
            },
        }
    }

    #[must_use]
    pub fn public_inputs(&self) -> &ClaimPublicInputs {
        &self.public_inputs
    }

    #[must_use]
    pub fn private_witness(&self) -> &ClaimPrivateWitness {
        &self.private_witness
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimPublicInputs {
    pub distribution_id: Hash32,
    pub nullifier: Hash32,
    pub allocation: u128,
    pub recipient_binding: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimPrivateWitness {
    pub claim_secret: ClaimSecret,
    pub leaf_salt: LeafSalt,
    pub recipient_binding: RecipientBinding,
    pub merkle_proof: MerkleProof,
}

#[cfg(test)]
mod tests {
    use crate::{
        eligibility::{ClaimSecret, LeafSalt, derive_recipient_binding},
        merkle::MerkleProof,
    };

    use super::ClaimPackage;

    #[test]
    fn public_claim_output_excludes_secret() {
        let package = ClaimPackage::new(
            [1; 32],
            ClaimSecret([2; 32]),
            100,
            LeafSalt([3; 32]),
            derive_recipient_binding(b"recipient"),
            MerkleProof {
                leaf: [4; 32],
                index: 0,
                siblings: vec![[5; 32]],
            },
        );

        let public = package.public_inputs();
        assert_eq!(public.distribution_id, [1; 32]);
        assert_eq!(public.allocation, 100);
        assert_ne!(public.nullifier, [2; 32]);
    }
}
