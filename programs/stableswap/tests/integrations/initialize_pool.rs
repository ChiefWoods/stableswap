#[cfg(test)]
mod tests {
    use anchor_lang::{
        solana_program::{rent::Rent, sysvar::SysvarId},
        system_program::System,
        Id,
    };
    use anchor_litesvm::AccountMeta;
    use anchor_spl::{associated_token::AssociatedToken, token::Token};
    use litesvm_token::{CreateAssociatedTokenAccount, CreateMint};
    use litesvm_utils::AssertionHelpers;
    use solana_address::Address;
    use solana_keypair::Keypair;
    use solana_signer::Signer;
    use stableswap::{self, Pool};

    use crate::common::*;

    fn derive_pool_address(lp_mint: Address) -> Address {
        Address::find_program_address(&[b"pool", lp_mint.as_ref()], &stableswap::id()).0
    }

    fn build_initialize_ix(
        ctx: &mut anchor_litesvm::AnchorContext,
        admin: &Keypair,
        lp_mint: &Keypair,
        amplification: u64,
        fee_bps: u16,
    ) -> anchor_lang::solana_program::instruction::Instruction {
        ctx.program()
            .accounts(stableswap::accounts::InitializePool {
                admin: admin.pubkey(),
                associated_token_program: AssociatedToken::id(),
                lp_mint: lp_mint.pubkey(),
                pool: derive_pool_address(lp_mint.pubkey()),
                rent: Rent::id(),
                system_program: System::id(),
                token_program: Token::id(),
            })
            .args(stableswap::instruction::InitializePool {
                amplification,
                fee_bps,
            })
            .instruction()
            .unwrap()
    }

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

    #[test]
    fn initialize_pool_rejects_zero_amplification() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let lp_mint = Keypair::new();
        let pool = derive_pool_address(lp_mint.pubkey());

        let mint_a = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();

        let vault_a = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_a)
            .owner(&pool)
            .send()
            .unwrap();
        let vault_b = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_b)
            .owner(&pool)
            .send()
            .unwrap();

        let mut ix =
            build_initialize_ix(&mut ctx, &actors.pool_admin, &lp_mint, 0, DEFAULT_FEE_BPS);
        ix.accounts.extend_from_slice(&[
            AccountMeta::new(mint_a, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(vault_b, false),
        ]);

        let result = ctx
            .execute_instruction(ix, &[&actors.pool_admin, &lp_mint])
            .unwrap();
        result.assert_anchor_error("InvalidAmplification");
    }

    #[test]
    fn initialize_pool_rejects_fee_above_max() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let lp_mint = Keypair::new();
        let pool = derive_pool_address(lp_mint.pubkey());

        let mint_a = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();

        let vault_a = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_a)
            .owner(&pool)
            .send()
            .unwrap();
        let vault_b = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_b)
            .owner(&pool)
            .send()
            .unwrap();

        let mut ix = build_initialize_ix(
            &mut ctx,
            &actors.pool_admin,
            &lp_mint,
            DEFAULT_AMPLIFICATION,
            stableswap::MAX_FEE_BPS + 1,
        );
        ix.accounts.extend_from_slice(&[
            AccountMeta::new(mint_a, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(vault_b, false),
        ]);

        let result = ctx
            .execute_instruction(ix, &[&actors.pool_admin, &lp_mint])
            .unwrap();
        result.assert_anchor_error("InvalidFee");
    }

    #[test]
    fn initialize_pool_rejects_odd_remaining_accounts() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let lp_mint = Keypair::new();
        let pool = derive_pool_address(lp_mint.pubkey());

        let mint_a = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();

        let vault_a = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_a)
            .owner(&pool)
            .send()
            .unwrap();

        let mut ix = build_initialize_ix(
            &mut ctx,
            &actors.pool_admin,
            &lp_mint,
            DEFAULT_AMPLIFICATION,
            DEFAULT_FEE_BPS,
        );
        ix.accounts.extend_from_slice(&[
            AccountMeta::new(mint_a, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(mint_b, false),
        ]);

        let result = ctx
            .execute_instruction(ix, &[&actors.pool_admin, &lp_mint])
            .unwrap();
        result.assert_anchor_error("InvalidRemainingAccounts");
    }

    #[test]
    fn initialize_pool_rejects_duplicate_mints() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let lp_mint = Keypair::new();
        let pool = derive_pool_address(lp_mint.pubkey());

        let mint_a = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();

        let vault_a = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_a)
            .owner(&pool)
            .send()
            .unwrap();

        let mut ix = build_initialize_ix(
            &mut ctx,
            &actors.pool_admin,
            &lp_mint,
            DEFAULT_AMPLIFICATION,
            DEFAULT_FEE_BPS,
        );
        ix.accounts.extend_from_slice(&[
            AccountMeta::new(mint_a, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(vault_a, false),
        ]);

        let result = ctx
            .execute_instruction(ix, &[&actors.pool_admin, &lp_mint])
            .unwrap();
        result.assert_anchor_error("DuplicateMint");
    }

    #[test]
    fn initialize_pool_rejects_wrong_vault_address() {
        let mut ctx = setup();
        let actors = create_test_actors(&mut ctx);
        let lp_mint = Keypair::new();
        let pool = derive_pool_address(lp_mint.pubkey());

        let mint_a = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();
        let mint_b = CreateMint::new(&mut ctx.svm, &actors.pool_admin)
            .authority(&actors.pool_admin.pubkey())
            .decimals(6)
            .send()
            .unwrap();

        let vault_a = CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_a)
            .owner(&pool)
            .send()
            .unwrap();
        let wrong_vault_b =
            CreateAssociatedTokenAccount::new(&mut ctx.svm, &actors.pool_admin, &mint_b)
                .owner(&actors.pool_admin.pubkey())
                .send()
                .unwrap();

        let mut ix = build_initialize_ix(
            &mut ctx,
            &actors.pool_admin,
            &lp_mint,
            DEFAULT_AMPLIFICATION,
            DEFAULT_FEE_BPS,
        );
        ix.accounts.extend_from_slice(&[
            AccountMeta::new(mint_a, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(wrong_vault_b, false),
        ]);

        let result = ctx
            .execute_instruction(ix, &[&actors.pool_admin, &lp_mint])
            .unwrap();
        result.assert_anchor_error("InvalidVault");
    }
}
