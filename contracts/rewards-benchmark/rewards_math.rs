//! Continuous rewards are based on a simple tallying function.
//!
//! The current value of the pool/user `liquidity` and pool `age` parameters
//! is based on the previous value + the current value multiplied by the time that the current
//! value has been active.
//!
//! Both parameters' current values depends on the results of the `lock` and `unlock`
//! user transactions. So when one of these is invoked, the "previous" values are updated;
//! then, during read-only queries the new "current" can be computes based on
//! the stored value and the elapsed time since the last such transaction.

use fadroma::scrt::{cosmwasm_std::{StdResult, Uint128}, utils::Uint256};

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;

/// Amount of funds
pub type Amount    = Uint128;

/// Liquidity = amount (u128) * time (u64)
pub type Volume    = Uint256;

/// A ratio represented as tuple (nom, denom)
pub type Ratio     = (Uint128, Uint128);

/// (balance, lifetime, last update)
pub type Status    = (Amount, Volume, Monotonic);

/// Calculate the current total based on the stored total and the time since last update.
pub fn tally (
    total_before_last_update: Volume,
    time_updated_last_update: Monotonic,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    total_before_last_update + Volume::from(value_after_last_update)
        .multiply_ratio(time_updated_last_update, 1u128)?
}
