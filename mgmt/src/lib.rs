#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;
pub mod types; use types::*;
pub mod strings;
pub mod vesting;
#[macro_use] mod helpers;

use vesting::{SCHEDULE, claimable, claimed};
use secret_toolkit::snip20::handle::{mint_msg, transfer_msg};

/// Default value (according to Reuven on Discord)
const BLOCK_SIZE: usize = 256;

contract!(

    [State] {
        /// The instantiatior of the contract
        admin:          Admin,

        /// The SNIP20 token contract that will be managed by this instance
        token_addr:     TokenAddress,

        /// The code hash of the managed contract
        /// (see `secretcli query compute contract-hash --help`)
        token_hash:     CodeHash,

        /// Timestamp of the moment this was launched
        launched:       Launched,

        /// History of fulfilled claims
        vested:         FulfilledClaims,

        /// A dedicated portion of the funds can be redirected at runtime
        recipients:     Allocation,

        /// TODO: public counter of invalid requests
        errors:         ErrorCount
    }

    // Initializing an instance of the contract:
    // * requires the address of a SNIP20-compatible token contract
    //   to be passed as an argument
    // * makes the initializer the admin
    [Init] (deps, env, msg: {
        token_addr: crate::TokenAddress,
        token_hash: crate::CodeHash
    }) {
        State {
            admin:      canon!(deps, &env.message.sender),
            token_addr: msg.token_addr,
            token_hash: msg.token_hash,
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
        Status     { launched:   crate::types::Launched }
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
                let total = recipients.iter().fold(
                    0u128,
                    |acc, x| acc + x.1.u128()
                );
                if total > SCHEDULE.configurable_daily.u128() {
                    err_msg(state, &crate::strings::err_allocation(
                        total,
                        SCHEDULE.configurable_daily.u128()
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
                        let token_hash = state.token_hash.clone();
                        let token_addr = state.token_addr.clone();
                        match mint_msg(
                            env.contract.address,
                            cosmwasm_std::Uint128::from(SCHEDULE.total),
                            None, // padding
                            BLOCK_SIZE,
                            token_hash,
                            token_addr
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
                            let token_hash = state.token_hash.clone();
                            let token_addr = state.token_addr.clone();
                            match transfer_msg(
                                claimant,
                                cosmwasm_std::Uint128::from(difference),
                                None,
                                BLOCK_SIZE,
                                token_hash,
                                token_addr,
                            ) {
                                Ok(msg) => {
                                    state.vested.push((
                                        claimant_canon,
                                        now,
                                        claimable
                                    ));
                                    ok_msg(state, vec![msg])
                                },
                                Err(e) => (state, Err(e))
                            }
                        } else {
                            err_msg(state, &crate::strings::NOTHING)
                        }
                    }
                }
            }
        }

    }

);
