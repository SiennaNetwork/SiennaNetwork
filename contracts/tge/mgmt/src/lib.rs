mod migration;
mod state;

pub use state::{Claim, Pagination};

use fadroma::{
    admin,
    admin::assert_admin,
    cosmwasm_std,
    cosmwasm_std::{
        log, CanonicalAddr, HandleResponse, HumanAddr, InitResponse, StdError, StdResult, Uint128,
    },
    derive_contract::*,
    killswitch, require_admin, schemars,
    secret_toolkit::snip20,
    Canonize, ContractLink, Humanize, BLOCK_SIZE,
};
use serde::{Deserialize, Serialize};

use migration::MgmtKillswitch;
use state::{Config, History, Participant};

use sienna_schedule::{validate::Validation, vesting::Vesting, Account, Pool, Schedule, Seconds};

/// This doesn't need to be private because the schedule and claim history are public by design.
const VIEWING_KEY: &str = "SiennaMGMT";

/// Error messages
#[macro_export]
macro_rules! MGMTError {
    (CORRUPTED) => {
        "Contract has entered a state that violates core assumptions".to_string()
    };
    (NOTHING) => {
        "Nothing to claim right now.".to_string()
    };
    (UNDERWAY) => {
        "The vesting has already begun.".to_string()
    };
    (PRELAUNCH) => {
        "The vesting has not yet begun.".to_string()
    };
    (NOT_FOUND) => {
        "Can't find account or pool by name".to_string()
    };
    (ADD_ACCOUNT) => {
        "Can't add account - pool full".to_string()
    };
    (PREFUND, $balance:expr, $required:expr) => {
        format!(
            "Required prefund balance: {}, actual balance: {}",
            $required, $balance
        )
    };
}

#[contract(
    entry,
    component(path = "admin"),
    component(path = "killswitch", custom_impl = "MgmtKillswitch")
)]
pub trait Mgmt {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        schedule: Schedule<HumanAddr>,
        token: ContractLink<HumanAddr>,
        // If false, the contract will be transfered ownership of the token and it will mint the required amount of tokens itself.
        // If true, it will expect and verify that it has the required balance upon launch.
        prefund: bool,
    ) -> StdResult<InitResponse> {
        schedule.validate()?;
        Config::save_schedule(deps, schedule)?;

        Config::save_token(deps, token.clone())?;
        Config::set_is_prefunded(&mut deps.storage, prefund)?;

        let mut response = admin::Admin::new(&admin::DefaultImpl, admin, deps, env)?;

        // We only need this key to check the balance during launch in the case
        // where the contract will be transfered the required amount before launch.
        if prefund {
            response.messages.push(snip20::set_viewing_key_msg(
                VIEWING_KEY.into(),
                None,
                BLOCK_SIZE,
                token.code_hash,
                token.address,
            )?);
        }

        Ok(response)
    }

    #[handle_guard]
    fn guard(msg: &HandleMsg) -> StdResult<()> {
        let operational = killswitch::is_operational(deps);

        if operational.is_err() && matches!(msg, HandleMsg::Killswitch(_) | HandleMsg::Admin(_)) {
            Ok(())
        } else {
            operational
        }
    }

    #[handle]
    #[require_admin]
    fn configure(schedule: Schedule<HumanAddr>) -> StdResult<HandleResponse> {
        Config::assert_not_launched(&deps.storage)?;

        schedule.validate()?;
        Config::save_schedule(deps, schedule)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "configure")],
            data: None,
        })
    }

    #[handle]
    #[require_admin]
    fn add_account(pool_name: String, account: Account<HumanAddr>) -> StdResult<HandleResponse> {
        let account = account.canonize(&deps.api)?;

        let mut schedule = Config::load_schedule(&deps.storage)?;
        schedule.add_account(&pool_name, account)?;

        Config::save_schedule(deps, schedule.humanize(&deps.api)?)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "add_account")],
            data: None,
        })
    }

    /// An instance can be launched only once.
    /// Launching:
    ///  - When not prefunding: the instance mints the total tokens as specified by
    /// the schedule, and prevents any more tokens from ever being minted
    /// by the underlying contract.
    ///
    ///  - When prefunding: the instance simply checks if its token balance is equal to the
    /// amount specified by the schedule.
    #[handle]
    #[require_admin]
    fn launch() -> StdResult<HandleResponse> {
        Config::assert_not_launched(&deps.storage)?;
        Config::set_launched(&mut deps.storage, env.block.time)?;

        let schedule = Config::load_schedule(&deps.storage)?;
        let token = Config::load_token(deps)?;

        let messages = if Config::is_prefunded(&deps.storage)? {
            let balance = snip20::balance_query(
                &deps.querier,
                env.contract.address,
                VIEWING_KEY.into(),
                BLOCK_SIZE,
                token.code_hash,
                token.address,
            )?
            .amount;

            if balance < schedule.total {
                return Err(StdError::generic_err(MGMTError!(
                    PREFUND,
                    balance,
                    schedule.total
                )));
            }

            vec![]
        } else {
            vec![
                snip20::mint_msg(
                    env.contract.address,
                    schedule.total,
                    None,
                    None,
                    BLOCK_SIZE,
                    token.code_hash.clone(),
                    token.address.clone(),
                )?,
                snip20::set_minters_msg(vec![], None, BLOCK_SIZE, token.code_hash, token.address)?,
            ]
        };

        Ok(HandleResponse {
            messages,
            log: vec![log("action", "launch"), log("launched", env.block.time)],
            data: None,
        })
    }

    /// After launch, recipients can call the Claim method to
    /// receive the gains that they have accumulated so far.
    #[handle]
    fn claim() -> StdResult<HandleResponse> {
        let launched = Config::assert_launched(&deps.storage)?;
        let elapsed = get_elapsed(env.block.time, launched);

        let mut claimant = Participant::new(deps, &env.message.sender)?;
        let schedule = Config::load_schedule(&deps.storage)?;

        let (unlocked, claimable) = portion(&schedule, &claimant, elapsed);

        if claimable.eq(&u128::MIN) {
            return Err(StdError::generic_err(MGMTError!(NOTHING)));
        }

        claimant.set_claimed(&mut deps.storage, unlocked.into())?;
        History::push(
            &mut deps.storage,
            Claim::new(claimant, &env.block, claimable.into()),
        )?;

        let token = Config::load_token(deps)?;

        Ok(HandleResponse {
            messages: vec![snip20::transfer_msg(
                env.message.sender,
                claimable.into(),
                None,
                None,
                BLOCK_SIZE,
                token.code_hash,
                token.address,
            )?],
            log: vec![log("action", "claim"), log("claimed", claimable)],
            data: None,
        })
    }
    #[handle]
    #[require_admin]
    fn increase_allocation(
        total_increment: Uint128,
        pool: Pool<HumanAddr>,
    ) -> StdResult<HandleResponse> {
        let mut schedule = Config::load_schedule(&deps.storage)?;
        let token = Config::load_token(&deps)?;
        let pool = pool.canonize(&deps.api)?;

        schedule.total += total_increment;
        schedule.pools.push(pool);

        schedule.validate()?;

        let total_claimed = History::total_claimed(&deps.storage)?;

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address,
            VIEWING_KEY.into(),
            BLOCK_SIZE,
            token.code_hash,
            token.address,
        )?
        .amount;

        if balance < (schedule.total - total_claimed).unwrap() {
            return Err(StdError::generic_err(MGMTError!(
                PREFUND,
                balance,
                schedule.total
            )));
        }
        
        Config::save_schedule(deps, schedule.humanize(&deps.api)?)?;

        Ok(HandleResponse::default())
    }

    #[query]
    fn progress(address: HumanAddr, time: Seconds) -> StdResult<ProgressResponse> {
        let launched = Config::assert_launched(&deps.storage)?;
        let elapsed = get_elapsed(time, launched);

        let participant = Participant::new(deps, &address)?;

        let schedule = Config::load_schedule(&deps.storage)?;
        let unlocked = schedule.unlocked(elapsed, &participant.address).into();

        Ok(ProgressResponse {
            launched,
            elapsed,
            unlocked,
            claimed: participant.claimed(),
        })
    }

    #[query]
    fn config() -> StdResult<ConfigResponse> {
        Ok(ConfigResponse {
            launched: Config::get_launched(&deps.storage)?,
            token: Config::load_token(deps)?,
        })
    }

    #[query]
    fn history(pagination: Pagination) -> StdResult<HistoryResponse> {
        History::list(deps, pagination)
    }

    #[query]
    fn schedule() -> StdResult<Schedule<HumanAddr>> {
        let schedule = Config::load_schedule(&deps.storage)?;

        schedule.humanize(&deps.api)
    }
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfigResponse {
    pub launched: Option<Seconds>,
    pub token: ContractLink<HumanAddr>,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoryResponse {
    pub entries: Vec<Claim<HumanAddr>>,
    pub total: u64,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgressResponse {
    pub launched: Seconds,
    pub elapsed: Seconds,
    pub unlocked: Uint128,
    pub claimed: Uint128,
}

fn get_elapsed(t1: Seconds, t2: Seconds) -> Seconds {
    if t1 > t2 {
        t1 - t2
    } else {
        0
    }
}

fn portion(
    schedule: &Schedule<CanonicalAddr>,
    participant: &Participant,
    elapsed: Seconds,
) -> (u128, u128) {
    let unlocked = schedule.unlocked(elapsed, &participant.address);

    if unlocked > 0 {
        let claimed = participant.claimed().u128();

        if unlocked > claimed {
            return (unlocked, unlocked - claimed);
        }
    }

    (unlocked, 0)
}
