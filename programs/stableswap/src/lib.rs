pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DXTgWdvcdzrRYCmxNTR5SXcTYm5NzfS1XEZDc7ngEeHN");

#[program]
pub mod stableswap {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        amplification: u64,
        fee_bps: u16,
    ) -> Result<()> {
        InitializePool::handler(ctx, amplification, fee_bps)
    }

    pub fn add_liquidity<'info>(
        ctx: Context<'info, ModifyLiquidity<'info>>,
        lp_amount: u64,
        max_amounts: [u64; MAX_TOKENS as usize],
    ) -> Result<()> {
        ModifyLiquidity::add_liquidity_handler(ctx, lp_amount, max_amounts)
    }

    pub fn remove_liquidity<'info>(
        ctx: Context<'info, ModifyLiquidity<'info>>,
        lp_amount: u64,
        min_amounts: [u64; MAX_TOKENS as usize],
    ) -> Result<()> {
        ModifyLiquidity::remove_liquidity_handler(ctx, lp_amount, min_amounts)
    }

    pub fn swap<'info>(
        ctx: Context<'info, Swap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
        input_index: u8,
        output_index: u8,
    ) -> Result<()> {
        Swap::handler(ctx, amount_in, min_amount_out, input_index, output_index)
    }
}
