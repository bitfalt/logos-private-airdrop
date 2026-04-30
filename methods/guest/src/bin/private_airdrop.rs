#![no_main]

use sha2::{Digest as _, Sha256};
use spel_framework::prelude::*;

risc0_zkvm::guest::entry!(main);

const LEAF_DOMAIN: &[u8] = b"lp0003:leaf:v1";
const CLAIM_PUBKEY_DOMAIN: &[u8] = b"lp0003:claim-pubkey:v1";
const NULLIFIER_DOMAIN: &[u8] = b"lp0003:nullifier:v1";
const NODE_DOMAIN: &[u8] = b"lp0003:node:v1";

const ERR_DISTRIBUTION_PAUSED: &str = "LP0003_DISTRIBUTION_PAUSED";
const ERR_NULLIFIER_ALREADY_CLAIMED: &str = "LP0003_NULLIFIER_ALREADY_CLAIMED";
const ERR_INVALID_MERKLE_PROOF: &str = "LP0003_INVALID_MERKLE_PROOF";
const ERR_ALLOCATION_MISMATCH: &str = "LP0003_ALLOCATION_MISMATCH";

/// Distribution PDA data. The PDA seed is the stable pair
/// `[literal("distribution"), arg("distribution_id")]`.
#[account_type]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct DistributionState {
    pub version: u8,
    pub authority: [u8; 32],
    pub distribution_id: [u8; 32],
    pub merkle_root: [u8; 32],
    pub mode: DistributionMode,
    pub allocation_policy: AllocationPolicy,
    pub total_eligible: u64,
    pub total_claimed: u64,
    pub created_at_slot_or_time: u64,
    pub paused: bool,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum DistributionMode {
    Airdrop,
    AllowlistGate,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum AllocationPolicy {
    Equal,
    Variable,
}

#[account_type]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct NullifierState {
    pub version: u8,
    pub distribution_id: [u8; 32],
    pub nullifier: [u8; 32],
    pub allocation: u128,
    pub claimed_at_slot_or_time: u64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ClaimWitness {
    pub claim_secret: [u8; 32],
    pub leaf_salt: [u8; 32],
    pub recipient_binding: [u8; 32],
    pub proof_leaf: [u8; 32],
    pub proof_index: u64,
    pub proof_siblings: Vec<[u8; 32]>,
}

impl AllocationPolicy {
    fn from_tag(tag: u8) -> Result<Self, SpelError> {
        match tag {
            0 => Ok(Self::Equal),
            1 => Ok(Self::Variable),
            _ => Err(SpelError::custom(1, "LP0003_INVALID_ALLOCATION_POLICY")),
        }
    }
}

fn custom_error(code: u32, message: &'static str) -> SpelError {
    SpelError::custom(code, message)
}

fn sha256_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

fn derive_claim_pubkey(claim_secret: &[u8; 32]) -> [u8; 32] {
    sha256_parts(&[CLAIM_PUBKEY_DOMAIN, claim_secret])
}

fn derive_nullifier(distribution_id: &[u8; 32], claim_secret: &[u8; 32]) -> [u8; 32] {
    sha256_parts(&[NULLIFIER_DOMAIN, distribution_id, claim_secret])
}

fn compute_leaf(
    distribution_id: &[u8; 32],
    claim_pubkey: &[u8; 32],
    allocation: u128,
    leaf_salt: &[u8; 32],
    recipient_binding: &[u8; 32],
) -> [u8; 32] {
    let allocation_bytes = allocation.to_be_bytes();
    sha256_parts(&[
        LEAF_DOMAIN,
        distribution_id,
        claim_pubkey,
        &allocation_bytes,
        leaf_salt,
        recipient_binding,
    ])
}

fn node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    sha256_parts(&[NODE_DOMAIN, &left, &right])
}

fn verify_merkle_proof(root: [u8; 32], leaf: [u8; 32], index: u64, siblings: &[[u8; 32]]) -> bool {
    let mut current = leaf;
    let mut node_index = index;

    for sibling in siblings {
        current = if node_index % 2 == 0 {
            node_hash(current, *sibling)
        } else {
            node_hash(*sibling, current)
        };
        node_index /= 2;
    }

    current == root
}

#[lez_program]
mod private_airdrop {
    #[allow(unused_imports)]
    use super::*;

    #[instruction]
    pub fn initialize_distribution(
        #[account(init, pda = [literal("distribution"), arg("distribution_id")])]
        mut distribution: AccountWithMetadata,
        #[account(signer)] authority: AccountWithMetadata,
        distribution_id: [u8; 32],
        merkle_root: [u8; 32],
        allocation_policy: u8,
        total_eligible: u64,
    ) -> SpelResult {
        let state = DistributionState {
            version: 1,
            authority: *authority.account_id.value(),
            distribution_id,
            merkle_root,
            mode: DistributionMode::Airdrop,
            allocation_policy: AllocationPolicy::from_tag(allocation_policy)?,
            total_eligible,
            total_claimed: 0,
            created_at_slot_or_time: 0,
            paused: false,
        };
        let bytes = borsh::to_vec(&state).map_err(|e| SpelError::SerializationError {
            message: e.to_string(),
        })?;
        distribution.account.data = bytes.try_into().unwrap();

        Ok(SpelOutput::execute(vec![distribution, authority], vec![]))
    }

    #[instruction]
    pub fn claim(
        #[account(mut, pda = [literal("distribution"), arg("distribution_id")])]
        mut distribution: AccountWithMetadata,
        #[account(init, pda = [literal("nullifier"), arg("nullifier_hash")])]
        mut nullifier: AccountWithMetadata,
        #[account(signer)] claimant: AccountWithMetadata,
        distribution_id: [u8; 32],
        nullifier_hash: [u8; 32],
        allocation: u128,
    ) -> SpelResult {
        let distribution_data: Vec<u8> = distribution.account.data.clone().into();
        let mut distribution_state: DistributionState = borsh::from_slice(&distribution_data)
            .map_err(|e| SpelError::DeserializationError {
                account_index: 0,
                message: e.to_string(),
            })?;

        if distribution_state.paused {
            return Err(custom_error(10, ERR_DISTRIBUTION_PAUSED));
        }
        if distribution_state.distribution_id != distribution_id {
            return Err(custom_error(11, ERR_INVALID_MERKLE_PROOF));
        }

        let nullifier_data: Vec<u8> = nullifier.account.data.clone().into();
        if !nullifier_data.is_empty() {
            return Err(custom_error(12, ERR_NULLIFIER_ALREADY_CLAIMED));
        }

        let witness_data: Vec<u8> = claimant.account.data.clone().into();
        let witness: ClaimWitness =
            borsh::from_slice(&witness_data).map_err(|e| SpelError::DeserializationError {
                account_index: 2,
                message: e.to_string(),
            })?;
        let claim_pubkey = derive_claim_pubkey(&witness.claim_secret);
        if derive_nullifier(&distribution_id, &witness.claim_secret) != nullifier_hash {
            return Err(custom_error(13, ERR_INVALID_MERKLE_PROOF));
        }

        if !verify_merkle_proof(
            distribution_state.merkle_root,
            witness.proof_leaf,
            witness.proof_index,
            &witness.proof_siblings,
        ) {
            return Err(custom_error(14, ERR_INVALID_MERKLE_PROOF));
        }

        let expected_leaf = compute_leaf(
            &distribution_id,
            &claim_pubkey,
            allocation,
            &witness.leaf_salt,
            &witness.recipient_binding,
        );
        if witness.proof_leaf != expected_leaf {
            return Err(custom_error(15, ERR_ALLOCATION_MISMATCH));
        }

        distribution_state.total_claimed = distribution_state.total_claimed.saturating_add(1);
        let nullifier_state = NullifierState {
            version: 1,
            distribution_id,
            nullifier: nullifier_hash,
            allocation,
            claimed_at_slot_or_time: 0,
        };

        let distribution_bytes =
            borsh::to_vec(&distribution_state).map_err(|e| SpelError::SerializationError {
                message: e.to_string(),
            })?;
        distribution.account.data = distribution_bytes.try_into().unwrap();
        let nullifier_bytes =
            borsh::to_vec(&nullifier_state).map_err(|e| SpelError::SerializationError {
                message: e.to_string(),
            })?;
        nullifier.account.data = nullifier_bytes.try_into().unwrap();

        Ok(SpelOutput::execute(
            vec![distribution, nullifier, claimant],
            vec![],
        ))
    }
}
