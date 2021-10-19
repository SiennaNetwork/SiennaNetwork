use fadroma::scrt::cosmwasm_std::{StdResult, StdError};

pub fn missing_reward_token <T> () -> StdResult<T> {
    Err(StdError::generic_err("missing reward token"))
}

pub fn missing_lp_token <T> () -> StdResult<T> {
    Err(StdError::generic_err("missing liquidity provision token"))
}

pub fn missing_self_reference <T> () -> StdResult<T> {
    Err(StdError::generic_err("missing self reference"))
}

pub fn missing_viewing_key <T> () -> StdResult<T> {
    Err(StdError::generic_err("missing reward token viewing key"))
}
