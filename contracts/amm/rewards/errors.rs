use crate::{
    account::Amount,
    time_utils::{Duration, Moment},
};
use fadroma::*;

/// When trying to set the epoch to something other than the next
pub fn invalid_epoch_number<T>(epoch: Moment, next_epoch: Moment) -> StdResult<T> {
    let msg = format!(
        "The current epoch is {}. The 'next_epoch' field must be set to {} instead of {}.",
        epoch,
        epoch + 1,
        next_epoch
    );
    Err(StdError::generic_err(msg))
}

/// When querying for a moment before the last update,
/// or when an accumulator has somehow decreased
pub fn no_time_travel<T>(code: u64) -> StdResult<T> {
    let msg = format!(
        "This service does not store history nor permit time travel. ({})",
        code
    );
    Err(StdError::generic_err(msg))
}

/// Returned if pool is not closed when draining
pub fn pool_not_closed<T>() -> StdResult<T> {
    Err(StdError::generic_err(
        "The pool must be permanently closed before performing this operation.",
    ))
}

/// User must have enough staked to retrieve
pub fn withdraw<T>(staked: Amount, withdrawn: Amount) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "not enough staked ({} < {})",
        staked, withdrawn
    )))
}

/// If pool does not have enough lp tokens then something has gone badly wrong
pub fn withdraw_fatal<T>(staked: Amount, withdrawn: Amount) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "FATAL: not enough tokens in pool ({} < {})",
        staked, withdrawn
    )))
}

/// If user calls claim before their bonding period is over
pub fn claim_bonding<T>(bonding: Duration) -> StdResult<T> {
    let msg = format!(
        "Stake tokens for {} more seconds to be eligible for rewards.",
        bonding
    );
    Err(StdError::generic_err(msg))
}

/// When pool doesn't get funding
pub fn claim_pool_empty<T>() -> StdResult<T> {
    let msg = StdError::generic_err(
        "This pool is currently empty. However, liquidity shares continue to accumulate.",
    );
    Err(msg)
}

/// Unreachable?
pub fn claim_zero_claimable<T>() -> StdResult<T> {
    Err(StdError::generic_err(
        "You have already claimed your exact share of the rewards.",
    ))
}

/// When a user tries to call EmigrationHandle::ExportState directly
pub fn export_state_miscalled<T>() -> StdResult<T> {
    Err(StdError::generic_err(
        "This handler must be called internally.",
    ))
}

/// When a user tries to migrate into a contract that is not whitelisted
pub fn immigration_disallowed<T>() -> StdResult<T> {
    Err(StdError::generic_err(
        "Migration to this contract is not enabled.",
    ))
}

/// When a user tries to migrate from a contract that is not whitelisted
pub fn emigration_disallowed<T>() -> StdResult<T> {
    Err(StdError::generic_err(
        "Migration from this contract is not enabled.",
    ))
}

//when a user tries to run an update on a poll which is expired
pub fn poll_expired<T>() -> StdResult<T> {
    Err(StdError::generic_err(
        "Poll has expired. Can't perform anymore updates. ",
    ))
}

pub fn unstake_disallowed<T>() -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "Unstaking not allowed. Make sure you have no active or created polls. "
    )))
}
pub fn governance_closed<T>(time: Moment, reason: String) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "The governance has been closed. Closed at: {}, reason: {}",
        time, reason
    )))
}
pub fn not_enough_stake_to_vote<T>(balance: Uint128, required: Uint128) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "Your staked balance is too low to vote. Balance: {}, required: {}",
        balance, required
    )))
}
