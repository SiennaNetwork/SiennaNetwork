use fadroma::*;
use crate::*;

pub fn pool_not_closed <T> () -> StdResult<T> {
    Err(StdError::generic_err("pool not closed"))
}

pub fn no_time_travel <T> () -> StdResult<T> {
    Err(StdError::generic_err("This service does not store history nor permit time travel."))
}

pub fn withdraw (staked: Amount, withdrawn: Amount) -> StdResult<HandleResponse> {
    // User must have enough staked to retrieve
    Err(StdError::generic_err(format!(
        "not enough staked ({} < {})", staked, withdrawn
    )))
}

pub fn withdraw_fatal (staked: Amount, withdrawn: Amount) -> StdResult<HandleResponse> {
    // If pool does not have enough lp tokens then something has gone badly wrong
    Err(StdError::generic_err(format!(
        "FATAL: not enough tokens in pool ({} < {})", staked, withdrawn
    )))
}

pub fn claim_bonding (bonding: Duration) -> StdResult<HandleResponse> {
    Err(StdError::generic_err(format!(
        "Stake tokens for {} more seconds to be eligible for rewards.",
        bonding
    )))
}

pub fn claim_pool_empty () -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "This pool is currently empty. \
        However, liquidity shares continue to accumulate."
    ))
}

pub fn claim_zero_claimable () -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "You have already claimed your exact share of the rewards."
    ))
}
