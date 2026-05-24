#[cfg(test)]
mod tests {
    use litesvm_utils::AssertionHelpers;
    use solana_signer::Signer;
    use stableswap::{self, Pool};
    
    use crate::common::*;

    #[test]
    fn initializes_pool() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        ctx.svm.assert_account_exists(&initialized.lp_mint);
        ctx.svm.assert_account_exists(&initialized.pool);

        let pool_acc = ctx.get_account::<Pool>(&initialized.pool).unwrap();
        
        assert_eq!(pool_acc.admin, actors.pool_admin.pubkey());
        assert_eq!(pool_acc.lp_mint, initialized.lp_mint);
        assert_eq!(pool_acc.amplification, initialized.amplification);
        assert_eq!(pool_acc.fee_bps, initialized.fee_bps);
        assert_eq!(pool_acc.n_tokens, 2);
        assert!(pool_acc.token_mints.contains(&initialized.mint_a));
        assert!(pool_acc.token_mints.contains(&initialized.mint_b));
    }
}
