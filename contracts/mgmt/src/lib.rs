#[macro_use] extern crate fadroma;

// TODO(fadroma): remove need for these to be public
pub use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg, change_admin_msg};
pub use sienna_migration::{ContractStatus, ContractStatusLevel, is_operational, can_set_status};
pub use sienna_schedule::{
    Seconds, Schedule, Pool, Account,
    vesting::Vesting, validate::Validation, canon::{Humanize, Canonize}
};
pub use linear_map::LinearMap;

/// How much each recipient has claimed so far
pub type History<T> = LinearMap<T, Uint128>;

/// The managed SNIP20 contract's code hash.
pub type CodeHash = String;

/// Pair of address and code hash
pub type ContractLink<T> = (T, CodeHash);

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// Default value for Secret Network block size
/// (according to Reuven on Discord; used for padding).
pub const BLOCK_SIZE: usize = 256;

/// Error messages
#[macro_export] macro_rules! MGMTError {
    (CORRUPTED)   => { "Contract has entered a state that violates core assumptions".to_string() };
    (NOTHING)     => { "Nothing to claim right now.".to_string() };
    (UNDERWAY)    => { "The vesting has already begun.".to_string() };
    (PRELAUNCH)   => { "The vesting has not yet begun.".to_string() };
    (NOT_FOUND)   => { "Can't find account or pool by name".to_string() };
    (ADD_ACCOUNT) => { "Can't add account - pool full".to_string() };
}

contract!(

    [State] {
        /// Starts out as the instantiatior of the contract, can be changed via `SetOwner`
        admin:    CanonicalAddr,
        /// The SNIP20 token contract that will be managed by this instance.
        /// This needs to be provided at init and can't be changed.
        /// (see `secretcli query compute contract-hash --help` to get the hash).
        token:    ContractLink<CanonicalAddr>,
        /// When this contract is launched, this is set to the block time.
        launched: Launched,
        /// How much each address has received from the contract.
        history:  History<CanonicalAddr>,
        /// Vesting configuration. Can be changed using `Configure`.
        schedule: Schedule<CanonicalAddr>,
        /// The paused/migration flag.
        status:   ContractStatus
    }

    [Init] (deps, env, msg: {
        schedule: Schedule<HumanAddr>,
        history:  Option<History<HumanAddr>>,
        token:    ContractLink<HumanAddr>
    }) {
        State {
            admin:    deps.api.canonical_address(&env.message.sender)?,
            history:  history.unwrap_or_default().canonize(&deps.api)?,
            launched: None,
            schedule: schedule.canonize(&deps.api)?,
            token:    (deps.api.canonical_address(&token.0)?, token.1),
            status:   ContractStatus::default()
        }
    }

    [Query] (deps, state, msg) -> Response {
        /// Return error count and launch timestamp.
        Status () {
            Ok(Response::Status {
                status:   state.status,
                launched: state.launched,
                token:    (deps.api.human_address(&state.token.0)?, state.token.1.clone()),
            })
        }

        /// Return schedule
        Schedule () {
            Ok(Response::Schedule { schedule: state.schedule.humanize(&deps.api)? })
        }

        /// Return claim history
        History () {
            Ok(Response::History { history: state.history.humanize(&deps.api)? })
        }

        /// Return amount that can be claimed by the specified address at the specified time
        Progress (address: HumanAddr, time: Seconds) {
            if let Some(launched) = &state.launched {
                let address_human = address;
                let address_canon = deps.api.canonical_address(&address_human)?;
                let elapsed  = get_elapsed(time, *launched);
                let unlocked = state.schedule.unlocked(elapsed, &address_canon).into();
                let claimed  = match state.history.get(&address_canon) {
                    Some(&claimed) => claimed,
                    None => Uint128::zero()
                };
                Ok(Response::Progress { time, launched: *launched, elapsed, unlocked, claimed })
            } else {
                Ok(Response::Error { msg: MGMTError!(PRELAUNCH).to_string() })
            }
        }
    }

    [Response] {
        Status   { launched: Launched, token: ContractLink<HumanAddr>, status: ContractStatus }
        Schedule { schedule: Schedule<HumanAddr> }
        History  { history: History<HumanAddr> }
        Progress { time: Seconds, launched: Seconds, elapsed: Seconds, unlocked: Uint128, claimed: Uint128 }
        Error    { msg: String }
        NotFound {}
    }

    [Handle] (deps, env, state, msg) -> Response {

        /// Set the contract status.
        /// Used to pause the contract operation in case of errors,
        /// and to initiate a migration to a fixed version of the contract.
        SetStatus (level: ContractStatusLevel, reason: String, new_address: Option<HumanAddr>) {
            is_admin(&deps.api, &state, &env)?;
            can_set_status(&state.status, &level)?; // can't go back from migration

            let messages = match level {
                // upon entering migration mode,
                // token admin is changed from "MGMT" to "MGMT's admin"
                // so that the token can be administrated manually
                ContractStatusLevel::Migrating => vec![{
                    change_admin_msg(
                        deps.api.human_address(&state.admin)?,
                        None, BLOCK_SIZE, state.token.1.clone(), deps.api.human_address(&state.token.0)?
                    )?
                }],
                _ => vec![]
            };
            state.status = ContractStatus { level, reason, new_address };

            save_state!();
            Ok(HandleResponse { messages, data: None, log: vec![] })
        }

        /// The current admin can make someone else the admin.
        SetOwner (new_admin: HumanAddr) {
            is_admin(&deps.api, &state, &env)?;
            is_operational(&state.status)?;

            state.admin = deps.api.canonical_address(&new_admin)?;

            save_state!();
            Ok(HandleResponse::default())
        }

        /// Load a new schedule (only before launching the contract)
        Configure (schedule: Schedule<HumanAddr>) {
            is_admin(&deps.api, &state, &env)?;
            is_operational(&state.status)?;
            is_not_launched(&state)?;

            schedule.validate()?;
            state.schedule = schedule.canonize(&deps.api)?;

            save_state!();
            Ok(HandleResponse::default())
        }

        /// Add a new account to a partially filled pool
        AddAccount (pool_name: String, account: Account<HumanAddr>) {
            is_admin(&deps.api, &state, &env)?;
            is_operational(&state.status)?;

            let account = account.canonize(&deps.api)?;
            state.schedule.add_account(&pool_name, account)?;

            save_state!();
            Ok(HandleResponse::default())
        }

        /// An instance can be launched only once.
        /// Launching the instance mints the total tokens as specified by
        /// the schedule, and prevents any more tokens from ever being minted
        /// by the underlying contract.
        Launch () {
            is_admin(&deps.api, &state, &env)?;
            is_not_launched(&state)?;
            is_operational(&state.status)?;

            state.launched = Some(env.block.time);
            let messages = mint_and_clear_minters(&deps.api, &state, &env)?;

            save_state!();
            Ok(HandleResponse { messages, data: None, log: vec![
                LogAttribute { key: "launched".to_string(), value: env.block.time.to_string() }
            ] })
        }

        /// After launch, recipients can call the Claim method to
        /// receive the gains that they have accumulated so far.
        Claim () {
            is_operational(&state.status)?;

            let launched = is_launched(&state)?;
            let elapsed  = get_elapsed(env.block.time, launched);
            let claimant = deps.api.canonical_address(&env.message.sender)?;
            let (unlocked, claimable) = portion(&state, &claimant, elapsed);
            if claimable > 0 {
                state.history.insert(claimant.clone(), unlocked.into());
                let messages = vec![transfer(&deps.api, &state, &claimant, claimable.into())?];

                save_state!();
                Ok(HandleResponse { messages, data: None, log: vec![] })
            } else {
                Err(StdError::GenericErr { msg: MGMTError!(NOTHING), backtrace: None })
            }
        }
    }

);

fn is_admin <A:Api> (api: &A, state: &State, env: &Env) -> StdResult<()> {
    let sender = api.canonical_address(&env.message.sender)?;
    if state.admin == sender { return Ok(()) }
    Err(StdError::Unauthorized { backtrace: None })
}

fn is_not_launched (state: &State) -> StdResult<()> {
    match state.launched {
        None => Ok(()),
        Some(_) => Err(StdError::GenericErr {
            msg: MGMTError!(UNDERWAY).to_string(),
            backtrace: None
        })
    }
}

fn is_launched (state: &State) -> StdResult<Seconds> {
    match state.launched {
        Some(launched) => Ok(launched),
        None => Err(StdError::GenericErr {
            msg: MGMTError!(PRELAUNCH).to_string(),
            backtrace: None
        })
    }
}

fn get_elapsed (t1: Seconds, t2: Seconds) -> Seconds {
    if t1 > t2 {
        t1 - t2
    } else {
        0
    }
}

fn portion (state: &State, address: &CanonicalAddr, elapsed: Seconds) -> (u128, u128) {
    let unlocked = state.schedule.unlocked(elapsed, &address);
    if unlocked > 0 {
        let claimed = match state.history.get(&address.clone().into()) {
            Some(claimed) => claimed.u128(),
            None => 0
        };
        if unlocked > claimed {
            return (unlocked, unlocked - claimed);
        }
    }
    return (unlocked, 0)
}

fn mint_and_clear_minters <A:Api> (api: &A, state: &State, env: &Env) -> StdResult<Vec<CosmosMsg>> {
    let (addr_canon, hash) = state.token.clone();
    let addr_human = api.human_address(&addr_canon)?;
    Ok(vec![
        mint_msg(
            env.contract.address.clone(), state.schedule.total,
            None, BLOCK_SIZE, hash.clone(), addr_human.clone()
        )?,
        set_minters_msg(
            vec![],
            None, BLOCK_SIZE, hash.clone(), addr_human.clone()
        )?,
    ])
}

fn transfer <A:Api> (
    api: &A, state: &State, recipient: &CanonicalAddr, amount: Uint128
) -> StdResult<CosmosMsg> {
    let (token_addr, token_hash) = state.token.clone();
    let token_addr = api.human_address(&token_addr)?;
    let recipient  = api.human_address(&recipient)?;
    transfer_msg(recipient, amount, None, BLOCK_SIZE, token_hash, token_addr)
}

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
#[cfg(all(feature="browser",target_arch="wasm32"))]
mod wasm_js { fadroma_bind_js::bind_js!(cosmwasm_std, crate); }
