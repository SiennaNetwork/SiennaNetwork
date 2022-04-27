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
pub enum ClaimRecipient {
    Contract {
        contract: ContractLink<HumanAddr>,
        msg: Option<Binary>
    },
    Human(HumanAddr)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RewardsHandle {
    // Public transactions
    Deposit { amount: Amount },
    DepositReceiver { from: HumanAddr, amount: Amount },
    Withdraw { amount: Amount },
    Claim { to: Option<ClaimRecipient> },
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
            RewardsHandle::DepositReceiver { from, amount } => {
                let lp_token = RewardsConfig::lp_token(core)?;

                if lp_token.link.address == env.message.sender {
                    Account::from_addr(core, &from, env.block.time)?
                        .deposit(core, amount, false)
                } else {
                    Err(StdError::unauthorized())                    
                }
            }
            RewardsHandle::Deposit { amount } => {
                Account::from_env(core, &env)?.deposit(core, amount, true)
            }
            RewardsHandle::Withdraw { amount } => {
                Account::from_env(core, &env)?.withdraw(core, amount)
            }
            RewardsHandle::Claim { to } => {
                Account::from_env(core, &env)?.claim(core, to)
            }
            // Authorized transactions
            RewardsHandle::BeginEpoch { next_epoch } => Clock::increment(core, &env, next_epoch),
            // Admin-only transactions
            _ => {
                Auth::assert_admin(core, &env)?;
                match self {
                    RewardsHandle::Configure(config) => Ok(HandleResponse {
                        messages: config.store(core, env)?,
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
