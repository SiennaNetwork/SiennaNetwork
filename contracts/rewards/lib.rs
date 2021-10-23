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

use crate::{core::*, auth::*, algo::*, math::*};

use fadroma::scrt::{
    cosmwasm_std::*,
    callback::ContractInstance as ContractLink,
    snip20_api::ISnip20
};

message!(Init {
    admin:        Option<HumanAddr>,
    lp_token:     Option<ContractLink<HumanAddr>>,
    reward_token: ContractLink<HumanAddr>,
    viewing_key:  ViewingKey,
    ratio:        Option<Ratio>,
    threshold:    Option<Time>,
    cooldown:     Option<Time>
});

pub fn init <S: Storage + AsRef<S> + AsMut<S>, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    Contract::init(deps, &env, &msg)
}

messages!(Handle {
    Auth(AuthHandle)
    Migration(MigrationHandle)
    Rewards(RewardsHandle)
    ReleaseSnip20 {
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    }
});

pub fn handle <S: Storage + AsRef<S> + AsMut<S>, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    Contract::handle(deps, &env, &msg)
}

messages!(Query {
    Auth(AuthQuery),
    Rewards(RewardsQuery),

    /// For Keplr integration
    TokenInfo {}
    /// For Keplr integration
    Balance { address: HumanAddr, key: String }
});

pub fn query <S: Storage + AsRef<S>, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>,
    msg:  Query
) -> StdResult<Binary> {
    to_binary(&Contract::query(deps, &msg)?)
}

messages!(Response {
    Auth(AuthResponse),
    Rewards(RewardsResponse),

    /// Keplr integration
    TokenInfo {
        name:         String,
        symbol:       String,
        decimals:     u8,
        total_supply: Option<Amount>
    }

    /// Keplr integration
    Balance {
        amount: Amount
    }
});

pub trait Contract<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
    + Rewards<S, A, Q>
{
    fn init (&mut self, env: &Env, msg: &Init) -> StdResult<InitResponse> {
        Auth::init(self, env, &msg.admin)?;
        Rewards::init(self, env, msg)?;
        Ok(InitResponse {
            log:      vec![],
            messages: vec![
                ISnip20::attach(&msg.reward_token).set_viewing_key(&msg.viewing_key.0)?
            ],
        })
    }

    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<HandleResponse> {
        match msg {
            Auth(msg) =>
                Auth::handle(self, env, msg),
            CreateViewingKey { entropy, padding } =>
                Auth::handle(self, env, AuthHandle::CreateViewingKey { entropy, padding }),
            SetViewingKey { key, padding } =>
                Auth::handle(self, env, AuthHandle::SetViewingKey { key, padding }),

        }
        if let Some(response) = Auth::handle(self, env, msg)? {
            return response
        } else if let Some(response) = Rewards::handle(self, env, msg)? {
            return response
        } else {
            match msg {
                Handle::ClosePool { .. } => {},
                Handle::ReleaseSnip20 { .. } => {},
                _  => Err(StdError::generic_err("not implemented"))
            }
        }
    }

    fn query  (&self, msg: &Query) -> StdResult<Binary> {
        if let Some(response) = Auth::query(self, msg) {
            response
        } else if let Some(response) = Rewards::query(self, msg) {
            response
        } else {
            Err(StdError::generic_err("not implemented"))
        }
    }
}

impl<S: Storage, A: Api, Q: Querier> Auth<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Rewards<S, A, Q> for Extern<S, A, Q> {}

impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q> for Extern<S, A, Q> {}
