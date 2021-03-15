#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;

use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg};
use sienna_schedule::{Seconds, History};

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
    /// Error message: assumptions have been violated.
    pub static ref CORRUPTED:   &'static str = "broken";
    /// Error message: unauthorized or nothing to claim right now.
    pub static ref NOTHING:     &'static str = "nothing for you";
    /// Error message: can't launch more than once.
    pub static ref UNDERWAY:    &'static str = "already underway";
    /// Error message: can't do this before launching.
    pub static ref PRELAUNCH:   &'static str = "not launched yet";
    /// Error message: schedule hasn't been set yet.
    pub static ref NO_SCHEDULE: &'static str = "set configuration first";
    /// Error message: can't find channel/pool by name.
    pub static ref NOT_FOUND:   &'static str = "target not found";
}

pub fn err_allocation (total: u128, max: u128) -> String {
    format!("allocations added up to {} which is over the maximum of {}",
        total, max)
}

contract!(

    [State] {
        /// The instantiatior of the contract.
        admin:          Option<cosmwasm_std::HumanAddr>,

        /// The SNIP20 token contract that will be managed by this instance.
        token_addr:     cosmwasm_std::HumanAddr,

        /// The code hash of the managed contract
        /// (see `secretcli query compute contract-hash --help`).
        token_hash:     CodeHash,

        /// When this contract is launched, this is set to the block time.
        launched:       Launched,

        /// History of fulfilled claims.
        history:        sienna_schedule::History,

        /// Vesting configuration.
        schedule:       Option<sienna_schedule::Schedule>,

        /// TODO: public counter of invalid requests
        errors:         ErrorCount
    }

    /// Initializing an instance of the contract:
    ///  - requires the address and code hash of
    ///    a contract that implements SNIP20
    ///  - makes the initializer the admin
    [Init] (deps, env, msg: {
        schedule:   Option<sienna_schedule::Schedule>,
        token_addr: cosmwasm_std::HumanAddr,
        token_hash: crate::CodeHash
    }) {
        State {
            admin:      Some(env.message.sender),
            errors:     0,

            token_addr: msg.token_addr,
            token_hash: msg.token_hash,

            schedule:   msg.schedule,
            launched:   None,
            history:    History::new(),
        }
    }

    [Query] (deps, state, msg) {
        Status () {
            msg::Response::Status {
                errors:   state.errors,
                launched: state.launched,
            }
        }
        GetSchedule () {
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
            schedule: Option<sienna_schedule::Schedule>
        }
    }

    [Handle] (deps, env, sender, state, msg) {

        /// Load a new schedule (only before launching the contract)
        Configure (
            schedule: sienna_schedule::Schedule
        ) {
            require_admin!(|env, state| {
                match state.launched {
                    Some(_) => err_msg(state, &UNDERWAY),
                    None => match schedule.validate() {
                        Err(e) => (state, Err(e)),
                        Ok(_) => {
                            state.schedule = Some(schedule);
                            ok(state)
                        },
                    }
                }
            })
        }

        /// Add a new channel to a pool
        AddChannel (
            pool_name:   String,
            name:        String,
            amount:      cosmwasm_std::Uint128,
            allocations: Vec<sienna_schedule::Allocation>,
            periodic:    Option<sienna_schedule::Periodic>
        ) {
            require_admin!(|env, state| {
                match &state.schedule {
                    None => err_msg(state, &NO_SCHEDULE),
                    Some(schedule) => {
                        let mut changed = false;
                        let mut schedule = schedule.clone();
                        for pool in schedule.pools.iter_mut() {
                            if pool.name == pool_name {
                                let channel = sienna_schedule::Channel {
                                    name, amount, periodic,
                                    allocations: vec![(0, allocations)]
                                };
                                channel.validate()?;
                                pool.add_channel(channel)?;
                                changed = true;
                                break
                            }
                        }
                        if changed {
                            state.schedule = Some(schedule);
                            ok(state)
                        } else {
                            err_msg(state, &NOT_FOUND)
                        }
                    }
                }
            })
        }

        /// Update the allocations of a channel
        Reallocate (
            pool_name:    String,
            channel_name: String,
            allocations:  Vec<sienna_schedule::Allocation>
        ) {
            require_admin!(|env, state| {
                match &state.schedule {
                    None => err_msg(state, &NO_SCHEDULE),
                    Some(schedule) => {
                        let mut schedule = schedule.clone();
                        let mut changed = false;
                        for pool in schedule.pools.iter_mut() {
                            if pool.name == pool_name {
                                for channel in pool.channels.iter_mut() {
                                    if channel.name == channel_name {
                                        channel.allocations.push((
                                            env.block.time,
                                            allocations.clone()
                                        ));
                                        changed = true;
                                        break
                                    }
                                }
                                if changed { break; }
                            }
                        }
                        if changed {
                            state.schedule = Some(schedule);
                            ok(state)
                        } else {
                            err_msg(state, &NOT_FOUND)
                        }
                    }
                }
            })
        }

        /// The admin can make someone else the admin,
        /// but there can be only one admin at a given time (or none)
        TransferOwnership (
            new_admin: cosmwasm_std::HumanAddr
        ) {
            require_admin!(|env, state| {
                state.admin = Some(new_admin);
                ok(state)
            })
        }

        /// The admin can disown the contract
        /// so that nobody can be admin anymore:
        Disown () {
            require_admin!(|env, state| {
                state.admin = None;
                ok(state)
            })
        }

        /// An instance can be launched only once.
        /// Launching the instance mints the total tokens as specified by
        /// the schedule, and prevents any more tokens from ever being minted
        /// by the underlying contract.
        Launch () {
            require_admin!(|env, state| {
                use crate::UNDERWAY;
                match &state.schedule {
                    None => err_msg(state, &NO_SCHEDULE),
                    Some(schedule) => match &state.launched {
                        Some(_) => err_msg(state, &UNDERWAY),
                        None => {
                            schedule.validate()?;
                            let actions = vec![
                                mint_msg(
                                    env.contract.address,
                                    schedule.total,
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

        /// After launch, recipients can call the Claim method to
        /// receive the gains that they have accumulated so far.
        Claim () {
            use crate::{PRELAUNCH, NOTHING};
            use cosmwasm_std::{Uint128};
            match &state.launched {
                None => err_msg(state, &PRELAUNCH),
                Some(launch) => {
                    let now       = env.block.time;
                    let claimant  = env.message.sender;
                    let elapsed   = now - *launch;
                    let schedule  = state.schedule.clone().unwrap();
                    let claimable = schedule.claimable(&claimant, elapsed)?;
                    if claimable.is_empty() {
                        err_msg(state, &NOTHING)
                    } else {
                        let unclaimed = state.history.unclaimed(claimable.clone());
                        if unclaimed.is_empty() {
                            err_msg(state, &NOTHING)
                        } else {
                            let mut sum: Uint128 = Uint128::zero();
                            let mut broken = false;
                            for portion in unclaimed.iter() {
                                if portion.address != claimant {
                                    broken = true;
                                    break
                                }
                                sum += portion.amount
                            }
                            if broken {
                                err_msg(state, &format!("portion for wrong address claimed by {}", &claimant))
                            } else {
                                let msg = transfer_msg(
                                    claimant, sum,
                                    None, BLOCK_SIZE,
                                    state.token_hash.clone(),
                                    state.token_addr.clone()
                                )?;
                                state.history.claim(now, unclaimed);
                                ok_msg(state, vec![msg])
                            }
                        }
                    }
                }
            }
        }

    }

);
