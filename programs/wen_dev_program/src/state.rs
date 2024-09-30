use crate::errors::*;
use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use core::{cmp::Ordering, fmt::Debug};

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub fee_wallet: Pubkey,

    pub platform_buy_fee_bps: u16,
    pub platform_sell_fee_bps: u16,
    pub pegasus_buy_fee_bps: u16,
    pub pegasus_sell_fee_bps: u16,

    pub lamport_amount_config: AmountConfig<u64>,
    pub token_supply_config: AmountConfig<u64>,
    pub token_decimals_config: AmountConfig<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AmountConfig<T: PartialEq + PartialOrd + Debug> {
    Range { min: Option<T>, max: Option<T> },
    Enum(Vec<T>),
}

impl<T: PartialEq + PartialOrd + Debug> AmountConfig<T> {
    pub fn validate(&self, value: &T) -> Result<()> {
        match self {
            Self::Range { min, max } => {
                if let Some(min) = min {
                    if value < min {
                        println!("value {value:?} too small, expected at least {min:?}");
                        return Err(ValueTooSmall.into());
                    }
                }
                if let Some(max) = max {
                    if value > max {
                        println!("value {value:?} too large, expected at most {max:?}");
                        return Err(ValueTooLarge.into());
                    }
                }

                Ok(())
            }
            Self::Enum(options) => {
                if options.contains(value) {
                    Ok(())
                } else {
                    println!("invalid value {value:?}, expected one of: {options:?}");
                    Err(ValueInvalid.into())
                }
            }
        }
    }
}

#[account]
pub struct TokenLaunch {
    pub token: Pubkey,
    pub creator: Pubkey,
    pub phase: LaunchPhase,

    pub virtual_lamport_reserves: u64,
    pub virtual_token_reserves: u64,
    pub initial_token_max_supply: u64,
}

impl TokenLaunch {
    pub const ACCOUNT_LEN: usize = 8 + 32 + 32 + 1 + 8 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum LaunchPhase {
    Presale,
    ProcessingPresale,
    OpenSale,
    Completed,
} 

impl LaunchPhase {
    pub fn assert_eq(&self, phase: &Self) -> Result<()> {
        if self != phase {
            println!("launch must be in phase {phase:?}, got {self:?}");
            Err(IncorrectLaunchPhase.into())
        } else {
            Ok(())
        }
    }
}

#[account]
pub struct SnipeQueue {
    pub token: Pubkey,
    pub snipes: Vec<SnipeConfig>,
}

impl SnipeQueue {
    pub const MIN_ACCOUNT_LEN: usize = 8 + 32 + 4;
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Eq, Debug)]
pub struct SnipeConfig {
    pub wallet: Pubkey,
    pub bid_amount: u64,
    pub buy_lamports: u64,
    pub token_amount: u64,
    pub processed: bool,
}

impl SnipeConfig {
    pub const DATA_LEN: usize = 32 + 8 + 8 + 8 + 1;
}

impl Ord for SnipeConfig {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.bid_amount, self.buy_lamports).cmp(&(other.bid_amount, other.buy_lamports))
    }
}

impl PartialOrd for SnipeConfig {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SnipeConfig {
    fn eq(&self, other: &Self) -> bool {
        (self.bid_amount, self.buy_lamports) == (other.bid_amount, other.buy_lamports)
    }
}
