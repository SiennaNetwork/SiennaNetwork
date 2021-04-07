#[macro_use] extern crate fadroma;

// TODO(fadroma): remove need for these to be public
pub use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg};
pub use sienna_schedule::{Seconds, Schedule, Pool, Account, vesting::Vesting, validate::Validation};
pub use linear_map::LinearMap;

/// How much each recipient has claimed so far
pub type History = LinearMap<CanonicalAddr, Uint128>;

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
    (CORRUPTED)   => { "broken" };  // Contract has entered a state that violates core assumptions.
    (NOTHING)     => { "nothing" }; // Unknown claimant or nothing to claim right now.
    (UNDERWAY)    => { "already underway" }; // Already launched
    (PRELAUNCH)   => { "not launched yet" }; // Not launched yet
    (NOT_FOUND)   => { "not found" };        // Can't find account or pool by name
    (ADD_ACCOUNT) => { "can't add account" } // Pool full
}

contract!(

    [State] {
        /// Starts out as the instantiatior of the contract, can be changed
        /// via `SetOwner` or `Disown`.
        admin:    Option<CanonicalAddr>,
        /// The SNIP20 token contract that will be managed by this instance.
        /// This needs to be provided at init and can't be changed.
        /// (see `secretcli query compute contract-hash --help` to get the hash).
        token:    ContractLink<CanonicalAddr>,
        /// When this contract is launched, this is set to the block time.
        launched: Launched,
        /// How much each address has received from the contract.
        history:  History,
        /// Vesting configuration. Can be changed using `Configure`.
        schedule: Schedule<CanonicalAddr>
    }

    [Init] (deps, env, msg: {
        schedule: Schedule<HumanAddr>,
        token:    ContractLink<HumanAddr>
    }) {
        let admin    = Some(deps.api.canonical_address(&env.message.sender)?);
        let history  = History::new();
        let launched = None;
        let schedule = schedule.canonize(&deps.api)?;
        let token = (deps.api.canonical_address(&token.0)?, token.1);
        State { admin, history, launched, schedule, token }
    }

    [Query] (deps, state, msg) {
        /// Return error count and launch timestamp.
        Status () {
            Ok(Response::Status { launched: state.launched })
        }

        /// Return schedule
        Schedule () {
            Ok(Response::Schedule { schedule: state.schedule.humanize(&deps.api)? })
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

        /// Return the allocated portion size of an account
        /// (used by RPT to validate its configuration)
        Portion (pool_name: String, account_name: String) {
            for pool in state.schedule.pools.iter() {
                if pool.name == pool_name {
                    for account in pool.accounts.iter() {
                        if account.name == account_name {
                            let portion = Uint128::from(account.portion_size());
                            return Ok(Response::Portion { portion })
                        }
                    }
                    break
                }
            }
            Ok(Response::NotFound {})
        }
    }

    [Response] {
        Status   { launched: Launched }
        Schedule { schedule: Schedule<HumanAddr> }
        Portion  { portion: Uint128 }
        Progress { time: Seconds, launched: Seconds, elapsed: Seconds, unlocked: Uint128, claimed: Uint128 }
        Error    { msg: String }
        NotFound {}
    }

    [Handle] (deps, env, state, msg) {
        /// Load a new schedule (only before launching the contract)
        Configure (schedule: Schedule<HumanAddr>) {
            is_admin(&deps.api, &state, &env)?;
            is_not_launched(&state)?;
            schedule.validate()?;
            state.schedule = schedule.canonize(&deps.api)?;
            ok!(state)
        }

        /// Add a new account to a partially filled pool
        AddAccount (pool_name: String, account: Account<HumanAddr>) {
            is_admin(&deps.api, &state, &env)?;
            let account = account.canonize(&deps.api)?;
            match state.schedule.add_account(pool_name, account) {
                Ok(()) => ok!(state),
                Err(e) => match e {
                    StdError::GenericErr { msg, .. } => err_msg(state, &msg),
                    _ => err_msg(state, MGMTError!(ADD_ACCOUNT))
                }
            }
        }

        /// The admin can make someone else the admin,
        /// but there can be only one admin at a given time (or none)
        SetOwner (new_admin: HumanAddr) {
            is_admin(&deps.api, &state, &env)?;
            state.admin = Some(deps.api.canonical_address(&new_admin)?);
            ok!(state)
        }

        /// DANGER: Set admin to None, making further changes impossible.
        Disown () {
            is_admin(&deps.api, &state, &env)?;
            state.admin = None;
            ok!(state)
        }

        /// An instance can be launched only once.
        /// Launching the instance mints the total tokens as specified by
        /// the schedule, and prevents any more tokens from ever being minted
        /// by the underlying contract.
        Launch () {
            is_admin(&deps.api, &state, &env)?;
            is_not_launched(&state)?;
            state.launched = Some(env.block.time);
            ok!(state, acquire(&deps.api, &state, &env)?, vec![
                LogAttribute { key: "launched".to_string(), value: env.block.time.to_string() }
            ])
        }

        /// After launch, recipients can call the Claim method to
        /// receive the gains that they have accumulated so far.
        Claim () {
            let launched = is_launched(&state)?;
            let elapsed  = get_elapsed(env.block.time, launched);
            let claimant = deps.api.canonical_address(&env.message.sender)?;
            let (unlocked, claimable) = portion(&state, &claimant, elapsed);
            if claimable > 0 {
                state.history.insert(claimant.clone(), unlocked.into());
                ok!(state, vec![transfer(&deps.api, &state, &claimant, claimable.into())?])
            } else {
                err_msg(state, MGMTError!(NOTHING))
            }
        }
    }

);

fn is_admin <A:Api> (api: &A, state: &State, env: &Env) -> StdResult<()> {
    let sender = api.canonical_address(&env.message.sender)?;
    if state.admin == Some(sender) { return Ok(()) }
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

fn acquire <A:Api> (api: &A, state: &State, env: &Env) -> StdResult<Vec<CosmosMsg>> {
    let (addr, hash) = state.token.clone();
    let addr = api.human_address(&addr)?;
    Ok(vec![
        mint_msg(
            env.contract.address.clone(), state.schedule.total,
            None, BLOCK_SIZE, hash.clone(), addr.clone()
        )?,
        set_minters_msg(
            vec![],
            None, BLOCK_SIZE, hash.clone(), addr.clone()
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
