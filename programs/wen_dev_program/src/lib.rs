use anchor_lang::prelude::*;

use instructions::*;

pub mod constants;
pub mod errors;
mod instructions;
pub mod state;
use crate::state::Config;

declare_id!("4ubxrDaVYVbP5Qmqdwe43haohnBMEaSY1aCXxxDJQVS7"); // TODO: Update this ID

#[program]
pub mod wen_dev_program {
    use super::*;

    pub fn configure<'info>(
        ctx: Context<'_, '_, '_, 'info, Configure<'info>>,
        new_config: Config,
    ) -> Result<()> {
        instructions::configure(ctx, new_config)
    }

    pub fn launch<'info>(
        ctx: Context<'_, '_, '_, 'info, Launch<'info>>,
        name: String,
        symbol: String,
        uri: String,
        virtual_lamport_reserves: u64,
        token_supply: u64,
        decimals: u8,
    ) -> Result<()> {
        instructions::launch(
            ctx,
            name,
            symbol,
            uri,
            virtual_lamport_reserves,
            token_supply,
            decimals,
        )
    }

    pub fn snipe<'info>(
        ctx: Context<'_, '_, '_, 'info, Snipe<'info>>,
        token: Pubkey,
        bid_amount: Option<u64>,
        buy_lamports: Option<u64>,
    ) -> Result<()> {
        instructions::snipe(ctx, token, bid_amount, buy_lamports)
    }

    pub fn migrate<'info>(
        ctx: Context<'_, '_, '_, 'info, Migrate<'info>>
    ) -> Result<()> {
        instructions::migrate(ctx)
    }
}
