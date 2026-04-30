#![no_main]

use spel_framework::prelude::*;

risc0_zkvm::guest::entry!(main);

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

impl AllocationPolicy {
    fn from_tag(tag: u8) -> Result<Self, SpelError> {
        match tag {
            0 => Ok(Self::Equal),
            1 => Ok(Self::Variable),
            _ => Err(SpelError::custom(1, "LP0003_INVALID_ALLOCATION_POLICY")),
        }
    }
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
        #[account(mut, pda = literal("distribution"))] distribution: AccountWithMetadata,
        #[account(init, pda = literal("nullifier"))] nullifier: AccountWithMetadata,
        #[account(signer)] claimant: AccountWithMetadata,
        nullifier_hash: [u8; 32],
        allocation: u128,
    ) -> SpelResult {
        let _ = (nullifier_hash, allocation);
        Ok(SpelOutput::execute(
            vec![distribution, nullifier, claimant],
            vec![],
        ))
    }
}
