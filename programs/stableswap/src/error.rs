use anchor_lang::prelude::*;

#[error_code]
pub enum StableSwapError {
    #[msg("Invalid amplification parameter")]
    InvalidAmplification,
    #[msg("Invalid fee: exceeds maximum")]
    InvalidFee,
    #[msg("Invalid number of tokens: must be between 2 and 6")]
    InvalidTokenCount,
    #[msg("Invalid token index")]
    InvalidTokenIndex,
    #[msg("Invalid vault account")]
    InvalidVault,
    #[msg("Invalid mint account")]
    InvalidMint,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Zero amount not allowed")]
    ZeroAmount,
    #[msg("Duplicate token mints")]
    DuplicateMint,
    #[msg("Newton's method failed to converge")]
    ConvergenceFailed,
    #[msg("Invalid remaining accounts")]
    InvalidRemainingAccounts,
    #[msg("Cannot swap same token")]
    SameTokenSwap,
    #[msg("Pool is empty")]
    EmptyPool,
    #[msg("Initial liquidity too low")]
    InsufficientInitialLiquidity,
}
