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
#[cfg(any(test, browser))] mod test_harness;
#[cfg(test)] mod test;
#[cfg(test)] mod test_2;
pub mod algo;
pub mod auth;
pub mod core;
pub mod math;
pub mod migration;

use fadroma::*;
use fadroma::{message, messages};

use std::cell::RefCell;

use crate::{
    core::*,
    auth::{
        Auth,
        AuthHandle,
        AuthQuery,
        AuthResponse
    },
    algo::{
        Rewards,
        RewardsConfig,
        RewardsInit,
        RewardsHandle,
        RewardsQuery,
        RewardsResponse,
        Pool
    },
    math::*,
    migration::*
};

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Init {
    admin:        Option<HumanAddr>,
    lp_token:     Option<ContractLink<HumanAddr>>,
    reward_token: ContractLink<HumanAddr>,
    viewing_key:  ViewingKey,
    ratio:        Option<Ratio>,
    threshold:    Option<Time>,
    cooldown:     Option<Time>
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
    ReleaseSnip20 {
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    },

    Rewards(RewardsHandle),
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
{
    fn init (&mut self, env: Env, msg: Init) -> StdResult<InitResponse> {

        Auth::init(self, &env, &msg.admin)?;

        let set_vk =  ISnip20::attach(&msg.reward_token)
            .set_viewing_key(&msg.viewing_key.0)?;

        Rewards::init(self, &env, &RewardsInit {
            reward_token: msg.reward_token,
            viewing_key:  msg.viewing_key,
            config: RewardsConfig {
                lp_token:  msg.lp_token,
                ratio:     msg.ratio,
                threshold: msg.threshold,
                cooldown:  msg.cooldown
            }
        })?;

        Ok(InitResponse { log: vec![], messages: vec![set_vk] })

    }

    fn handle (&mut self, env: Env, msg: Handle) -> StdResult<HandleResponse> {
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
                Migration::handle(self, env, msg)
        }
    }

    fn query (&self, msg: Query) -> StdResult<Response> {
        Ok(match msg {
            Query::Auth(msg) =>
                Response::Auth(Auth::query(self, msg)?),
            Query::Rewards(msg) =>
                Response::Rewards(Rewards::query(self, msg)?),
            Query::TokenInfo {} =>
                self.token_info()?,
            Query::Balance { address, key } =>
                self.balance(address, ViewingKey(key))?,
        })
    }

    fn token_info (&self) -> StdResult<Response> {
        let link = self.humanize(self.get(b"/lp_token")?)?;
        let info = ISnip20::attach(&link).query(&self.querier()).token_info()?;
        Ok(Response::TokenInfo {
            name:         format!("Sienna Rewards: {}", info.name),
            symbol:       "SRW".into(),
            decimals:     1,
            total_supply: None
        })
    }

    fn balance (&self, address: HumanAddr, key: ViewingKey) -> StdResult<Response> {
        let address = self.canonize(address)?;
        Auth::check_viewing_key(self, &key, address.as_slice())?;
        let user = Pool::new().user(&self.storage(), address);
        let amount = user.locked.get(&self.storage())?;
        Ok(Response::Balance { amount })
    }
}

impl<S: Storage, A: Api, Q: Querier> Auth<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Rewards<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Migration<S, A, Q> for Extern<S, A, Q> {}
