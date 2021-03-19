#[macro_use] extern crate fadroma;

// these are public so that the submodules defined by the macro can see them
// by importing `super::*`; if they show up in the docs as reexports, all the better -
// a cursory look through the docs would provide a (not-necessarily-exhaustive)
// list of the SNIP20 interactions that this contract performs
pub use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg};
pub use sienna_schedule::{Seconds, Schedule, Pool, Account, Vesting, Validation, LinearMap};

/// How much each recipient has claimed so far
pub type History = LinearMap<HumanAddr, Uint128>;

/// The managed SNIP20 contract's code hash.
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// Public counter of invalid operations.
pub type ErrorCount = u64;

/// Default value for Secret Network block size
/// (according to Reuven on Discord).
pub const BLOCK_SIZE: usize = 256;

/// Authentication
macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if Some($env.message.sender) != $state.admin {
            err_auth($state)
        } else {
            $body
        }
    }
}

/// Error messages
#[macro_export] macro_rules! MGMTError {
    (CORRUPTED)   => { "broken" };  // Contract has entered a state that violates core assumptions.
    (NOTHING)     => { "nothing" }; // Unknown claimant or nothing to claim right now.
    (UNDERWAY)    => { "already underway" }; // Already launched
    (PRELAUNCH)   => { "not launched yet" }; // Not launched yet
    (NOT_FOUND)   => { "not found" };        // Can't find account or pool by name
    (ADD_ACCOUNT) => { "unexpected error when adding account" } // Shouldn't happen
}

contract!(

    [State] {
        /// The instantiatior of the contract.
        admin:      Option<HumanAddr>,
        /// The SNIP20 token contract that will be managed by this instance.
        token_addr: HumanAddr,
        /// The code hash of the managed contract
        /// (see `secretcli query compute contract-hash --help`).
        token_hash: CodeHash,
        /// When this contract is launched, this is set to the block time.
        launched:   Launched,
        /// History of fulfilled claims.
        history:    History,
        /// Vesting configuration.
        schedule:   Schedule,
        /// TODO: public counter of invalid requests
        errors:     ErrorCount
    }

    /// Initializing an instance of the contract:
    ///  - requires the address and code hash of
    ///    a contract that implements SNIP20
    ///  - makes the initializer the admin
    [Init] (deps, env, msg: {
        schedule:   Schedule,
        token_addr: HumanAddr,
        token_hash: CodeHash
    }) {
        let errors   = 0;
        let admin    = Some(env.message.sender);
        let history  = History::new();
        let launched = None;
        State {
            errors, admin, history, launched,
            schedule, token_addr, token_hash
        }
    }

    [Query] (_deps, state, msg) {

        /// Return error count and launch timestamp.
        Status () {
            let State { errors, launched, .. } = state;
            Response::Status { errors, launched }
        }

        /// Return schedule
        Schedule () {
            let State { schedule, .. } = state;
            Response::Schedule { schedule }
        }

        /// Return the allocated portion size of an account
        /// (used by RPT to validate its configuration)
        Portion (pool_name: String, account_name: String) {
            for pool in state.schedule.pools.iter() {
                if pool.name == pool_name {
                    for account in pool.accounts.iter() {
                        if account.name == account_name {
                            return Response::Portion {
                                address: account.address.clone(),
                                portion: Uint128::from(account.portion_size())
                            }
                        }
                    }
                    break
                }
            }
            Response::NotFound {}
        }

        /// Return amount that can be claimed by the specified address at the specified time
        Progress (address: HumanAddr, time: Seconds) {
            if let Some(_) = &state.launched {
                let unlocked = state.schedule.unlocked(&address, time).into();
                let claimed = match state.history.get(&address.clone()) {
                    Some(&claimed) => claimed,
                    None => Uint128::zero()
                };
                Response::Progress { address, unlocked, claimed }
            } else {
                Response::Error { msg: MGMTError!(PRELAUNCH).to_string() }
            }
        }
    }

    [Response] {
        Status   { errors: ErrorCount, launched: Launched }
        Schedule { schedule: Schedule }
        Portion  { address: HumanAddr, portion: Uint128 }
        Progress { address: HumanAddr, unlocked: Uint128, claimed: Uint128 }
        Error    { msg: String }
        NotFound {}
    }

    [Handle] (deps, env, state, msg) {

        /// Load a new schedule (only before launching the contract)
        Configure (schedule: Schedule) {
            require_admin!(|env, state| {
                if let Some(_) = &state.launched { return err_msg(state, MGMTError!(UNDERWAY)) }
                schedule.validate()?;
                state.schedule = schedule;
                ok!(state)
            })
        }

        /// Add a new account to a partially filled pool
        AddAccount (pool: String, account: Account) {
            require_admin!(|env, state| {
                match state.schedule.add_account(pool, account) {
                    Ok(()) => ok!(state),
                    Err(e) => match e {
                        StdError::GenericErr { msg, .. } => err_msg(state, &msg),
                        _ => err_msg(state, MGMTError!(ADD_ACCOUNT))
                    }
                }
            })
        }

        /// The admin can make someone else the admin,
        /// but there can be only one admin at a given time (or none)
        SetOwner (new_admin: HumanAddr) {
            require_admin!(|env, state| {
                state.admin = Some(new_admin);
                ok!(state)
            })
        }

        /// The admin can disown the contract
        /// so that nobody can be admin anymore:
        Disown () {
            require_admin!(|env, state| {
                state.admin = None;
                ok!(state)
            })
        }

        /// An instance can be launched only once.
        /// Launching the instance mints the total tokens as specified by
        /// the schedule, and prevents any more tokens from ever being minted
        /// by the underlying contract.
        Launch () {
            require_admin!(|env, state| {
                if let Some(_) = &state.launched { return err_msg(state, MGMTError!(UNDERWAY)) }
                let messages = vec![
                    mint_msg(
                        env.contract.address,
                        state.schedule.total,
                        None, BLOCK_SIZE,
                        state.token_hash.clone(),
                        state.token_addr.clone()
                    ).unwrap(),
                    set_minters_msg(
                        vec![],
                        None, BLOCK_SIZE,
                        state.token_hash.clone(),
                        state.token_addr.clone()
                    ).unwrap(),
                ];
                state.launched = Some(env.block.time);
                ok!(state, messages)
            })
        }

        /// After launch, recipients can call the Claim method to
        /// receive the gains that they have accumulated so far.
        Claim () {
            if let &Some(launch) = &state.launched {
                let address = env.message.sender;
                let elapsed = env.block.time - launch;
                let (unlocked, claimable) = portion(&state, &address, elapsed);
                if claimable > 0 {
                    state.history.insert(address.clone().into(), unlocked.into());
                    ok!(state, vec![transfer(&state, &address, claimable.into())?])
                } else {
                    err_msg(state, MGMTError!(NOTHING))
                }
            } else {
                err_msg(state, MGMTError!(PRELAUNCH))
            }
        }
    }

);

fn portion (state: &State, address: &HumanAddr, elapsed: Seconds) -> (u128, u128) {
    let unlocked = state.schedule.unlocked(&address, elapsed);
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

fn transfer (state: &State, addr: &HumanAddr, amount: Uint128) -> StdResult<CosmosMsg> {
    let token_hash = state.token_hash.clone();
    let token_addr = state.token_addr.clone();
    transfer_msg(addr.clone(), amount, None, BLOCK_SIZE, token_hash, token_addr)
}
