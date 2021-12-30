use fadroma::*;
use crate::*;

pub fn invalid_epoch_number <T> (epoch: Moment, next_epoch: Moment) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "The current epoch is {}. The 'next_epoch' field must be set to {} instead of {}.",
        epoch,
        epoch + 1,
        next_epoch
    )))
}

pub fn no_time_travel <T> (code: u64) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "This service does not store history nor permit time travel. ({})",
        code
    )))
}

pub fn pool_not_closed <T> () -> StdResult<T> {
    Err(StdError::generic_err("The pool must be permanently closed before performing this operation."))
}

pub fn withdraw <T> (staked: Amount, withdrawn: Amount) -> StdResult<T> {
    // User must have enough staked to retrieve
    Err(StdError::generic_err(format!(
        "not enough staked ({} < {})", staked, withdrawn
    )))
}

pub fn withdraw_fatal <T> (staked: Amount, withdrawn: Amount) -> StdResult<T> {
    // If pool does not have enough lp tokens then something has gone badly wrong
    Err(StdError::generic_err(format!(
        "FATAL: not enough tokens in pool ({} < {})", staked, withdrawn
    )))
}

pub fn claim_bonding <T> (bonding: Duration) -> StdResult<T> {
    Err(StdError::generic_err(format!(
        "Stake tokens for {} more seconds to be eligible for rewards.",
        bonding
    )))
}

pub fn claim_pool_empty <T> () -> StdResult<T> {
    Err(StdError::generic_err(
        "This pool is currently empty. \
        However, liquidity shares continue to accumulate."
    ))
}

pub fn claim_zero_claimable <T> () -> StdResult<T> {
    Err(StdError::generic_err(
        "You have already claimed your exact share of the rewards."
    ))
}

pub fn export_state_miscalled <T> () -> StdResult<T> {
    Err(StdError::generic_err("This handler must be called internally."))
}

pub fn immigration_disallowed <T> () -> StdResult<T> {
    Err(StdError::generic_err("Migration to this contract is not enabled."))
}

pub fn emigration_disallowed <T> () -> StdResult<T> {
    Err(StdError::generic_err("Migration from this contract is not enabled."))
}
