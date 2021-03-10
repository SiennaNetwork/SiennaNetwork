#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;

use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg};

//macro_rules! debug { ($($tt:tt)*)=>{} }
pub mod safety; pub use safety::*;

/// The managed SNIP20 contract's code hash.
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<sienna_schedule::Seconds>;

/// Public counter of invalid operations.
pub type ErrorCount = u64;

/// Default value for Secret Network block size
/// (according to Reuven on Discord).
pub const BLOCK_SIZE: usize = 256;

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
        schedule:       sienna_schedule::Portions,

        /// Total amount to mint
        total:          cosmwasm_std::Uint128,

        /// TODO: public counter of invalid requests
        errors:         ErrorCount
    }

    /// Initializing an instance of the contract:
    ///  - requires the address and code hash of
    ///    a contract that implements SNIP20
    ///  - makes the initializer the admin
    [Init] (deps, env, msg: {
        schedule:   Option<sienna_schedule::Portions>,
        token_addr: cosmwasm_std::HumanAddr,
        token_hash: crate::CodeHash
    }) {
        State {
            admin:      Some(env.message.sender),
            errors:     0,

            token_addr: msg.token_addr,
            token_hash: msg.token_hash,

            total:      cosmwasm_std::Uint128::zero(),
            schedule:   match (msg.schedule) {
                Some(portions) => portions,
                None => vec![]
            },
            history:    sienna_schedule::History::new(),
            launched:   None,
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
                total:    state.total
            }
        }
    }

    [Response] {
        Status {
            errors:   crate::ErrorCount,
            launched: crate::Launched
        }
        Schedule {
            schedule: sienna_schedule::Portions,
            total:    cosmwasm_std::Uint128
        }
    }

    [Handle] (deps, env, sender, state, msg) {

        /// Load a new schedule (only before launching the contract)
        Configure (
            portions: sienna_schedule::Portions
        ) {
            require_admin!(|env, state| {
                state.history.validate_schedule_update(
                    &state.schedule,
                    &portions
                )?;
                state.total = cosmwasm_std::Uint128::zero();
                for portion in portions.iter() {
                    state.total += portion.amount
                }
                state.schedule = portions;
                ok(state)
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
                match &state.launched {
                    Some(_) => err_msg(state, &crate::UNDERWAY),
                    None => {
                        if state.schedule.len() < 1
                        || state.total == cosmwasm_std::Uint128::zero() {
                            err_msg(state, &crate::NO_SCHEDULE)
                        } else  {
                            let actions = vec![
                                mint_msg(
                                    env.contract.address,
                                    state.total,
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
            match &state.launched {
                None => err_msg(state, &crate::PRELAUNCH),
                Some(launch) => {
                    let now      = env.block.time;
                    let elapsed  = now - *launch;
                    let claimant = env.message.sender;
                    let claimable: sienna_schedule::Portions = state.schedule.clone()
                        .into_iter().filter(|portion| {
                            portion.vested<=elapsed &&
                            portion.address==claimant
                        }).collect();
                    if claimable.len() > 0 {
                        let unclaimed = state.history.unclaimed(&claimable);
                        if unclaimed.is_empty() {
                            err_msg(state, &NOTHING)
                        } else {
                            use cosmwasm_std::Uint128;
                            let mut sum: Uint128 = Uint128::zero();
                            for portion in unclaimed.iter() {
                                if portion.address != claimant {
                                    panic!("portion for wrong address {} claimed by {}", &portion.address, &claimant);
                                }
                                sum += portion.amount
                            }
                            let msg = transfer_msg(
                                claimant, sum,
                                None, BLOCK_SIZE,
                                state.token_hash.clone(),
                                state.token_addr.clone()
                            )?;
                            state.history.claim(now, unclaimed);
                            ok_msg(state, vec![msg])
                        }
                    } else {
                        err_msg(state, &crate::NOTHING)
                    }
                }
            }
        }

    }

);
