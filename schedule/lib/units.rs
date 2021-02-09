pub use cosmwasm_std::{Uint128, HumanAddr};

// Time
pub type Seconds    = u64;
pub type Days       = u64;
pub type Months     = u64;
pub const DAY:   Seconds = 24*60*60;
pub const MONTH: Seconds = 30*DAY;

// Money
pub type Percentage = u64;
pub type Amount     = u128;
pub const ONE_SIENNA: Amount = 1000000000000000000u128;
