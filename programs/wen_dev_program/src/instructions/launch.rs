use crate::{
    constants::{CONFIG, METADATA, SNIPE_QUEUE, TOKEN_LAUNCH},
    errors::*,
    state::{Config, LaunchPhase, SnipeQueue, TokenLaunch},
};
use anchor_lang::{prelude::*, solana_program::sysvar::SysvarId, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{mpl_token_metadata::types::DataV2, Metadata},
    token::{spl_token::instruction::AuthorityType, Mint, Token},
};

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct Launch<'info> {
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    config: Box<Account<'info, Config>>,

    #[account(mut)]
    creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        mint::decimals = decimals,
        mint::authority = token_launch.key(),
    )]
    token: Box<Account<'info, Mint>>,

    /// CHECK: passed to token metadata program
    #[account(
        mut,
        seeds = [
            METADATA.as_bytes(),
            &anchor_spl::metadata::ID.to_bytes(),
            &token.key().to_bytes(),
        ],
        bump,
        seeds::program = anchor_spl::metadata::ID
    )]
    token_metadata_account: UncheckedAccount<'info>,

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
    launch_token_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        space = TokenLaunch::ACCOUNT_LEN,
        seeds = [TOKEN_LAUNCH.as_bytes(), &token.key().to_bytes()],
        bump
    )]
    token_launch: Box<Account<'info, TokenLaunch>>,

    #[account(
        init,
        payer = creator,
        space = SnipeQueue::MIN_ACCOUNT_LEN,
        seeds = [SNIPE_QUEUE.as_bytes(), &token.key().to_bytes()],
        bump,
    )]
    snipe_queue: Box<Account<'info, SnipeQueue>>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    rent: Sysvar<'info, Rent>,

    #[account(address = anchor_spl::token::ID)]
    token_program: Program<'info, Token>,

    #[account(address = anchor_spl::associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = anchor_spl::metadata::ID)]
    mpl_token_metadata_program: Program<'info, Metadata>,
}

pub fn launch<'info>(
    ctx: Context<'_, '_, '_, 'info, Launch<'info>>,

    // metadata
    name: String,
    symbol: String,
    uri: String,

    // launch config
    virtual_lamport_reserves: u64,
    token_supply: u64,
    decimals: u8,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let creator = &ctx.accounts.creator;
    let token = &ctx.accounts.token;
    let launch_token_account = &ctx.accounts.launch_token_account;
    let token_launch = &mut ctx.accounts.token_launch;
    let snipe_queue = &mut ctx.accounts.snipe_queue;

    // TODO: verify checks

    let decimal_multiplier = 10u64.pow(decimals as u32);
    let fractional_tokens = token_supply % decimal_multiplier;
    if fractional_tokens != 0 {
        println!("expected whole number of tokens, got fractional tokens: 0.{fractional_tokens}");
        return Err(ValueInvalid.into());
    }

    config
        .lamport_amount_config
        .validate(&virtual_lamport_reserves)?;

    config
        .token_supply_config
        .validate(&(token_supply / decimal_multiplier))?;
    config.token_decimals_config.validate(&decimals)?;

    // create launch
    token_launch.token = token.key();
    token_launch.creator = creator.key();
    token_launch.phase = LaunchPhase::Presale;
    token_launch.virtual_lamport_reserves = virtual_lamport_reserves;
    token_launch.virtual_token_reserves = token_supply;
    token_launch.initial_token_max_supply = token_supply;

    snipe_queue.token = token_launch.token;

    // create launch token account
    anchor_spl::associated_token::create(CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        anchor_spl::associated_token::Create {
            payer: creator.to_account_info(),
            associated_token: launch_token_account.to_account_info(),
            authority: launch_token_account.to_account_info(),
            mint: token.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    // mint tokens
    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: token.to_account_info(),
                to: launch_token_account.to_account_info(),
                authority: launch_token_account.to_account_info(),
            },
            &[&[
                TOKEN_LAUNCH.as_bytes(),
                &token.key().to_bytes(),
                &[ctx.bumps.token_launch],
            ]],
        ),
        token_supply,
    )?;

    // create metadata
    anchor_spl::metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.mpl_token_metadata_program.to_account_info(),
            anchor_spl::metadata::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.token_metadata_account.to_account_info(),
                mint: token.to_account_info(),
                mint_authority: launch_token_account.to_account_info(),
                payer: creator.to_account_info(),
                update_authority: launch_token_account.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[&[
                TOKEN_LAUNCH.as_bytes(),
                &token.key().to_bytes(),
                &[ctx.bumps.token_launch],
            ]],
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false,
        true,
        None,
    )?;

    // relinquish mint authority
    anchor_spl::token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::SetAuthority {
                current_authority: launch_token_account.to_account_info(),
                account_or_mint: token.to_account_info(),
            },
            &[&[
                TOKEN_LAUNCH.as_bytes(),
                &token.key().to_bytes(),
                &[ctx.bumps.token_launch],
            ]],
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    Ok(())
}
