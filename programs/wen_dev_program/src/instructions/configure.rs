use crate::{constants::CONFIG, errors::*, state::Config};
use anchor_lang::{prelude::*, system_program, Discriminator};

#[derive(Accounts)]
pub struct Configure<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: initialization handled in instruction
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    config: UncheckedAccount<'info>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,
}

pub fn configure<'info>(
    ctx: Context<'_, '_, '_, 'info, Configure<'info>>,
    new_config: Config,
) -> Result<()> {
    let payer = &ctx.accounts.payer;
    let config = &ctx.accounts.config;

    let serialized_config = [&Config::DISCRIMINATOR, new_config.try_to_vec()?.as_slice()].concat();
    let serialized_config_len = serialized_config.len();
    let config_cost = Rent::get()?.minimum_balance(serialized_config_len);

    if config.owner != &crate::ID {
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::CreateAccount {
                from: payer.to_account_info(),
                to: config.to_account_info(),
            },
        );
        system_program::create_account(
            cpi_context.with_signer(&[&[CONFIG.as_bytes(), &[ctx.bumps.config]]]),
            config_cost,
            serialized_config_len as u64,
            &crate::ID,
        )?;
    } else {
        let config = Config::deserialize(&mut &(**config.try_borrow_data()?))?;
        if config.authority != payer.key() {
            return Err(IncorrectAuthority.into());
        }
    }

    let lamport_delta = (config_cost as i64) - (config.lamports() as i64);
    if lamport_delta > 0 {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: payer.to_account_info(),
                    to: config.to_account_info(),
                },
            ),
            lamport_delta as u64,
        )?;
        config.realloc(serialized_config_len, false)?;
    }

    (config.try_borrow_mut_data()?[..serialized_config_len])
        .copy_from_slice(serialized_config.as_slice());

    Ok(())
}
