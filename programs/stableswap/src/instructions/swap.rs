use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::TokenAccount,
    token::{transfer, Token, Transfer},
};

use crate::{error::StableSwapError, math::calculate_swap, Pool};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    // Remaining accounts passed dynamically:
    // - First n accounts: vault token accounts (one per token in pool)
    // - Next: user's input token account
    // - Last: user's output token account
}

impl<'info> Swap<'info> {
    pub fn validate_vaults(&self, vaults: &[AccountInfo<'info>]) -> Result<()> {
        for (i, vault) in vaults.iter().enumerate() {
            let expected =
                get_associated_token_address(&self.pool.key(), &self.pool.token_mints[i]);
            require!(vault.key() == expected, StableSwapError::InvalidVault);
        }
        Ok(())
    }

    pub fn read_reserves(&self, vaults: &[AccountInfo<'info>]) -> Result<Vec<u128>> {
        let mut reserves = Vec::with_capacity(vaults.len());
        for vault in vaults {
            let data = TokenAccount::try_deserialize(&mut &**vault.try_borrow_data()?)?;
            reserves.push(data.amount as u128);
        }
        Ok(reserves)
    }

    /// Transfer input tokens from user to vault
    pub fn transfer_in(
        &self,
        user_ata: &AccountInfo<'info>,
        vault: &AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        // transfer_checked not used to avoid passing mint accounts
        transfer(
            CpiContext::new(
                self.token_program.key(),
                Transfer {
                    from: user_ata.to_account_info(),
                    to: vault.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )
    }

    /// Transfer output tokens from vault to user
    pub fn transfer_out(
        &self,
        vault: &AccountInfo<'info>,
        user_ata: &AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let seeds: &[&[u8]] = &[b"pool", self.pool.lp_mint.as_ref(), &[self.pool.bump]];

        // transfer_checked not used to avoid passing mint accounts
        transfer(
            CpiContext::new_with_signer(
                self.token_program.key(),
                Transfer {
                    from: vault.to_account_info(),
                    to: user_ata.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )
    }

    pub fn handler(
        ctx: Context<'info, Swap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
        input_index: u8,
        output_index: u8,
    ) -> Result<()> {
        require!(amount_in > 0, StableSwapError::ZeroAmount);
        let pool = &ctx.accounts.pool;
        let n = pool.n_tokens as usize;
        let remaining = ctx.remaining_accounts;
        // Validate indices
        require!(
            input_index < pool.n_tokens,
            StableSwapError::InvalidTokenIndex
        );
        require!(
            output_index < pool.n_tokens,
            StableSwapError::InvalidTokenIndex
        );
        require!(input_index != output_index, StableSwapError::SameTokenSwap);
        // Validate remaining accounts: n vaults + 2 user ATAs
        require!(
            remaining.len() == n + 2,
            StableSwapError::InvalidRemainingAccounts
        );
        let vaults = &remaining[0..n];
        let user_input = &remaining[n];
        let user_output = &remaining[n + 1];
        ctx.accounts.validate_vaults(vaults)?;
        let reserves = ctx.accounts.read_reserves(vaults)?;
        // Calculate swap using StableSwap math
        let (amount_out, fee_amount) = calculate_swap(
            &reserves,
            input_index as usize,
            output_index as usize,
            amount_in as u128,
            pool.amplification as u128,
            pool.fee_bps,
        )?;
        // Slippage check
        require!(
            amount_out >= min_amount_out as u128,
            StableSwapError::SlippageExceeded
        );
        // Execute transfers
        ctx.accounts
            .transfer_in(user_input, &vaults[input_index as usize], amount_in)?;
        ctx.accounts.transfer_out(
            &vaults[output_index as usize],
            user_output,
            amount_out as u64,
        )?;
        msg!(
            "Swap: {} in -> {} out (fee: {})",
            amount_in,
            amount_out,
            fee_amount
        );

        Ok(())
    }
}
