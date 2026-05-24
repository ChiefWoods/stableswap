use crate::constants::MAX_TOKENS;
use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub admin: Pubkey,
    pub lp_mint: Pubkey,
    pub amplification: u64,
    pub fee_bps: u16,
    pub n_tokens: u8,
    pub token_mints: [Pubkey; MAX_TOKENS as usize],
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = Pool::DISCRIMINATOR.len() +
        32 +                    // admin
        32 +                    // lp_mint
        8 +                     // amplification
        2 +                     // fee_bps
        1 +                     // n_tokens
        32 * MAX_TOKENS as usize +       // token_mints
        1; // bump

    pub fn active_mints(&self) -> &[Pubkey] {
        &self.token_mints[..self.n_tokens as usize]
    }
}
