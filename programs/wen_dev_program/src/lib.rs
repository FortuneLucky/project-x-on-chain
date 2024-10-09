pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
use instructions::*;
pub use state::*;

declare_id!("8cbUjpy6GfkincGpYHnstJbAwxQWsqJNLyNS2xuR5uXi");

#[program]
pub mod project_on_chain {
    use super::*;

    /// Initiazlize a swap pool
    pub fn proxy_initialize(
        ctx: Context<ProxyInitialize>,
        nonce: u8,
        open_time: u64,
        init_pc_amount: u64,
        init_coin_amount: u64,
    ) -> Result<()> {
        instructions::initialize(ctx, nonce, open_time, init_pc_amount, init_coin_amount)
    }

    /// deposit instruction
    pub fn proxy_deposit(
        ctx: Context<ProxyDeposit>,
        max_coin_amount: u64,
        max_pc_amount: u64,
        base_side: u64,
    ) -> Result<()> {
        instructions::deposit(ctx, max_coin_amount, max_pc_amount, base_side)
    }

    /// withdraw instruction
    pub fn proxy_withdraw(ctx: Context<ProxyWithdraw>, amount: u64) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }

    /// swap_base_in instruction
    pub fn proxy_swap_base_in(
        ctx: Context<ProxySwapBaseIn>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap_base_in(ctx, amount_in, minimum_amount_out)
    }

    /// swap_base_out instruction
    pub fn proxy_swap_base_out(
        ctx: Context<ProxySwapBaseOut>,
        max_amount_in: u64,
        amount_out: u64,
    ) -> Result<()> {
        instructions::swap_base_out(ctx, max_amount_in, amount_out)
    }

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
