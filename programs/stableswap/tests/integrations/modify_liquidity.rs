#[cfg(test)]
mod tests {
    use anchor_lang::Id;
    use anchor_litesvm::AccountMeta;
    use anchor_spl::token::Token;
    use litesvm_token::{get_spl_account, spl_token::state::Account as TokenAccount};
    use litesvm_utils::AssertionHelpers;
    use solana_address::Address;
    use solana_signer::Signer;

    use crate::common::*;

    fn setup_depositor_accounts(
        ctx: &mut anchor_litesvm::AnchorContext,
        actors: &TestActors,
        initialized: &InitializedPool,
    ) -> (Address, Address, Address) {
        let depositor_a_ata = create_ata_for_owner(
            ctx,
            &actors.depositor,
            &initialized.mint_a,
            &actors.depositor,
        );
        let depositor_b_ata = create_ata_for_owner(
            ctx,
            &actors.depositor,
            &initialized.mint_b,
            &actors.depositor,
        );
        let depositor_lp_ata = create_ata_for_owner(
            ctx,
            &actors.depositor,
            &initialized.lp_mint,
            &actors.depositor,
        );

        (depositor_a_ata, depositor_b_ata, depositor_lp_ata)
    }

    fn build_add_ix(
        ctx: &mut anchor_litesvm::AnchorContext,
        initialized: &InitializedPool,
        user: &solana_keypair::Keypair,
        user_lp_ata: Address,
        lp_amount: u64,
        max_amounts: [u64; stableswap::MAX_TOKENS as usize],
    ) -> anchor_lang::solana_program::instruction::Instruction {
        ctx.program()
            .accounts(stableswap::accounts::ModifyLiquidity {
                pool: initialized.pool,
                lp_mint: initialized.lp_mint,
                user_lp_account: user_lp_ata,
                user: user.pubkey(),
                token_program: Token::id(),
            })
            .args(stableswap::instruction::AddLiquidity {
                lp_amount,
                max_amounts,
            })
            .instruction()
            .unwrap()
    }

    fn build_remove_ix(
        ctx: &mut anchor_litesvm::AnchorContext,
        initialized: &InitializedPool,
        user: &solana_keypair::Keypair,
        user_lp_ata: Address,
        lp_amount: u64,
        min_amounts: [u64; stableswap::MAX_TOKENS as usize],
    ) -> anchor_lang::solana_program::instruction::Instruction {
        ctx.program()
            .accounts(stableswap::accounts::ModifyLiquidity {
                pool: initialized.pool,
                lp_mint: initialized.lp_mint,
                user_lp_account: user_lp_ata,
                user: user.pubkey(),
                token_program: Token::id(),
            })
            .args(stableswap::instruction::RemoveLiquidity {
                lp_amount,
                min_amounts,
            })
            .instruction()
            .unwrap()
    }

    fn fund_depositor_tokens(
        ctx: &mut anchor_litesvm::AnchorContext,
        actors: &TestActors,
        initialized: &InitializedPool,
        depositor_a_ata: Address,
        depositor_b_ata: Address,
        amount: u64,
    ) {
        mint_to_account(
            ctx,
            &initialized.mint_a,
            &depositor_a_ata,
            &actors.pool_admin,
            amount,
        );
        mint_to_account(
            ctx,
            &initialized.mint_b,
            &depositor_b_ata,
            &actors.pool_admin,
            amount,
        );
    }

    #[test]
    fn add_liquidity() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        fund_depositor_tokens(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            1_000_000,
        );

        let mut max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        max_amounts[0] = 700_000;
        max_amounts[1] = 300_000;

        let mut ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            100_000,
            max_amounts,
        );

        ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(ix, &[&actors.depositor]).unwrap();

        ctx.svm.assert_token_balance(&depositor_a_ata, 300_000);
        ctx.svm.assert_token_balance(&depositor_b_ata, 700_000);
        ctx.svm
            .assert_token_balance(&initialized.mint_a_vault, 700_000);
        ctx.svm
            .assert_token_balance(&initialized.mint_b_vault, 300_000);

        let lp_account: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        assert!(lp_account.amount > 0);
        ctx.svm
            .assert_mint_supply(&initialized.lp_mint, lp_account.amount);
    }

    #[test]
    fn add_liquidity_rejects_zero_lp_amount() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);
        let (_, _, depositor_lp_ata) = setup_depositor_accounts(&mut ctx, &actors, &initialized);

        let max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        let ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            0,
            max_amounts,
        );

        let result = ctx.execute_instruction(ix, &[&actors.depositor]).unwrap();
        result.assert_anchor_error("ZeroAmount");
    }

    #[test]
    fn add_liquidity_rejects_all_zero_max_amounts() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);
        let (_, _, depositor_lp_ata) = setup_depositor_accounts(&mut ctx, &actors, &initialized);

        let max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        let ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            1,
            max_amounts,
        );

        let result = ctx.execute_instruction(ix, &[&actors.depositor]).unwrap();
        result.assert_anchor_error("ZeroAmount");
    }

    #[test]
    fn add_liquidity_rejects_insufficient_initial_liquidity() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        fund_depositor_tokens(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            10,
        );

        let mut max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        max_amounts[0] = 1;
        max_amounts[1] = 1;

        let mut ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            1,
            max_amounts,
        );

        ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        let result = ctx.execute_instruction(ix, &[&actors.depositor]).unwrap();
        result.assert_anchor_error("InsufficientInitialLiquidity");
    }

    #[test]
    fn add_liquidity_rejects_wrong_remaining_account_count() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        fund_depositor_tokens(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            1_000_000,
        );

        let mut max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        max_amounts[0] = 500_000;
        max_amounts[1] = 500_000;

        let mut ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            100_000,
            max_amounts,
        );

        ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
        ]);

        let result = ctx.execute_instruction(ix, &[&actors.depositor]).unwrap();
        result.assert_anchor_error("InvalidRemainingAccounts");
    }

    #[test]
    fn remove_liquidity() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        fund_depositor_tokens(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            1_000_000,
        );

        let mut add_max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        add_max_amounts[0] = 500_000;
        add_max_amounts[1] = 500_000;

        let mut add_ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            100_000,
            add_max_amounts,
        );

        add_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(add_ix, &[&actors.depositor])
            .unwrap();

        let lp_before: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        let lp_to_remove = lp_before.amount / 2;
        assert!(lp_to_remove > 0);

        let mut min_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        min_amounts[0] = 1;
        min_amounts[1] = 1;

        let mut remove_ix = build_remove_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            lp_to_remove,
            min_amounts,
        );

        remove_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(remove_ix, &[&actors.depositor])
            .unwrap();

        let lp_after: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        assert_eq!(lp_after.amount, lp_before.amount - lp_to_remove);

        let user_a_after: TokenAccount = get_spl_account(&ctx.svm, &depositor_a_ata).unwrap();
        let user_b_after: TokenAccount = get_spl_account(&ctx.svm, &depositor_b_ata).unwrap();
        let vault_a_after: TokenAccount =
            get_spl_account(&ctx.svm, &initialized.mint_a_vault).unwrap();
        let vault_b_after: TokenAccount =
            get_spl_account(&ctx.svm, &initialized.mint_b_vault).unwrap();

        assert!(user_a_after.amount > 500_000);
        assert!(user_b_after.amount > 500_000);
        assert!(vault_a_after.amount < 500_000);
        assert!(vault_b_after.amount < 500_000);
    }

    #[test]
    fn remove_liquidity_rejects_empty_pool() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        let min_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        let mut ix = build_remove_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            1,
            min_amounts,
        );

        ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        let result = ctx.execute_instruction(ix, &[&actors.depositor]).unwrap();
        result.assert_anchor_error("EmptyPool");
    }

    #[test]
    fn remove_liquidity_rejects_amount_over_supply() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        fund_depositor_tokens(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            1_000_000,
        );

        let mut add_max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        add_max_amounts[0] = 500_000;
        add_max_amounts[1] = 500_000;

        let mut add_ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            100_000,
            add_max_amounts,
        );

        add_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(add_ix, &[&actors.depositor])
            .unwrap();

        let lp_before: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        let min_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        let mut remove_ix = build_remove_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            lp_before.amount + 1,
            min_amounts,
        );

        remove_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        let result = ctx
            .execute_instruction(remove_ix, &[&actors.depositor])
            .unwrap();
        result.assert_anchor_error("InsufficientLiquidity");
    }

    #[test]
    fn remove_liquidity_rejects_wrong_remaining_account_count() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        fund_depositor_tokens(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            1_000_000,
        );

        let mut add_max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        add_max_amounts[0] = 500_000;
        add_max_amounts[1] = 500_000;

        let mut add_ix = build_add_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            100_000,
            add_max_amounts,
        );

        add_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(add_ix, &[&actors.depositor])
            .unwrap();

        let lp_before: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        let mut min_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        min_amounts[0] = 1;
        min_amounts[1] = 1;

        let mut remove_ix = build_remove_ix(
            &mut ctx,
            &initialized,
            &actors.depositor,
            depositor_lp_ata,
            lp_before.amount / 2,
            min_amounts,
        );

        remove_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
        ]);

        let result = ctx
            .execute_instruction(remove_ix, &[&actors.depositor])
            .unwrap();
        result.assert_anchor_error("InvalidRemainingAccounts");
    }
}
