use anchor_lang::{
    solana_program::{rent::Rent, sysvar::SysvarId},
    system_program::System,
    Id,
};
use anchor_litesvm::{AccountMeta, AnchorContext, AnchorLiteSVM};
use anchor_spl::{associated_token::AssociatedToken, token::Token};
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use litesvm_utils::TestHelpers;
use solana_address::Address;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub const DEFAULT_AMPLIFICATION: u64 = 100_000;
pub const DEFAULT_FEE_BPS: u16 = 100;

pub struct TestActors {
    pub pool_admin: Keypair,
    pub depositor: Keypair,
    pub swap_user: Keypair,
}

pub struct InitializedPool {
    pub pool: Address,
    pub lp_mint: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub mint_a_vault: Address,
    pub mint_b_vault: Address,
    pub amplification: u64,
    pub fee_bps: u16,
}

pub fn setup() -> AnchorContext {
    let program_id = stableswap::id();
    AnchorLiteSVM::build_with_program(
        program_id,
        include_bytes!("../../../target/deploy/stableswap.so"),
    )
}

pub fn create_funded_account(ctx: &mut AnchorContext) -> Keypair {
    ctx.svm.create_funded_account(1_000_000_000).unwrap()
}

pub fn create_test_actors(ctx: &mut AnchorContext) -> TestActors {
    TestActors {
        pool_admin: create_funded_account(ctx),
        depositor: create_funded_account(ctx),
        swap_user: create_funded_account(ctx),
    }
}

fn derive_pool_address(lp_mint: Address) -> Address {
    Address::find_program_address(&[b"pool", lp_mint.as_ref()], &stableswap::id()).0
}

pub fn initialize_pool_with_two_mints(ctx: &mut AnchorContext, pool_admin: &Keypair) -> InitializedPool {
    let lp_mint = Keypair::new();
    let pool = derive_pool_address(lp_mint.pubkey());

    let mut ix = ctx
        .program()
        .accounts(stableswap::accounts::InitializePool {
            admin: pool_admin.pubkey(),
            associated_token_program: AssociatedToken::id(),
            lp_mint: lp_mint.pubkey(),
            pool,
            rent: Rent::id(),
            system_program: System::id(),
            token_program: Token::id(),
        })
        .args(stableswap::instruction::InitializePool {
            amplification: DEFAULT_AMPLIFICATION,
            fee_bps: DEFAULT_FEE_BPS,
        })
        .instruction()
        .unwrap();

    let mint_a = CreateMint::new(&mut ctx.svm, pool_admin)
        .authority(&pool_admin.pubkey())
        .decimals(6)
        .send()
        .unwrap();
    let mint_b = CreateMint::new(&mut ctx.svm, pool_admin)
        .authority(&pool_admin.pubkey())
        .decimals(6)
        .send()
        .unwrap();

    let mint_a_vault = CreateAssociatedTokenAccount::new(&mut ctx.svm, pool_admin, &mint_a)
        .owner(&pool)
        .send()
        .unwrap();
    let mint_b_vault = CreateAssociatedTokenAccount::new(&mut ctx.svm, pool_admin, &mint_b)
        .owner(&pool)
        .send()
        .unwrap();

    ix.accounts.extend_from_slice(&[
        AccountMeta::new(mint_a, false),
        AccountMeta::new(mint_a_vault, false),
        AccountMeta::new(mint_b, false),
        AccountMeta::new(mint_b_vault, false),
    ]);

    ctx.execute_instruction(ix, &[pool_admin, &lp_mint]).unwrap();

    InitializedPool {
        pool,
        lp_mint: lp_mint.pubkey(),
        mint_a,
        mint_b,
        mint_a_vault,
        mint_b_vault,
        amplification: DEFAULT_AMPLIFICATION,
        fee_bps: DEFAULT_FEE_BPS,
    }
}

pub fn create_ata_for_owner(
    ctx: &mut AnchorContext,
    payer: &Keypair,
    mint: &Address,
    owner: &Keypair,
) -> Address {
    CreateAssociatedTokenAccount::new(&mut ctx.svm, payer, mint)
        .owner(&owner.pubkey())
        .send()
        .unwrap()
}

pub fn mint_to_account(
    ctx: &mut AnchorContext,
    mint: &Address,
    destination: &Address,
    mint_authority: &Keypair,
    amount: u64,
) {
    MintTo::new(&mut ctx.svm, mint_authority, mint, destination, amount)
        .owner(mint_authority)
        .send()
        .unwrap();
}
