//! Core LP-0003 cryptographic primitives.

pub mod domain;
pub mod eligibility;
pub mod errors;
pub mod hash;
pub mod merkle;
pub mod nullifier;
pub mod test_vectors;

pub const CRATE_NAME: &str = "airdrop_core";
