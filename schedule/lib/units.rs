//! # Unit definitions
//!
//! * Time
//! * Money
//! * Addresses
//! * Success/failure

// Time
pub type Seconds = u64;
pub type Days    = u128;
pub type Months  = u128;
pub const DAY:   Seconds = 24*60*60;
pub const MONTH: Seconds = 30*DAY;

// Money
pub use cosmwasm_std::Uint128;
pub const ONE_SIENNA: u128 = 1000000000000000000u128;

// Address
pub use cosmwasm_std::HumanAddr;

// Success/failure
pub use cosmwasm_std::{StdResult, StdError};
