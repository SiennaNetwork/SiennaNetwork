#[macro_use] extern crate fadroma;
pub use fadroma::*;

pub mod algo; use crate::algo::{*, RewardsResponse};
pub mod auth; use crate::auth::{*, Auth};
pub mod drain; use crate::drain::Drain;
pub mod errors;
pub mod keplr; use crate::keplr::*;
pub mod migration; use crate::migration::*;

#[cfg(test)] #[macro_use] extern crate prettytable;
#[cfg(test)] mod test;

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
#[cfg(all(feature="browser",target_arch="wasm32"))] mod wasm { fadroma::bind_js!(super); }

pub fn init <S: Storage, A: Api, Q: Querier> (deps: &mut Extern<S, A, Q>, env: Env, msg: Init)
    -> StdResult<InitResponse>   { Contract::init(deps, env, msg) }
pub fn handle <S: Storage, A: Api, Q: Querier> (deps: &mut Extern<S, A, Q>, env: Env, msg: Handle)
    -> StdResult<HandleResponse> { Contract::handle(deps, env, msg) }
pub fn query <S: Storage, A: Api, Q: Querier> (deps: &Extern<S, A, Q>, msg: Query)
    -> StdResult<Binary>         { to_binary(&Contract::query(deps, msg)?) }

pub trait Contract<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
    + Rewards<S, A, Q>
    + MigrationImport<S, A, Q>
    + MigrationExport<S, A, Q>
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
    MigrationImport(MigrationImportHandle),
    MigrationExport(MigrationExportHandle),
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
            Handle::MigrationImport(msg) =>
                MigrationImport::handle(core, env, msg),
            Handle::MigrationExport(msg) =>
                MigrationExport::handle(core, env, msg),
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

/// Implement the feature traits on the base struct.
/// Reused in test harness (test/mod.rs), where the same
/// traits need to be implemented on a clonable MockExtern
#[macro_export] macro_rules! compose {
    ($Core:ty) => {

        impl<S: Storage, A: Api, Q: Querier> crate::Contract<S, A, Q> for $Core {}

        impl<S: Storage, A: Api, Q: Querier> crate::auth::Auth<S, A, Q> for $Core {}

        impl<S: Storage, A: Api, Q: Querier> crate::algo::Rewards<S, A, Q> for $Core {}

        impl<S: Storage, A: Api, Q: Querier> crate::keplr::KeplrCompat<S, A, Q> for $Core {
            fn token_info (&self) -> StdResult<Response> {
                let info = RewardsConfig::lp_token(self)?.query_token_info(self.querier())?;
                Ok(Response::TokenInfo {
                    name:         format!("Sienna Rewards: {}", info.name),
                    symbol:       "SRW".into(),
                    decimals:     1,
                    total_supply: None
                })
            }
            fn balance (&self, address: HumanAddr, key: ViewingKey) -> StdResult<Response> {
                let id = self.canonize(address)?;
                Auth::check_vk(self, &key, id.as_slice())?;
                let amount = self.get_ns(crate::algo::Account::STAKED, id.as_slice())?;
                Ok(Response::Balance { amount: amount.unwrap_or(Amount::zero()) })
            }
        }

        impl<S: Storage, A: Api, Q: Querier> crate::migration::MigrationExport<S, A, Q> for $Core {
            fn export_state (&mut self, _env: Env, addr: HumanAddr) -> StdResult<Binary> {
                to_binary(&crate::algo::Account::export(self, addr)?)
            }
        }

        impl<S: Storage, A: Api, Q: Querier> crate::migration::MigrationImport<S, A, Q> for $Core {
            fn import_state (&mut self, _env: Env, data: Binary) -> StdResult<()> {
                crate::algo::Account::import(self, from_slice(&data.as_slice())?)
            }
        }

        impl<S: Storage, A: Api, Q: Querier> crate::drain::Drain<S, A, Q> for $Core {}

    };
}

compose!(Extern<S, A, Q>);
