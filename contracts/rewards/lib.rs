extern crate fadroma;
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
#[cfg(all(feature="browser",target_arch="wasm32"))] mod wasm { fadroma_bind_js::bind_js!(fadroma, super); }
#[cfg(all(not(feature="browser"),target_arch="wasm32"))] mod wasm {
    use super::{init, handle, query};
    use cosmwasm_std::{do_handle, do_init, do_query, ExternalApi, ExternalQuerier, ExternalStorage};
    #[no_mangle] extern "C" fn init(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_init(&init::<ExternalStorage, ExternalApi, ExternalQuerier>, env_ptr, msg_ptr) }
    #[no_mangle] extern "C" fn handle(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_handle(&handle::<ExternalStorage, ExternalApi, ExternalQuerier>, env_ptr, msg_ptr) }
    #[no_mangle] extern "C" fn query(msg_ptr: u32) -> u32 {
        do_query(&query::<ExternalStorage, ExternalApi, ExternalQuerier>, msg_ptr) }
}

pub fn init <S: Storage, A: Api, Q: Querier> (deps: &mut Extern<S, A, Q>, env: Env, msg: Init)
    -> StdResult<InitResponse>   { Contract::init(deps, env, msg) }
pub fn handle <S: Storage, A: Api, Q: Querier> (deps: &mut Extern<S, A, Q>, env: Env, msg: Handle)
    -> StdResult<HandleResponse> { Contract::handle(deps, env, msg) }
pub fn query <S: Storage, A: Api, Q: Querier> (deps: &Extern<S, A, Q>, msg: Query)
    -> StdResult<Binary>         { to_binary(&Contract::query(deps, msg)?) }

pub trait Contract<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
    + Rewards<S, A, Q>
    + Immigration<S, A, Q>
    + Emigration<S, A, Q>
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
    Immigration(ImmigrationHandle),
    Emigration(EmigrationHandle),
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
            Handle::Immigration(msg) =>
                Immigration::handle(core, env, msg),
            Handle::Emigration(msg) =>
                Emigration::handle(core, env, msg),
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
    TokenInfo { name: String, symbol: String, decimals: u8, total_supply: Option<Amount> },
    /// Keplr integration
    Balance { amount: Amount }
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

        impl<S: Storage, A: Api, Q: Querier> crate::migration::Emigration<S, A, Q> for $Core {
            fn handle_export_state (&mut self, env: &Env, migrant: &HumanAddr)
                -> StdResult<HandleResponse>
            {
                let receiver    = self.can_export_state(&env, &migrant)?;
                let mut account = Account::from_addr(self, &migrant, env.block.time)?;
                let staked      = account.staked;
                let id          = self.canonize(migrant.clone())?;

                let snapshot = to_binary(&((
                    migrant.clone(),
                    Auth::load_vk(self, id.as_slice())?.map(|vk|vk.0),
                    staked
                ) as AccountSnapshot))?;

                let mut response = HandleResponse::default();

                if staked > Amount::zero() {
                    // Write off the user's LP tokens as withdrawn
                    account.commit_withdrawal(self, staked)?;
                    // Transfer LP tokens directly to the new version
                    response = response.msg(
                        RewardsConfig::lp_token(self)?.transfer(&env.message.sender, staked)?
                    )?;
                }

                response = response.msg(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr:      receiver.address,
                    callback_code_hash: receiver.code_hash,
                    send: vec![],
                    msg: self.wrap_receive_msg(ImmigrationHandle::ReceiveMigration(snapshot))?
                }))?;

                Ok(response)
            }

            fn wrap_receive_msg (&self, msg: ImmigrationHandle) -> StdResult<Binary> {
                to_binary(&Handle::Immigration(msg))
            }
        }

        impl<S: Storage, A: Api, Q: Querier> crate::migration::Immigration<S, A, Q> for $Core {
            fn handle_receive_migration (&mut self, env: Env, data: Binary) ->
                StdResult<HandleResponse>
            {
                let (migrant, vk, staked): AccountSnapshot = from_slice(&data.as_slice())?;
                let id = self.canonize(migrant.clone())?;
                // Set the migrant's viewing key
                if let Some(vk) = vk {
                    // for some reason it does not see Auth as implemented
                    //Auth::save_vk(&mut core, id.as_slice(), &vk)?;
                    self.set_ns(crate::auth::VIEWING_KEYS, id.as_slice(), &vk)?;
                }
                // Add the LP tokens transferred by the migration
                // to the migrant's new account
                Account::from_addr(self, &migrant, env.block.time)?
                    .commit_deposit(self, staked)?;
                HandleResponse::default()
                    .log("migrated", &staked.to_string())
            }

            fn wrap_export_msg (&self, msg: EmigrationHandle) -> StdResult<Binary> {
                to_binary(&Handle::Emigration(msg))
            }
        }

        impl<S: Storage, A: Api, Q: Querier> crate::drain::Drain<S, A, Q> for $Core {}

    };
}

compose!(Extern<S, A, Q>);
