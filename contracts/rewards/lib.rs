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

#[cfg(test)] #[macro_use] extern crate prettytable;
#[cfg(test)] mod test;
pub mod algo;
pub mod auth;
pub mod drain;
pub mod errors;
pub mod keplr;
pub mod migration;

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
//#[cfg(test)] #[macro_use] extern crate kukumba;

use fadroma::*;
use crate::{
    algo::{*, RewardsResponse},
    auth::{*, Auth},
    drain::*,
    keplr::*,
    migration::*,
};

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Init {
    admin:  Option<HumanAddr>,
    config: RewardsConfig
}

pub fn init <S: Storage + AsRef<S> + AsMut<S>, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    Contract::init(deps, env, msg)
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum Handle {
    Auth(AuthHandle),
    CreateViewingKey {
        entropy: String,
        padding: Option<String>
    },
    SetViewingKey {
        key:     String,
        padding: Option<String>
    },

    Migration(MigrationHandle),

    Rewards(RewardsHandle),

    Drain {
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    },
}

pub fn handle <S: Storage + AsRef<S> + AsMut<S>, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    Contract::handle(deps, env, msg)
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum Query {
    Auth(AuthQuery),
    Rewards(RewardsQuery),

    /// For Keplr integration
    TokenInfo {},
    /// For Keplr integration
    Balance { address: HumanAddr, key: String }
}

pub fn query <S: Storage + AsRef<S>, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>,
    msg:  Query
) -> StdResult<Binary> {
    to_binary(&Contract::query(deps, msg)?)
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum Response {
    Auth(AuthResponse),
    Rewards(RewardsResponse),

    /// Keplr integration
    TokenInfo {
        name:         String,
        symbol:       String,
        decimals:     u8,
        total_supply: Option<Amount>
    },

    /// Keplr integration
    Balance {
        amount: Amount
    }
}

pub trait Contract<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
    + Rewards<S, A, Q>
    + Migration<S, A, Q>
    + KeplrCompat<S, A, Q>
    + Drain<S, A, Q>
{
    fn init (&mut self, env: Env, msg: Init) -> StdResult<InitResponse> where Self: Sized {
        Auth::init(self, &env, &msg.admin)?;
        Ok(InitResponse {
            messages: Rewards::init(self, &env, msg.config)?,
            log:      vec![]
        })
    }

    fn handle (&mut self, env: Env, msg: Handle) -> StdResult<HandleResponse> where Self: Sized {
        match msg {
            Handle::Auth(msg) =>
                Auth::handle(self, env, msg),
            Handle::CreateViewingKey { entropy, padding } =>
                Auth::handle(self, env, AuthHandle::CreateViewingKey { entropy, padding }),
            Handle::SetViewingKey { key, padding } =>
                Auth::handle(self, env, AuthHandle::SetViewingKey { key, padding }),
            Handle::Rewards(msg) =>
                Rewards::handle(self, env, msg),
            Handle::Migration(msg) =>
                Migration::handle(self, env, msg),
            Handle::Drain { snip20, recipient, key } =>
                Drain::drain(self, env, snip20, recipient, key)
        }
    }

    fn query (&self, msg: Query) -> StdResult<Response> where Self: Sized {
        Ok(match msg {
            Query::Auth(msg) =>
                Response::Auth(Auth::query(self, msg)?),
            Query::Rewards(msg) =>
                Response::Rewards(Rewards::query(self, msg)?),
            Query::TokenInfo {} =>
                KeplrCompat::token_info(self)?,
            Query::Balance { address, key } =>
                KeplrCompat::balance(self, address, ViewingKey(key))?
        })
    }
}

impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Auth<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Rewards<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> KeplrCompat<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Migration<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Drain<S, A, Q> for Extern<S, A, Q> {}
