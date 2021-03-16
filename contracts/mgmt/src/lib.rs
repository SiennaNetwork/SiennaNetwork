#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;

// these are public so that the submodules defined by the macro can see them
// by importing `super::*`; if they show up in the docs as reexports, all the better -
// a cursory look through the docs would provide a (not-necessarily-exhaustive)
// list of the SNIP20 interactions that this contract performs
pub use secret_toolkit::snip20::handle::{mint_msg, transfer_msg, set_minters_msg};
pub use sienna_schedule::{Seconds, Schedule, Pool, Account, Portions, History};

#[macro_use] pub mod safety; pub use safety::*;

/// The managed SNIP20 contract's code hash.
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// Public counter of invalid operations.
pub type ErrorCount = u64;

/// Default value for Secret Network block size
/// (according to Reuven on Discord).
pub const BLOCK_SIZE: usize = 256;

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
        /// Total amount to mint
        total:      Uint128,
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
        let total    = Uint128::zero();
        let history  = History::new();
        let launched = None;
        State { errors, admin, token_addr, token_hash, total, schedule, history, launched }
    }

    [Query] (deps, state, msg) {

        /// Return error count and launch timestamp.
        Status () {
            Response::Status {
                errors:   state.errors,
                launched: state.launched,
            }
        }

        /// Return schedule and sum of total minted tokens
        GetSchedule () {
            Response::Schedule {
                schedule: state.schedule,
                total:    state.total
            }
        }

        /// Return one account from the schedule
        GetAccount (pool_name: String, account_name: String) {
            for pool in state.schedule.pools.iter() {
                if pool.name == pool_name {
                    for account in pool.accounts.iter() {
                        if account.name == account_name {
                            return Response::Account { pool, account }
                        }
                    }
                    break
                }
            }
            Response::NotFound {}
        }

        /// Return amount that can be claimed by the specified address at the specified time
        Claimable (address: HumanAddr, time: Seconds) {
            let amount = Uint128::zero();
            if let Some(launch) = &state.launched {
                let elapsed = time - *launch;
                let vested  = state.schedule.vested(address, elapsed);
                let claimed = state.history.get(address);
                if claimed < vested {
                    amount = vested - claimed
                }
                Response::Claimable { address, amount }
            } else {
                StdError::GenericErr { msg: String::from(&PRELAUNCH), backtrace: None }
            }
        }
    }

    [Response] {
        Status {
            errors:   ErrorCount,
            launched: Launched
        }
        Schedule {
            schedule: Portions,
            total:    Uint128
        }
        Account {
            pool:    Pool,
            account: Account
        }
        Claimable {
            address:  HumanAddr,
            amount:   Uint128
        }
        NotFound {}
    }

    [Handle] (deps, env, state, msg) {

        /// Load a new schedule (only before launching the contract)
        Configure (portions: Portions) {
            require_admin!(|env, state| {
                state.history.validate_schedule_update(
                    &state.schedule,
                    &portions
                )?;
                state.total = Uint128::zero();
                for portion in portions.iter() {
                    state.total += portion.amount
                }
                state.schedule = portions;
                ok!(state)
            })
        }

        /// The admin can make someone else the admin,
        /// but there can be only one admin at a given time (or none)
        TransferOwnership (new_admin: HumanAddr) {
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
                if let Some(_) = &state.launched {
                    return err_msg(state, &UNDERWAY)
                }
                if state.schedule.len() < 1 || state.total == Uint128::zero() {
                    return err_msg(state, &NO_SCHEDULE)
                }
                let messages = vec![
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
                ok!(state, messages)
            })
        }

        /// After launch, recipients can call the Claim method to
        /// receive the gains that they have accumulated so far.
        Claim () {
            if let Some(launch) = &state.launched {
                let now      = env.block.time;
                let elapsed  = now - *launch;
                let claimant = env.message.sender;

                let claimable: Portions = state.schedule.clone()
                    .into_iter().filter(|p| {p.vested<=elapsed && p.address==claimant}).collect();
                if claimable.is_empty() {
                    return err_msg(state, &NOTHING)
                }

                let unclaimed = state.history.unclaimed(&claimable);
                if unclaimed.is_empty() {
                    return err_msg(state, &NOTHING)
                }

                let mut sum: Uint128 = Uint128::zero();
                for p in unclaimed.iter() {
                    if p.address != claimant {
                        panic!("p for wrong address {} was to be claimed by {}",
                            &p.address,
                            &claimant
                        );
                    }
                    sum += p.amount
                }

                state.history.claim(now, unclaimed);
                ok!(state, vec![transfer_msg(
                    claimant, sum,
                    None, BLOCK_SIZE,
                    (&state.token_hash).clone(),
                    (&state.token_addr).clone()
                )?])
            } else {
                err_msg(state, &PRELAUNCH)
            }
        }

    }

);
