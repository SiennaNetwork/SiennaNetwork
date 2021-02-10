#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;

use cosmwasm_std::HumanAddr;
use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg};
use sienna_schedule::{Seconds, Schedule, History, Account};

//macro_rules! debug { ($($tt:tt)*)=>{} }

/// Auth
macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if Some($env.message.sender) != $state.admin {
            err_auth($state)
        } else $body
    }
}

/// The managed SNIP20 contract's code hash.
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// Public counter of invalid operations.
pub type ErrorCount = u64;

/// Default value for Secret Network block size
/// (according to Reuven on Discord).
pub const BLOCK_SIZE: usize = 256;

lazy_static! {
    /// Error message: claimed more than claimable.
    pub static ref BROKEN:      &'static str = "broken";
    /// Error message: unauthorized or nothing to claim right now.
    pub static ref NOTHING:     &'static str = "nothing for you";
    /// Error message: can't launch more than once.
    pub static ref UNDERWAY:    &'static str = "already underway";
    /// Error message: can't do this before launching.
    pub static ref PRELAUNCH:   &'static str = "not launched yet";
    /// Error message: schedule hasn't been set yet.
    pub static ref NO_SCHEDULE: &'static str = "set configuration first";
}

pub fn err_allocation (total: u128, max: u128) -> String {
    format!("allocations added up to {} which is over the maximum of {}",
        total, max)
}

contract!(

    [State] {
        /// The instantiatior of the contract
        admin:          Option<HumanAddr>,

        /// The SNIP20 token contract that will be managed by this instance
        token_addr:     HumanAddr,

        /// The code hash of the managed contract
        /// (see `secretcli query compute contract-hash --help`)
        token_hash:     CodeHash,

        /// Timestamp of the moment this was launched
        launched:       Launched,

        /// History of fulfilled claims
        vested:         History,

        /// A dedicated portion of the funds can be redirected at runtime
        schedule:       Option<Schedule>,

        /// TODO: public counter of invalid requests
        errors:         ErrorCount
    }

    /* Initializing an instance of the contract:
     *   - requires the address and code hash of
     *     a contract that implements SNIP20
     *   - makes the initializer the admin */
    [Init] (deps, env, msg: {
        token_addr: cosmwasm_std::HumanAddr,
        token_hash: crate::CodeHash,
        schedule:   Option<crate::Schedule>
    }) {
        State {
            admin:      Some(env.message.sender),
            schedule:   msg.schedule,
            token_addr: msg.token_addr,
            token_hash: msg.token_hash,
            launched:   None,
            vested:     History { history: vec![] },
            errors:     0
        }
    }

    [Query] (deps, state, msg) {
        // Querying the status.
        // TODO how much info should be available here?
        Status () {
            msg::Response::Status {
                errors:   state.errors,
                launched: state.launched,
            }
        }
        Schedule () {
            msg::Response::Schedule {
                schedule: state.schedule,
            }
        }
    }

    [Response] {
        Status {
            errors:   crate::ErrorCount,
            launched: crate::Launched
        }
        Schedule {
            schedule: Option<crate::Schedule>
        }
    }

    [Handle] (deps, env, sender, state, msg) {

        // Before launching the contract, a schedule must be loaded
        Configure (schedule: crate::Schedule) {
            require_admin!(|env, state| {
                match schedule.validate() {
                    Ok(_) => {
                        state.schedule = Some(schedule);
                        ok(state)
                    },
                    Err(e) => (state, Err(e))
                }
            })
        }

        // The admin can make someone else the admin
        // but there can be only one admin at a given time,
        TransferOwnership (new_admin: cosmwasm_std::HumanAddr) {
            require_admin!(|env, state| {
                state.admin = Some(new_admin);
                ok(state)
            })
        }

        // or the admin can disown the contract
        // so that nobody can be admin anymore:
        Disown () {
            require_admin!(|env, state| {
                state.admin = None;
                ok(state)
            })
        }

        // After configuring the instance, launch confirmation must be given.
        // An instance can be launched only once.
        Launch () {
            require_admin!(|env, state| {
                use crate::UNDERWAY;
                use cosmwasm_std::Uint128;
                match state.schedule {
                    None => err_msg(state, &NO_SCHEDULE),
                    Some(ref schedule) => match state.launched {
                        Some(_) => err_msg(state, &UNDERWAY),
                        None => {
                            let actions = vec![
                                mint_msg(
                                    env.contract.address,
                                    Uint128::from(schedule.clone().total),
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
                            ok_msg(state, actions)
                        }
                    }
                }
            })
        }

        // Recipients can call the Claim method to receive
        // the gains that they have accumulated so far.
        Claim () {
            use crate::{PRELAUNCH, BROKEN, NOTHING};
            use cosmwasm_std::Uint128;
            match &state.launched {
                None => err_msg(state, &PRELAUNCH),
                Some(launch) => {
                    let now       = env.block.time;
                    let claimant  = env.message.sender;
                    let elapsed   = now - *launch;
                    let schedule  = state.schedule.clone().unwrap();
                    let claimable = schedule.claimable(&claimant, elapsed);
                    let claimed   = state.vested.claimed(&claimant, now);
                    if claimable < claimed {
                        err_msg(state, &BROKEN)
                    } else {
                        let difference = claimable - claimed;
                        if difference <= 0 {
                            err_msg(state, &NOTHING)
                        } else {
                            match transfer_msg(
                                claimant.clone(),
                                Uint128::from(difference),
                                None, BLOCK_SIZE,
                                state.token_hash.clone(),
                                state.token_addr.clone(),
                            ) {
                                Err(e) => (state, Err(e)),
                                Ok(msg) => {
                                    let difference = Uint128::from(difference);
                                    state.vested.history.push((claimant, now, difference));
                                    ok_msg(state, vec![msg])
                                },
                            }
                        }
                    }
                }
            }
        }

    }

);
