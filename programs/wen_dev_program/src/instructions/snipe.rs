use crate::{
    constants::{SNIPE_QUEUE, TOKEN_LAUNCH},
    errors::*,
    state::{LaunchPhase, SnipeConfig, SnipeQueue, TokenLaunch},
};
use anchor_lang::{prelude::*, system_program};
use std::cmp::Ordering;

#[derive(Accounts)]
#[instruction(token: Pubkey)]
pub struct Snipe<'info> {
    #[account(mut)]
    wallet: Signer<'info>,

    #[account(seeds = [TOKEN_LAUNCH.as_bytes(), &token.to_bytes()], bump)]
    token_launch: Account<'info, TokenLaunch>,

    #[account(
        mut,
        seeds = [SNIPE_QUEUE.as_bytes(), &token.to_bytes()],
        bump,
    )]
    snipe_queue: Box<Account<'info, SnipeQueue>>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,
}

pub fn snipe<'info>(
    ctx: Context<'_, '_, '_, 'info, Snipe<'info>>,
    token: Pubkey,
    bid_amount: Option<u64>,
    buy_lamports: Option<u64>,
) -> Result<()> {
    if matches!((bid_amount, buy_lamports), (None, None)) {
        msg!("both bid_amount and buy_lamports cannot be None");
        return Err(NothingToDo.into());
    }

    let wallet = &ctx.accounts.wallet;
    let token_launch = &ctx.accounts.token_launch;
    let snipe_queue = &mut ctx.accounts.snipe_queue;

    token_launch.phase.assert_eq(&LaunchPhase::Presale)?;

    // TODO: charge user for ATA creation + tx fee

    // TODO: check lamport amount against bonding curve
    if let Some(buy_lamports) = buy_lamports {
        // Ensure buy_lamports does not exceed 1% of the token supply
        let token_supply = token_launch.initial_token_max_supply;
        let max_snipe_amount = token_supply / 100;
        if buy_lamports > max_snipe_amount {
            msg!("buy_lamports exceeds 1% of the token supply");
            return Err(BuyLamportsExceedsMaxSnipeAmount.into());
        }
    }

    // see if this is creating a new snipe or updating an existing one
    match snipe_queue
        .snipes
        .iter_mut()
        .find(|snipe_config| snipe_config.wallet == wallet.key())
    {
        // existing snipe, update values
        Some(snipe_config) => {
            let old_balance = snipe_config.bid_amount + snipe_config.buy_lamports;
            if let Some(bid_amount) = bid_amount {
                snipe_config.bid_amount = bid_amount;
            }
            if let Some(buy_lamports) = buy_lamports {
                snipe_config.buy_lamports = buy_lamports;
            }

            // balance adjustments
            let new_balance = snipe_config.bid_amount + snipe_config.buy_lamports;
            let delta = (old_balance as i64) - (new_balance as i64);

            match delta.cmp(&0) {
                // deficit, transfer from user -> snipe queue
                Ordering::Less => {
                    system_program::transfer(
                        CpiContext::new(
                            ctx.accounts.system_program.to_account_info(),
                            system_program::Transfer {
                                from: wallet.to_account_info(),
                                to: snipe_queue.to_account_info(),
                            },
                        ),
                        -delta as u64,
                    )?;
                }
                // excess, transfer from snipe queue -> user
                Ordering::Greater => {
                    system_program::transfer(
                        CpiContext::new_with_signer(
                            ctx.accounts.system_program.to_account_info(),
                            system_program::Transfer {
                                from: snipe_queue.to_account_info(),
                                to: wallet.to_account_info(),
                            },
                            &[&[
                                SNIPE_QUEUE.as_bytes(),
                                &token.to_bytes(),
                                &[ctx.bumps.snipe_queue],
                            ]],
                        ),
                        delta as u64,
                    )?;
                }
                // equal, no change
                Ordering::Equal => {}
            }
        }

        // new snipe
        None => {
            let snipe_config = match (bid_amount, buy_lamports) {
                (Some(bid_amount), Some(buy_lamports)) => SnipeConfig {
                    wallet: wallet.key(),
                    bid_amount,
                    buy_lamports,
                    token_amount: 0,
                    processed: false,
                },
                _ => {
                    println!(
                        "both bid_amount and buy_lamports must be specified to create sniper\n\
                        got bid_amount = {bid_amount:?}; buy_lamports = {buy_lamports:?}"
                    );
                    return Err(MissingValueToCreateSniper.into());
                }
            };

            // increase account length
            let snipe_queue_account_info = snipe_queue.to_account_info();
            let new_snipe_queue_len = snipe_queue_account_info.data_len() + SnipeConfig::DATA_LEN;
            let new_snipe_queue_cost = Rent::get()?.minimum_balance(new_snipe_queue_len);

            let lamport_delta = new_snipe_queue_cost.saturating_sub(snipe_queue.get_lamports());
            let transfer_amount =
                snipe_config.bid_amount + snipe_config.buy_lamports + lamport_delta;
            system_program::transfer(
                CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    system_program::Transfer {
                        from: wallet.to_account_info(),
                        to: snipe_queue_account_info.clone(),
                    },
                ),
                transfer_amount,
            )?;
            snipe_queue_account_info.realloc(new_snipe_queue_len, false)?;

            // append new snipe config to end of snipe queue
            snipe_queue.snipes.push(snipe_config);
        }
    }

    Ok(())
}
