use solana_address::Address;

#[allow(dead_code)]
pub fn derive_pool_address(lp_mint: Address) -> Address {
    Address::find_program_address(&[b"pool", lp_mint.as_ref()], &stableswap::id()).0
}
