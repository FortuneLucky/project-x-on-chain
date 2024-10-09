use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, TokenAccount, Transfer, Token, Mint}
};
use solana_program::{program::invoke_signed, system_instruction};

use crate::constants::{
    SOL_VAULT,
    TOKEN_LAUNCH,
    CONFIG
};

use crate::state::{
    TokenLaunch,
    LaunchPhase,
    Config
};
use crate::errors::*;

#[derive(Accounts)]
pub struct Migrate<'info> {
    // Account that controls the migration process (i.e., admin)
    #[account(mut)]
    authority: Signer<'info>,

    /// CHECK: initialization handled in instruction
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    config: Account<'info, Config>,

    #[account(mut)]
    token: Box<Account<'info, Mint>>,

    /// CHECK: created in instruction
    #[account(
        mut,
        seeds = [
            &token_launch.key().to_bytes(),
            &anchor_spl::associated_token::ID.to_bytes(),
            &token.key().to_bytes(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    launch_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [TOKEN_LAUNCH.as_bytes(), &token.key().to_bytes()],
        bump
    )]
    token_launch: Box<Account<'info, TokenLaunch>>,

    // The account holding the SOL from the bonding curve
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        seeds = [SOL_VAULT.as_bytes(), &token.key().to_bytes()],
        bump
    )]
    pub bonding_sol_vault: AccountInfo<'info>,

    // Special wallet to receive the withdrawn tokens
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token,
        associated_token::authority = special_sol_wallet,
    )]
    pub special_token_wallet: Account<'info, TokenAccount>,

    // Special wallet to receive the withdrawn SOL
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut, constraint = special_sol_wallet.key() == config.fee_wallet)]
    pub special_sol_wallet: AccountInfo<'info>,

    // Token program account
    pub token_program: Program<'info, Token>,
    // AssociatedToken program account
    pub associated_token_program: Program<'info, AssociatedToken>,
    // System program account (for transferring SOL)
    pub system_program: Program<'info, System>,
}

pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
    let accts = ctx.accounts;
    require!(accts.token_launch.phase == LaunchPhase::Completed, WenDevError::NotCompleted);

    // Transfer token from launch token account to special token wallet
    let (_, bump) = Pubkey::find_program_address(&[TOKEN_LAUNCH.as_bytes(), &accts.token.key().to_bytes()], ctx.program_id);

    let vault_seeds = &[TOKEN_LAUNCH.as_bytes(), &accts.token.key().to_bytes(), &[bump]];
    let signer = &[&vault_seeds[..]];

    let token_balance = accts.launch_token_account.amount;

    token::transfer(
        CpiContext::new_with_signer(
            accts.token_program.to_account_info(),
            Transfer {
                from: accts.launch_token_account.to_account_info(),
                to: accts.special_token_wallet.to_account_info(),
                authority: accts.authority.to_account_info(),
            },
            signer,
        ),
        token_balance
    )?;

    // 2. Transfer SOL to special wallet
    let (_, sol_bump) = Pubkey::find_program_address(&[SOL_VAULT.as_bytes(), &accts.token.key().to_bytes()], ctx.program_id);
    let sol_vault_seeds = &[SOL_VAULT.as_bytes(), &accts.token.key().to_bytes(), &[sol_bump]];
    let sol_signer = &[&sol_vault_seeds[..]];

    // Use invoke_signed to send the instruction with the signer seeds
    invoke_signed(
        &system_instruction::transfer(&accts.bonding_sol_vault.key(), &accts.special_sol_wallet.key(), accts.bonding_sol_vault.lamports()),
        &[
            accts.bonding_sol_vault.to_account_info().clone(),
            accts.special_sol_wallet.to_account_info().clone(),
            accts.system_program.to_account_info().clone(),
        ],
        sol_signer,
    )?;

    Ok(())
}
