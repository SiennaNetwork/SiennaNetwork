//! Since there is a limited amount of rewards for each day,
//! they need to be distributed among the top liquidity providers.
//!
//! By locking funds, the user starts accruing a lifetime share of the pool
//! which entitles them to an equal percent of the total rewards,
//! which are distributed daily and the user can claim one per day.
//!
//! This lifetime share fluctuates as a result of the other users
//! locking and unlocking amounts of funds for different amounts of time.
//! If it remains constant or increases, users are guaranteed a new reward
//! every day. If they fall behind, they may be able to claim rewards
//! less frequently, and need to lock more tokens to restore their place
//! in the queue.

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
#[cfg(test)] #[macro_use] extern crate kukumba;
#[cfg(any(test, browser))] mod rewards_harness;
#[cfg(test)] mod rewards_test;
#[cfg(test)] mod rewards_test_2;
pub mod rewards_api;
pub mod rewards_contract;
pub mod rewards_math;
pub mod rewards_pool;
pub mod rewards_user;
pub mod rewards_field;

use crate::{
    rewards_api::*,
    rewards_contract::*,
};

use fadroma::scrt::cosmwasm_std::*;

pub fn init <S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    let Extern { storage, api, querier } = deps;
    Contract { storage, api, querier, env }.init(msg)
}

pub fn handle <S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    let Extern { storage, api, querier } = deps;
    Contract { storage, api, querier, env }.handle(msg)
}

pub fn query <S: Storage, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>,
    msg:  Query
) -> StdResult<Binary> {
    let Extern { storage, api, querier } = deps;
    to_binary(&(Contract { storage, api, querier, env: () }.query(msg)?))
}
