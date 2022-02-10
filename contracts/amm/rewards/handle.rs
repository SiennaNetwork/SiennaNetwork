use crate::{
    account::{Account, Amount, IAccount},
    auth::Auth,
    config::{IRewardsConfig, RewardsConfig},
    time_utils::{Clock, IClock},
    Rewards,
};
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RewardsHandle {
    // Public transactions
    Deposit { amount: Amount },
    Withdraw { amount: Amount },
    Claim {},
    // Authorized transactions
    BeginEpoch { next_epoch: u64 },
    // Admin-only transactions
    Configure(RewardsConfig),
    Close { message: String },
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for RewardsHandle
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn dispatch_handle(self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        match self {
            // Public transactions
            RewardsHandle::Deposit { amount } => {
                Account::from_env(core, &env)?.deposit(core, amount)
            }
            RewardsHandle::Withdraw { amount } => {
                Account::from_env(core, &env)?.withdraw(core, amount)
            }
            RewardsHandle::Claim {} => Account::from_env(core, &env)?.claim(core),
            // Authorized transactions
            RewardsHandle::BeginEpoch { next_epoch } => Clock::increment(core, &env, next_epoch),
            // Admin-only transactions
            _ => {
                Auth::assert_admin(core, &env)?;
                match self {
                    RewardsHandle::Configure(config) => Ok(HandleResponse {
                        messages: config.store(core)?,
                        log: vec![],
                        data: None,
                    }),
                    RewardsHandle::Close { message } => {
                        core.set(RewardsConfig::CLOSED, Some((env.block.time, message)))?;
                        Ok(HandleResponse::default())
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
