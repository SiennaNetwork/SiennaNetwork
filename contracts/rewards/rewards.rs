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
//pub mod rewards_admin;
pub mod rewards_api;
//pub mod rewards_contract;
//pub mod rewards_field;
pub mod rewards_math;
//pub mod rewards_pool;
//pub mod rewards_user;
pub mod rewards_vk;

use std::{rc::Rc, cell::RefCell};

use crate::{
    rewards_api::*,
    //rewards_contract::*,
};

use fadroma::scrt::cosmwasm_std::*;
use fadroma::scrt::callback::ContractInstance as ContractLink;

pub fn init <S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    Contract::init(deps, &env, &msg)
}

pub fn handle <S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    Contract::handle(deps, &env, &msg)
}

pub fn query <S: Storage, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>,
    msg:  Query
) -> StdResult<Binary> {
    let response = Contract::query(deps, &msg)?;
    to_binary(&response)
}

pub trait ExternHook<S: Storage, A: Api, Q: Querier> {
    fn storage (self) -> S;
    fn api     (self) -> A;
    fn querier (self) -> Q;
}

impl<S: Storage, A: Api, Q: Querier> ExternHook<S, A, Q>
for Extern<S, A, Q> {
    fn storage (self) -> S { self.storage }
    fn api     (self) -> A { self.api }
    fn querier (self) -> Q { self.querier }
}

pub trait Contract {
    fn init   (&mut self, env: &Env, msg: &Init)   -> StdResult<InitResponse>;
    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse>;
    fn query  (&self, msg: &Query)                 -> StdResult<Binary>;
}

impl<S: Storage, A: Api, Q: Querier> Contract
for Extern<S, A, Q> {
    fn init   (&mut self, env: &Env, msg: &Init)   -> StdResult<InitResponse> {
        SingleAdminAuthentication::init(self, env, msg);
        self.self_link.set(&ContractLink {
            address:   self.env.contract.address.clone(),
            code_hash: self.env.contract_code_hash.clone()
        }.canonize(&self.deps.api)?);
        let admin = msg.admin.unwrap_or(self.env.message.sender.clone());
        self.admin.set(&self.deps.api.canonical_address(&admin)?)?;
        self.reward_token.set(&msg.reward_token.canonize(&self.deps.api)?);
        self.viewing_key.set(&msg.viewing_key)?;
        if let Some(lp_token) = msg.lp_token {
            self.save_lp_token(&lp_token)?;
        }
        self.save_initial_pool_config(&msg)?;
        Ok(InitResponse {
            log:      vec![],
            messages: vec![
                ISnip20::attach(&msg.reward_token).set_viewing_key(&msg.viewing_key.0)?
            ],
        })
        Err(StdError::generic_err("init"))
    }
    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse> {
        if let Some(response) = SingleAdminAuthentication::handle(self, env, msg) {
            return response
        } else if let Some(response) = ViewingKeyAuthentication::handle(self, env, msg) {
            return response
        } else {
            Err(StdError::generic_err("init"))
        }
    }
    fn query  (&self, msg: &Query)            -> StdResult<Binary> {
        Err(StdError::generic_err("init"))
    }
}

pub trait SingleAdminAuthentication {
    fn init   (&mut self, env: &Env, msg: &Init)   -> StdResult<InitResponse>;
    fn handle (&mut self, env: &Env, msg: &Handle) -> Option<StdResult<HandleResponse>>;
    fn query  (&self, msg: &Query)            -> Option<StdResult<Binary>>;
}

impl<S: Storage, A: Api, Q: Querier> SingleAdminAuthentication
for Extern<S, A, Q> {
    fn init   (&mut self, env: &Env, msg: &Init)   -> StdResult<InitResponse> {
        Err(StdError::generic_err("not implemented"))
    }
    fn handle (&mut self, env: &Env, msg: &Handle) -> Option<StdResult<HandleResponse>> {
        Some(Err(StdError::generic_err("not implemented")))
    }
    fn query  (&self, msg: &Query)            -> Option<StdResult<Binary>> {
        Some(Err(StdError::generic_err("not implemented")))
    }
}

pub trait ViewingKeyAuthentication {
    fn handle (&mut self, env: &Env, msg: &Handle) -> Option<StdResult<HandleResponse>>;
    fn query  (&self, msg: &Query)            -> Option<StdResult<Binary>>;
}

impl<S: Storage, A: Api, Q: Querier> ViewingKeyAuthentication
for Extern<S, A, Q> {
    fn handle (&mut self, env: &Env, msg: &Handle) -> Option<StdResult<HandleResponse>> {
        Some(Err(StdError::generic_err("not implemented")))
    }
    fn query  (&self, msg: &Query)            -> Option<StdResult<Binary>> {
        Some(Err(StdError::generic_err("not implemented")))
    }
}
