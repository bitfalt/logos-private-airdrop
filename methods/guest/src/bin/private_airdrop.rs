#![no_main]

use spel_framework::prelude::*;

risc0_zkvm::guest::entry!(main);

#[lez_program]
mod private_airdrop {
    #[allow(unused_imports)]
    use super::*;

    #[instruction]
    pub fn initialize(
        #[account(init, pda = literal("distribution"))] distribution: AccountWithMetadata,
        #[account(signer)] authority: AccountWithMetadata,
    ) -> SpelResult {
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
