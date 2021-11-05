use fadroma::*;
use crate::*;

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

pub fn claim_threshold (threshold: Duration, liquid: Duration) -> StdResult<HandleResponse> {
    Err(StdError::generic_err(format!(
        "You must keep some tokens staked for {} more seconds \
        before you are able to claim for the first time.",
        threshold - liquid
    )))
}

pub fn claim_cooldown (cooldown: Duration) -> StdResult<HandleResponse> {
    Err(StdError::generic_err(format!(
        "You must keep some tokens staked for {} more seconds \
        before you are able to claim again.",
        cooldown
    )))
}

pub fn claim_pool_empty () -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "This pool is currently empty. \
        However, liquidity shares continue to accumulate."
    ))
}

pub fn claim_global_ratio_zero () -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "Rewards from this pool are currently stopped. \
        However, liquidity shares continue to accumulate."
    ))
}

pub fn claim_crowded_out () -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "Your liquidity share has steeply diminished \
        since you last claimed. Lock more tokens to get \
        to the front of the queue faster."
    ))
}

pub fn claim_zero_claimable () -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "You have already claimed your exact share of the rewards."
    ))
}

