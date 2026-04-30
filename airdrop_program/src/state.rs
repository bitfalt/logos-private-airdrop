use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum DistributionMode {
    Airdrop,
    AllowlistGate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum AllocationPolicy {
    Equal,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct NullifierState {
    pub version: u8,
    pub distribution_id: [u8; 32],
    pub nullifier: [u8; 32],
    pub allocation: u128,
    pub claimed_at_slot_or_time: u64,
}

#[cfg(test)]
mod tests {
    use borsh::BorshDeserialize;

    use super::{AllocationPolicy, DistributionMode, DistributionState, NullifierState};

    #[test]
    fn distribution_state_round_trips_with_variable_allocation_policy() {
        let state = DistributionState {
            version: 1,
            authority: [1; 32],
            distribution_id: [2; 32],
            merkle_root: [3; 32],
            mode: DistributionMode::Airdrop,
            allocation_policy: AllocationPolicy::Variable,
            total_eligible: 30,
            total_claimed: 0,
            created_at_slot_or_time: 42,
            paused: false,
        };
        let bytes = borsh::to_vec(&state).unwrap();
        assert_eq!(DistributionState::try_from_slice(&bytes).unwrap(), state);
    }

    #[test]
    fn nullifier_state_round_trips() {
        let state = NullifierState {
            version: 1,
            distribution_id: [2; 32],
            nullifier: [9; 32],
            allocation: 100,
            claimed_at_slot_or_time: 77,
        };
        let bytes = borsh::to_vec(&state).unwrap();
        assert_eq!(NullifierState::try_from_slice(&bytes).unwrap(), state);
    }
}
