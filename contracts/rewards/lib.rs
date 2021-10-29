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

pub mod algo; #[cfg(test)] mod algo_test;
pub mod auth; #[cfg(test)] mod auth_test;
pub mod migration;

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
//#[cfg(test)] #[macro_use] extern crate kukumba;

use fadroma::*;

use crate::{
    auth::{
        Auth,
        AuthHandle,
        AuthQuery,
        AuthResponse
    },
    algo::{*, RewardsResponse},
    migration::*
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
        let mut messages = vec![];
        if let Some(set_vk) = Rewards::init(self, &env, msg.config)? {
            messages.push(set_vk);
        }
        Ok(InitResponse { messages, log: vec![] })
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
        let link = self.humanize(
            self.get(b"/lp_token")?.ok_or(StdError::generic_err("no lp token"))?
        )?;
        let info = ISnip20::attach(link).query_token_info(self.querier())?;
        Ok(Response::TokenInfo {
            name:         format!("Sienna Rewards: {}", info.name),
            symbol:       "SRW".into(),
            decimals:     1,
            total_supply: None
        })
    }

    fn balance (&self, address: HumanAddr, key: ViewingKey) -> StdResult<Response> {
        let id = self.canonize(address)?;
        Auth::check_viewing_key(self, &key, id.as_slice())?;
        Ok(Response::Balance {
            amount: self.get_ns(algo::user::LOCKED, id.as_slice())?.unwrap_or(Amount::zero())
        })
    }
}

impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q> for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Rewards<S, A, Q> for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Auth<S, A, Q> for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Migration<S, A, Q> for Extern<S, A, Q> {}
