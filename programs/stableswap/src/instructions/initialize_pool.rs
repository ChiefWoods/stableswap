use crate::{error::StableSwapError, state::Pool, MAX_AMP, MAX_FEE_BPS, MAX_TOKENS, MIN_TOKENS};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::{Mint, Token},
};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = Pool::LEN,
        seeds = [b"pool", lp_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub lp_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializePool<'info> {
    pub fn validate(&self, amplification: u64, fee_bps: u16, n_tokens: usize) -> Result<()> {
        require!(
            amplification > 0 && amplification <= MAX_AMP,
            StableSwapError::InvalidAmplification
        );
        require!(fee_bps <= MAX_FEE_BPS, StableSwapError::InvalidFee);
        require!(
            n_tokens >= MIN_TOKENS.into() && n_tokens <= MAX_TOKENS.into(),
            StableSwapError::InvalidTokenCount
        );
        Ok(())
    }

    pub fn handler(ctx: Context<InitializePool>, amplification: u64, fee_bps: u16) -> Result<()> {
        let remaining = &ctx.remaining_accounts;

        // Each token needs: mint, vault (2 accounts per token)
        require!(
            remaining.len() >= MIN_TOKENS as usize * 2
                && remaining.len() <= MAX_TOKENS as usize * 2,
            StableSwapError::InvalidRemainingAccounts
        );
        require!(
            remaining.len() % 2 == 0,
            StableSwapError::InvalidRemainingAccounts
        );

        let n_tokens = remaining.len() / 2;

        ctx.accounts.validate(amplification, fee_bps, n_tokens)?;

        // Extract and validate mints, check for duplicates
        let mut token_mints = [Pubkey::default(); MAX_TOKENS as usize];

        for i in 0..n_tokens {
            let mint_info = &remaining[i * 2];
            let vault_info = &remaining[i * 2 + 1];

            // Validate mint is actually a mint account
            let mint_data = mint_info.try_borrow_data()?;
            require!(
                mint_data.len() == anchor_spl::token::Mint::LEN,
                StableSwapError::InvalidMint
            );

            // Check for duplicate mints
            for token_mint in token_mints {
                require!(
                    token_mint != mint_info.key(),
                    StableSwapError::DuplicateMint
                );
            }

            // Validate vault is the expected ATA
            let expected_vault =
                get_associated_token_address(&ctx.accounts.pool.key(), &mint_info.key());
            require!(
                vault_info.key() == expected_vault,
                StableSwapError::InvalidVault
            );

            token_mints[i] = mint_info.key();
        }

        ctx.accounts.pool.set_inner(Pool {
            admin: ctx.accounts.admin.key(),
            lp_mint: ctx.accounts.lp_mint.key(),
            amplification,
            fee_bps,
            n_tokens: n_tokens as u8,
            token_mints,
            bump: ctx.bumps.pool,
        });

        msg!(
            "Pool initialized with {} tokens, A={}, fee={}bps",
            n_tokens,
            amplification,
            fee_bps
        );

        Ok(())
    }
}
