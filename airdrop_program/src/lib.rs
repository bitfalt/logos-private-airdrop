//! SPEL-facing LP-0003 program crate.

pub mod errors;
pub mod state;
pub mod validation;

pub use state::{AllocationPolicy, DistributionMode, DistributionState, NullifierState};

pub const PROGRAM_NAME: &str = "private_airdrop";
