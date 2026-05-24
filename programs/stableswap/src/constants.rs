use anchor_lang::prelude::*;

#[constant]
pub const MAX_TOKENS: u8 = 6;
pub const MIN_TOKENS: u8 = 2;
pub const MINIMUM_LIQUIDITY: u64 = 1_000;
pub const MAX_AMP: u64 = 1_000_000;
pub const MAX_FEE_BPS: u16 = 10000;
