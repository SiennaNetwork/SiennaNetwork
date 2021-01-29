#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;
pub mod types; use types::*;
pub mod strings;
pub mod vesting;
#[macro_use] mod helpers;

use vesting::{SCHEDULE, claimable, claimed};

contract!(

    [State] {
        admin:      Admin,
        token:      Token,
        launched:   Launched,
        vested:     FulfilledClaims,
        recipients: Allocation,
        errors:     ErrorCount
    }

    // Initializing an instance of the contract:
    // * requires the address of a SNIP20-compatible token contract
    //   to be passed as an argument
    // * makes the initializer the admin
    [Init] (deps, env, msg: {
        token: crate::Token
    }) {
        State {
            admin:      canon!(deps, &env.message.sender),
            token:      msg.token,
            launched:   None,
            recipients: vec![],
            vested:     vec![],
            errors:     0
        }
    }

    [Query] (deps, state, msg) {
        // Querying the status.
        // TODO how much info should be available here?
        Status () {
            msg::Response::Status { launched: state.launched }
        }
        Recipients () {
            let response =  msg::Response::Recipients { recipients: state.recipients };
            response
        }
    }

    [Response] {
        Status     { launched:   Option<u64> }
        Recipients { recipients: crate::types::Allocation }
    }

    [Handle] (deps, env, sender, state, msg) {

        // Most schedules are static (imported from config at compile time).
        // However the config supports `release_mode: configurable` which
        // allows their streams to be redirected in runtime-configurable
        // proportions.
        SetRecipients (recipients: crate::types::Allocation) {
            if sender != state.admin {
                state.errors += 1;
                err_auth(state)
            } else {
                let total = recipients.iter().fold(0, |acc, x| acc + x.1);
                if total > SCHEDULE.configurable_daily {
                    err_msg(state, &crate::strings::err_allocation(
                        total,
                        SCHEDULE.configurable
                    ))
                } else {
                    state.recipients = recipients.clone();
                    ok(state)
                }
            }
        }

        // After configuring the instance, launch confirmation must be given.
        // An instance can be launched only once.
        // TODO emergency vote to stop everything and refund the initializer
        // TODO launch transaction should receive/mint its budget?
        Launch () {
            if sender != state.admin {
                state.errors += 1;
                err_auth(state)
            } else {
                match state.launched {
                    Some(_) => err_msg(state, &crate::strings::UNDERWAY),
                    None => {
                        state.launched = Some(env.block.time);
                        ok(state)
                    }
                }
            }
        }

        // Recipients can call the Claim method to receive
        // the gains that have accumulated so far.
        Claim () {
            match &state.launched {
                None => {
                    state.errors += 1;
                    err_msg(state, &crate::strings::PRELAUNCH)
                },
                Some(launch) => {
                    let now = env.block.time;
                    let contract = env.contract.address;
                    let claimant = env.message.sender;
                    let claimant_canon = canon!(deps, &claimant);
                    let claimable = claimable(
                        &claimant, &claimant_canon,
                        &state.recipients, *launch, now);
                    let claimed = claimed(
                        &claimant_canon,
                        &state.vested, now);
                    println!("claim, {}/{} @ {}", claimed, claimable, now);
                    if claimable < claimed {
                        err_msg(state, &crate::strings::BROKEN)
                    } else {
                        let difference = claimable - claimed;
                        if difference > 0 {
                            state.vested.push((claimant_canon, now, claimable));
                            ok_send(
                                state,
                                contract,
                                claimant,
                                SIENNA!(difference as u128)
                            )
                        } else {
                            err_msg(state, &crate::strings::NOTHING)
                        }
                    }
                }
            }
        }

    }

);
