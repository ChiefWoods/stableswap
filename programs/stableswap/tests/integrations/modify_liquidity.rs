#[cfg(test)]
mod tests {
    use anchor_lang::Id;
    use anchor_litesvm::AccountMeta;
    use anchor_spl::token::Token;
    use litesvm_token::{get_spl_account, spl_token::state::Account as TokenAccount};
    use litesvm_utils::AssertionHelpers;
    use solana_signer::Signer;

    use crate::common::*;

    #[test]
    fn add_liquidity() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let depositor_a_ata =
            create_ata_for_owner(&mut ctx, &actors.depositor, &initialized.mint_a, &actors.depositor);
        let depositor_b_ata =
            create_ata_for_owner(&mut ctx, &actors.depositor, &initialized.mint_b, &actors.depositor);
        let depositor_lp_ata =
            create_ata_for_owner(&mut ctx, &actors.depositor, &initialized.lp_mint, &actors.depositor);

        mint_to_account(
            &mut ctx,
            &initialized.mint_a,
            &depositor_a_ata,
            &actors.pool_admin,
            1_000_000,
        );
        mint_to_account(
            &mut ctx,
            &initialized.mint_b,
            &depositor_b_ata,
            &actors.pool_admin,
            1_000_000,
        );

        let mut max_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        max_amounts[0] = 700_000;
        max_amounts[1] = 300_000;

        let mut ix = ctx
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
                max_amounts,
            })
            .instruction()
            .unwrap();

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
        ctx.svm.assert_mint_supply(&initialized.lp_mint, lp_account.amount);
    }

    #[test]
    fn remove_liquidity() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let initialized = initialize_pool_with_two_mints(&mut ctx, &actors.pool_admin);

        let depositor_a_ata =
            create_ata_for_owner(&mut ctx, &actors.depositor, &initialized.mint_a, &actors.depositor);
        let depositor_b_ata =
            create_ata_for_owner(&mut ctx, &actors.depositor, &initialized.mint_b, &actors.depositor);
        let depositor_lp_ata =
            create_ata_for_owner(&mut ctx, &actors.depositor, &initialized.lp_mint, &actors.depositor);

        mint_to_account(
            &mut ctx,
            &initialized.mint_a,
            &depositor_a_ata,
            &actors.pool_admin,
            1_000_000,
        );
        mint_to_account(
            &mut ctx,
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

        ctx.execute_instruction(add_ix, &[&actors.depositor]).unwrap();

        let lp_before: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        let lp_to_remove = lp_before.amount / 2;
        assert!(lp_to_remove > 0);

        let mut min_amounts = [0u64; stableswap::MAX_TOKENS as usize];
        min_amounts[0] = 1;
        min_amounts[1] = 1;

        let mut remove_ix = ctx
            .program()
            .accounts(stableswap::accounts::ModifyLiquidity {
                pool: initialized.pool,
                lp_mint: initialized.lp_mint,
                user_lp_account: depositor_lp_ata,
                user: actors.depositor.pubkey(),
                token_program: Token::id(),
            })
            .args(stableswap::instruction::RemoveLiquidity {
                lp_amount: lp_to_remove,
                min_amounts,
            })
            .instruction()
            .unwrap();

        remove_ix.accounts.extend_from_slice(&[
            AccountMeta::new(initialized.mint_a_vault, false),
            AccountMeta::new(initialized.mint_b_vault, false),
            AccountMeta::new(depositor_a_ata, false),
            AccountMeta::new(depositor_b_ata, false),
        ]);

        ctx.execute_instruction(remove_ix, &[&actors.depositor]).unwrap();

        let lp_after: TokenAccount = get_spl_account(&ctx.svm, &depositor_lp_ata).unwrap();
        assert_eq!(lp_after.amount, lp_before.amount - lp_to_remove);

        let user_a_after: TokenAccount = get_spl_account(&ctx.svm, &depositor_a_ata).unwrap();
        let user_b_after: TokenAccount = get_spl_account(&ctx.svm, &depositor_b_ata).unwrap();
        let vault_a_after: TokenAccount = get_spl_account(&ctx.svm, &initialized.mint_a_vault).unwrap();
        let vault_b_after: TokenAccount = get_spl_account(&ctx.svm, &initialized.mint_b_vault).unwrap();

        assert!(user_a_after.amount > 500_000);
        assert!(user_b_after.amount > 500_000);
        assert!(vault_a_after.amount < 500_000);
        assert!(vault_b_after.amount < 500_000);
    }
}
