#[cfg(test)]
mod tests {
    use anchor_lang::Id;
    use anchor_litesvm::AccountMeta;
    use anchor_spl::token::Token;
    use litesvm_token::get_spl_account;
    use litesvm_token::{spl_token::state::Account as TokenAccount, CreateAssociatedTokenAccount};
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

    fn seed_pool_liquidity(
        ctx: &mut anchor_litesvm::AnchorContext,
        actors: &TestActors,
        initialized: &InitializedPool,
        depositor_a_ata: Address,
        depositor_b_ata: Address,
        depositor_lp_ata: Address,
    ) {
        mint_to_account(
            ctx,
            &initialized.mint_a,
            &depositor_a_ata,
            &actors.pool_admin,
            1_000_000,
        );
        mint_to_account(
            ctx,
            &initialized.mint_b,
            &depositor_b_ata,
            &actors.pool_admin,
            1_000_000,
        );

        let mut add_max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        add_max_amounts[0] = 500_000;
        add_max_amounts[1] = 500_000;

        let mut add_ix = ctx
            .program()
            .accounts(stableswap::accounts::ModifyLiquidity {
                pool: initialized.pool,
                lp_mint: initialized.lp_mint,
                user_lp_account: depositor_lp_ata,
                user: actors.depositor.pubkey(),
                token_program: Token::id(),
            })
            .args(stableswap::instruction::AddLiquidity {
                lp_amount: 100_000,
                max_amounts: add_max_amounts,
            })
            .instruction()
            .unwrap();

        add_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(add_ix, &[&actors.depositor])
            .unwrap();
    }

    fn build_swap_ix(
        ctx: &mut anchor_litesvm::AnchorContext,
        initialized: &InitializedPool,
        user: &solana_keypair::Keypair,
        amount_in: u64,
        min_amount_out: u64,
        input_index: u8,
        output_index: u8,
    ) -> anchor_lang::solana_program::instruction::Instruction {
        ctx.program()
            .accounts(stableswap::accounts::Swap {
                pool: initialized.pool,
                user: user.pubkey(),
                token_program: Token::id(),
            })
            .args(stableswap::instruction::Swap {
                amount_in,
                min_amount_out,
                input_index,
                output_index,
            })
            .instruction()
            .unwrap()
    }

    #[test]
    fn swap_a_to_b() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let (depositor_a_ata, depositor_b_ata, depositor_lp_ata) =
            setup_depositor_accounts(&mut ctx, &actors, &initialized);

        seed_pool_liquidity(
            &mut ctx,
            &actors,
            &initialized,
            depositor_a_ata,
            depositor_b_ata,
            depositor_lp_ata,
        );

        let swap_user_a_ata = create_ata_for_owner(
            &mut ctx,
            &actors.swap_user,
            &initialized.mint_a,
            &actors.swap_user,
        );
        let swap_user_b_ata = create_ata_for_owner(
            &mut ctx,
            &actors.swap_user,
            &initialized.mint_b,
            &actors.swap_user,
        );

        mint_to_account(
            &mut ctx,
            &initialized.mint_a,
            &swap_user_a_ata,
            &actors.pool_admin,
            100_000,
        );

        let swap_amount_in = 50_000;
        let user_a_before: TokenAccount = get_spl_account(&ctx.svm, &swap_user_a_ata).unwrap();
        let user_b_before: TokenAccount = get_spl_account(&ctx.svm, &swap_user_b_ata).unwrap();
        let vault_a_before: TokenAccount =
            get_spl_account(&ctx.svm, &initialized.mint_a_vault).unwrap();
        let vault_b_before: TokenAccount =
            get_spl_account(&ctx.svm, &initialized.mint_b_vault).unwrap();

        let mut swap_ix = build_swap_ix(
            &mut ctx,
            &initialized,
            &actors.swap_user,
            swap_amount_in,
            1,
            0,
            1,
        );

        swap_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(swap_user_a_ata, false),
            AccountMeta::new(swap_user_b_ata, false),
        ]);

        ctx.execute_instruction(swap_ix, &[&actors.swap_user])
            .unwrap();

        let user_a_after: TokenAccount = get_spl_account(&ctx.svm, &swap_user_a_ata).unwrap();
        let user_b_after: TokenAccount = get_spl_account(&ctx.svm, &swap_user_b_ata).unwrap();
        let vault_a_after: TokenAccount =
            get_spl_account(&ctx.svm, &initialized.mint_a_vault).unwrap();
        let vault_b_after: TokenAccount =
            get_spl_account(&ctx.svm, &initialized.mint_b_vault).unwrap();

        assert_eq!(user_a_after.amount, user_a_before.amount - swap_amount_in);
        assert!(user_b_after.amount > user_b_before.amount);
        assert_eq!(vault_a_after.amount, vault_a_before.amount + swap_amount_in);
        assert!(vault_b_after.amount < vault_b_before.amount);
    }

    #[test]
    fn swap_rejects_zero_amount_in() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let ix = build_swap_ix(&mut ctx, &initialized, &actors.swap_user, 0, 1, 0, 1);
        let result = ctx.execute_instruction(ix, &[&actors.swap_user]).unwrap();
        result.assert_anchor_error("ZeroAmount");
    }

    #[test]
    fn swap_rejects_same_input_output_index() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let ix = build_swap_ix(&mut ctx, &initialized, &actors.swap_user, 10, 1, 0, 0);
        let result = ctx.execute_instruction(ix, &[&actors.swap_user]).unwrap();
        result.assert_anchor_error("SameTokenSwap");
    }

    #[test]
    fn swap_rejects_invalid_input_or_output_index() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let ix = build_swap_ix(&mut ctx, &initialized, &actors.swap_user, 10, 1, 2, 1);
        let result = ctx.execute_instruction(ix, &[&actors.swap_user]).unwrap();
        result.assert_anchor_error("InvalidTokenIndex");
    }

    #[test]
    fn swap_rejects_wrong_vault_account() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let swap_user_a_ata = create_ata_for_owner(
            &mut ctx,
            &actors.swap_user,
            &initialized.mint_a,
            &actors.swap_user,
        );
        let swap_user_b_ata = create_ata_for_owner(
            &mut ctx,
            &actors.swap_user,
            &initialized.mint_b,
            &actors.swap_user,
        );

        let wrong_vault = CreateAssociatedTokenAccount::new(
            &mut ctx.svm,
            &actors.pool_admin,
            &initialized.mint_a,
        )
        .owner(&actors.pool_admin.pubkey())
        .send()
        .unwrap();

        let mut ix = build_swap_ix(&mut ctx, &initialized, &actors.swap_user, 10, 1, 0, 1);
        ix.accounts.extend_from_slice(&[
            AccountMeta::new(wrong_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(swap_user_a_ata, false),
            AccountMeta::new(swap_user_b_ata, false),
        ]);

        let result = ctx.execute_instruction(ix, &[&actors.swap_user]).unwrap();
        result.assert_anchor_error("InvalidVault");
    }

    #[test]
    fn swap_rejects_wrong_remaining_account_count() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let ix = build_swap_ix(&mut ctx, &initialized, &actors.swap_user, 10, 1, 0, 1);
        let result = ctx.execute_instruction(ix, &[&actors.swap_user]).unwrap();
        result.assert_anchor_error("InvalidRemainingAccounts");
    }
}
