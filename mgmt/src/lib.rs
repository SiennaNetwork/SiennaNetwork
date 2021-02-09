#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;

use cosmwasm_std::HumanAddr;
use secret_toolkit::snip20::handle::{mint_msg, transfer_msg};
pub use sienna_schedule::{
    DAY, MONTH, ONE_SIENNA,
    Seconds, Days, Months, Percentage, Amount,
    Schedule, Pool, Account, Allocation, Vesting, Interval,
    History,
};

//macro_rules! debug { ($($tt:tt)*)=>{} }

/// Auth
macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if $env.message.sender != $state.admin {
            err_auth($state)
        } else $body
    }
}

/// A contract's code hash
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// Public counter of invalid operations.
pub type ErrorCount = u64;

/// Default value for Secret Network block size
/// (according to Reuven on Discord)
pub const BLOCK_SIZE: usize = 256;

lazy_static! {
    pub static ref BROKEN:      &'static str = "broken";
    pub static ref NOTHING:     &'static str = "nothing for you";
    pub static ref UNDERWAY:    &'static str = "already underway";
    pub static ref PRELAUNCH:   &'static str = "not launched yet";
    pub static ref NO_SCHEDULE: &'static str = "set configuration first";
}

pub fn err_allocation (total: Amount, max: Amount) -> String {
    format!("allocations added up to {} which is over the maximum of {}",
        total, max)
}

contract!(

    [State] {
        /// The instantiatior of the contract
        admin:          HumanAddr,

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
        use cosmwasm_std::Uint128;
        State {
            admin:      env.message.sender,
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
                            let schedule = schedule.clone();
                            let token_hash = state.token_hash.clone();
                            let token_addr = state.token_addr.clone();
                            match mint_msg(
                                env.contract.address,
                                Uint128::from(schedule.total),
                                None, BLOCK_SIZE, token_hash, token_addr
                            ) {
                                Ok(msg) => {
                                    state.launched = Some(env.block.time);
                                    ok_msg(state, vec![msg])
                                },
                                Err(e) => (state, Err(e))
                            }
                        }
                    }
                }
            })
        }

        // The admin can make someone else the admin
        // but there can be only one admin at a given time
        TransferOwnership (new_admin: cosmwasm_std::HumanAddr) {
            require_admin!(|env, state| {
                state.admin = new_admin;
                ok(state)
            })
        }

        // Update vesting configuration
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

        // Recipients can call the Claim method to receive
        // the gains that have accumulated so far.
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
