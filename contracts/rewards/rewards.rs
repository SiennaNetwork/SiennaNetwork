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
pub mod rewards_admin;
pub mod rewards_api;
pub mod rewards_contract;
pub mod rewards_field;
pub mod rewards_math;
pub mod rewards_pool;
pub mod rewards_user;
pub mod rewards_vk;

use std::{rc::Rc, cell::RefCell};

use crate::{
    rewards_api::*,
    rewards_contract::*,
    rewards_vk::ViewingKey
};

use fadroma::scrt::{
    cosmwasm_std::*,
    callback::ContractInstance as ContractLink,
    snip20_api::ISnip20
};

pub fn init <S: Storage + AsRef<S> + AsMut<S>, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    RewardPool::init(deps, &env, &msg)
}

pub fn handle <S: Storage + AsRef<S> + AsMut<S>, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    RewardPool::handle(deps, &env, &msg)
}

pub fn query <S: Storage + AsRef<S>, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>,
    msg:  Query
) -> StdResult<Binary> {
    to_binary(&RewardPool::query(deps, &msg)?)
}

pub trait ExternHook<S, A, Q> {
    fn storage (self) -> S;
    fn api     (self) -> A;
    fn querier (self) -> Q;
}

impl<S: Storage, A: Api, Q: Querier> ExternHook<S, A, Q> for Extern<S, A, Q> {
    fn storage (self) -> S { self.storage }
    fn api     (self) -> A { self.api }
    fn querier (self) -> Q { self.querier }
}

pub trait ContractAPI<S, A, Q>: ExternHook<S, A, Q> {
    fn init   (&mut self, env: &Env, msg: &Init) -> StdResult<InitResponse> {
        Err(StdError::generic_err("not implemented"))
    }
    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse> {
        Err(StdError::generic_err("not implemented"))
    }
    fn query  (&self, msg: &Query) -> StdResult<Binary> {
        Err(StdError::generic_err("not implemented"))
    }
}

impl<S: Storage, A: Api, Q: Querier> ContractAPI<S, A, Q> for Extern<S, A, Q> {}

pub trait RewardPool<S: Storage, A: Api, Q: Querier>: ContractAPI<S, A, Q>
    + SelfReference<S, A, Q>
    + SingleAdminAuthentication<S, A, Q>
    + ViewingKeyAuthentication
    + RewardPoolConfiguration
    + RewardPoolDistribution
{
    fn init (&mut self, env: &Env, msg: &Init) -> StdResult<InitResponse> {
        SelfReference::init(self, env, msg)?;
        SingleAdminAuthentication::init(self, env, msg)?;
        RewardPoolConfiguration::init(self, env, msg)?;
        Ok(InitResponse {
            log:      vec![],
            messages: vec![
                ISnip20::attach(&msg.reward_token).set_viewing_key(&msg.viewing_key.0)?
            ],
        })
    }

    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse> {
        if let Some(response) = SingleAdminAuthentication::handle(self, env, msg) {
            return response
        } else if let Some(response) = ViewingKeyAuthentication::handle(self, env, msg) {
            return response
        } else if let Some(response) = RewardPoolConfiguration::handle(self, env, msg) {
            return response
        } else if let Some(response) = RewardPoolDistribution::handle(self, env, msg) {
            return response
        } else {
            match msg {
                Handle::ClosePool { .. } => {},
                Handle::ReleaseSnip20 { .. } => {},
                _  => Err(StdError::generic_err("not implementd"))
            }
        }
    }

    fn query  (&self, msg: &Query) -> StdResult<Binary> {
        match msg {
            Query::Admin {} =>
                self.admin(),
            Query::PoolInfo { at } =>
                self.pool_info(at),
            Query::UserInfo { at, address, key } =>
                self.user_info(at, address, key),
            Query::TokenInfo {} =>
                self.token_info(),
            Query::Balance { address, key } =>
                self.balance(address, key),
        }
    }
}

impl<S: Storage, A: Api, Q: Querier> RewardPool<S, A, Q> for Extern<S, A, Q> {}

pub trait RewardPoolConfiguration {
    fn init (&mut self, env: &Env, msg: &Init) -> StdResult<()> {
        self.set_reward_token(&msg.reward_token)?;
        self.set_own_viewing_key(&msg.viewing_key)?;
        self.set_lp_token(&msg.lp_token)?;
        self.set_initial_pool_config(&msg)?;
        Ok(())
    }

    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse> {
        match msg {
            Handle::SetProvidedToken { .. } => {},
            Handle::ChangeRatio { .. } => {},
            Handle::ChangeThreshold { .. } => {},
            Handle::ChangeCooldown { .. } => {},
            _  => Err(StdError::generic_err("not implementd"))
        }
    }

    fn set_reward_token (&mut self, token: &ContractLink<HumanAddr>) -> StdResult<()> { Ok(()) }
    fn set_own_viewing_key (&mut self, token: &ViewingKey) -> StdResult<()> { Ok(()) }
    fn set_lp_token (&mut self, token: &Option<ContractLink<HumanAddr>>) -> StdResult<()> { Ok(()) }
    fn set_initial_pool_config (&mut self, token: &Init) -> StdResult<()> { Ok(()) }

    fn query  (&self, msg: &Query) -> StdResult<Binary> {
        match msg {
            Query::PoolInfo { at } => self.pool_info(at),
            Query::UserInfo { at, address, key } => self.user_info(at, address, key),
        }
    }
}

impl<S: Storage, A: Api, Q: Querier> RewardPoolConfiguration for Extern<S, A, Q> {}

pub trait RewardPoolDistribution {
    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse> {
        match msg {
            Handle::Lock { .. } => {},
            Handle::Retrieve { .. } => {},
            Handle::Claim { .. } => {},
        }
    }

    fn query  (&self, msg: &Query) -> StdResult<Binary> {
        match msg {
            Query::Admin {} =>
                self.admin(),
            Query::PoolInfo { at } =>
                self.pool_info(at),
            Query::UserInfo { at, address, key } =>
                self.user_info(at, address, key),
            Query::TokenInfo {} =>
                self.token_info(),
            Query::Balance { address, key } =>
                self.balance(address, key),
        }
    }
}

impl<S: Storage, A: Api, Q: Querier> RewardPoolDistribution for Extern<S, A, Q> {}

pub trait SelfReference<S: Storage, A: Api, Q: Querier>: ContractAPI<S, A, Q> {
    fn init (&mut self, env: &Env, msg: &Init) -> StdResult<()> {
        let link = &ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        };
        //self.self_link.set(link.canonize(&self.api())?)
    }
}

impl<S: Storage, A: Api, Q: Querier> SelfReference<S, A, Q> for Extern<S, A, Q> {}

pub trait SingleAdminAuthentication<S, A, Q>: ContractAPI<S, A, Q> {
    fn init (&mut self, env: &Env, msg: &Init) -> StdResult<()> {
        let admin = msg.admin.unwrap_or(env.message.sender.clone());
        self.admin.set(&self.api().canonical_address(&admin)?)
    }
    fn handle (&mut self, env: &Env, msg: &Handle) -> Option<StdResult<HandleResponse>> {
        Some(Err(StdError::generic_err("not implemented")))
    }
    fn query (&self, msg: &Query) -> Option<StdResult<Binary>> {
        Some(Err(StdError::generic_err("not implemented")))
    }
}

impl<S: Storage, A: Api, Q: Querier> SingleAdminAuthentication<S, A, Q> for Extern<S, A, Q> {}

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

//pub trait ExternHook<S: Storage, A: Api, Q: Querier> {
    //fn storage (self) -> S;
    //fn api     (self) -> A;
    //fn querier (self) -> Q;
//}

//impl<S: Storage, A: Api, Q: Querier> ExternHook<S, A, Q>
//for Extern<S, A, Q> {
    //fn storage (self) -> S { self.storage }
    //fn api     (self) -> A { self.api }
    //fn querier (self) -> Q { self.querier }
//}
