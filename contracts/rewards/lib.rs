#[cfg(test)] #[macro_use] extern crate prettytable;
#[cfg(test)] mod test;
pub mod algo;
pub mod auth;
pub mod drain;
pub mod errors;
pub mod keplr;
pub mod migration;
#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
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
impl Init {
    fn init <S, A, Q, C> (self, core: &mut C, env: Env) -> StdResult<InitResponse> where
        S: Storage, A: Api, Q: Querier, C: Contract<S, A, Q>
    {
        Auth::init(core, &env, &self.admin)?;
        Ok(InitResponse {
            messages: Rewards::init(core, &env, self.config)?,
            log:      vec![]
        })
    }
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum Handle {
    Auth(AuthHandle),
    CreateViewingKey { entropy: String, padding: Option<String> },
    SetViewingKey { key: String, padding: Option<String> },
    Migration(MigrationHandle),
    Rewards(RewardsHandle),
    Drain { snip20: ContractLink<HumanAddr>, recipient: Option<HumanAddr>, key: String },
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for Handle where
    S: Storage, A: Api, Q: Querier, C: Contract<S, A, Q>
{
    fn dispatch_handle (self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        match self {
            Handle::Auth(msg) =>
                Auth::handle(core, env, msg),
            Handle::CreateViewingKey { entropy, padding } =>
                Auth::handle(core, env, AuthHandle::CreateViewingKey { entropy, padding }),
            Handle::SetViewingKey { key, padding } =>
                Auth::handle(core, env, AuthHandle::SetViewingKey { key, padding }),
            Handle::Rewards(msg) =>
                Rewards::handle(core, env, msg),
            Handle::Migration(msg) =>
                Migration::handle(core, env, msg),
            Handle::Drain { snip20, recipient, key } =>
                Drain::drain(core, env, snip20, recipient, key)
        }
    }
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
impl<S, A, Q, C> QueryDispatch<S, A, Q, C, Response> for Query where
    S: Storage, A: Api, Q: Querier, C: Contract<S, A, Q>
{
    fn dispatch_query (self, core: &C) -> StdResult<Response> {
        Ok(match self {
            Query::Auth(msg) =>
                Response::Auth(Auth::query(core, msg)?),
            Query::Rewards(msg) =>
                Response::Rewards(Rewards::query(core, msg)?),
            Query::TokenInfo {} =>
                KeplrCompat::token_info(core)?,
            Query::Balance { address, key } =>
                KeplrCompat::balance(core, address, ViewingKey(key))?
        })
    }
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

impl<S: Storage, A: Api, Q: Querier> Auth<S, A, Q>        for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Rewards<S, A, Q>     for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> KeplrCompat<S, A, Q> for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Migration<S, A, Q>   for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Drain<S, A, Q>       for Extern<S, A, Q> {}
impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q>    for Extern<S, A, Q> {}

pub trait Contract<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
    + Rewards<S, A, Q>
    + Migration<S, A, Q>
    + KeplrCompat<S, A, Q>
    + Drain<S, A, Q>
    + Sized
{
    fn init (&mut self, env: Env, msg: Init)
        -> StdResult<InitResponse>   { msg.init(self, env) }
    fn handle (&mut self, env: Env, msg: Handle)
        -> StdResult<HandleResponse> { msg.dispatch_handle(self, env) }
    fn query (&self, msg: Query)
        -> StdResult<Response>       { msg.dispatch_query(self) }
}

pub fn init <S: Storage, A: Api, Q: Querier> (deps: &mut Extern<S, A, Q>, env: Env, msg: Init)
    -> StdResult<InitResponse>   { Contract::init(deps, env, msg) }
pub fn handle <S: Storage, A: Api, Q: Querier> (deps: &mut Extern<S, A, Q>, env: Env, msg: Handle)
    -> StdResult<HandleResponse> { Contract::handle(deps, env, msg) }
pub fn query <S: Storage, A: Api, Q: Querier> (deps: &Extern<S, A, Q>, msg: Query)
    -> StdResult<Binary>         { to_binary(&Contract::query(deps, msg)?) }
