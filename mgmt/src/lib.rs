mod types; use types::*;
mod schedule;
mod progress; use progress::FulfilledClaims;
mod configurable; use configurable::ConfiguredRecipients;

#[macro_use] extern crate fadroma;

/// Creator of contract.
/// TODO make configurable
type Admin = Address;

/// The token contract that will be controlled.
/// TODO see how this can be generated for testing
type Token = Option<Address>;

/// Whether the vesting process has begun and when.
type Launched = Option<Time>;

/// TODO: Public hit counter. ;)
type ErrorCount = u64;

macro_rules! SIENNA {
    ($x:tt) => {
        cosmwasm_std::coins($x, "SIENNA")
    }
}

macro_rules! canon {
    ($deps:ident, $($x:tt)*) => {
        $deps.api.canonical_address($($x)*).unwrap();
    }
}

//macro_rules! human {
    //($deps:ident, $($x:tt)*) => {
        //$deps.api.human_address($($x)*).unwrap();
    //}
//}

contract!(

    State {
        admin:      Admin,
        token:      Token,
        launched:   Launched,
        vested:     FulfilledClaims,
        recipients: ConfiguredRecipients,
        errors:     ErrorCount
    }

    // Initializing an instance of the contract:
    // * requires the address of a SNIP20-compatible token contract
    //   to be passed as an argument
    // * makes the initializer the admin
    InitMsg (deps, env, msg: {
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

    QueryMsg (deps, state, msg) {
        // Querying the status.
        // TODO how much info should be available here?
        StatusQuery () {
            msg::StatusResponse { launched: state.launched }
        }
    }

    HandleMsg (deps, env, sender, state, msg) {

        // Most schedules are static (imported from config at compile time).
        // However the config supports `release_mode: configurable` which
        // allows their streams to be redirected in runtime-configurable
        // proportions.
        SetRecipients (recipients: crate::ConfiguredRecipients) {
            if sender != state.admin {
                state.errors += 1;
                return err_auth(state)
            }

            state.recipients = recipients;
            ok(state)
        }

        // After configuring the instance, launch confirmation must be given.
        // An instance can be launched only once.
        // TODO emergency vote to stop everything and refund the initializer
        // TODO launch transaction should receive/mint its budget?
        Launch () {
            if sender != state.admin {
                state.errors += 1;
                return err_auth(state)
            }
            match state.launched {
                Some(_) => err_msg(state, "already underway"),
                None => {
                    state.launched = Some(env.block.time);
                    ok(state)
                }
            }
        }

        // Recipients can call the Claim method to receive
        // the gains that have accumulated so far.
        Claim () {
            match &state.launched {
                None => {
                    state.errors += 1;
                    err_auth(state)
                },
                Some(launch) => {
                    let now      = env.block.time;
                    let contract = env.contract.address;
                    let claimant = env.message.sender;

                    let claimant_canon =
                        canon!(deps, &claimant);
                    let slope =
                        schedule::at(&claimant_canon, *launch, now);
                    let progress =
                        progress::at(&claimant_canon, &state.vested, now);
                    let difference =
                        slope - progress;

                    if difference < 0 {
                        return err_msg(state, "broken")
                    }

                    if difference == 0 {
                        return err_msg(state, "nothing for you")
                    }

                    state.vested.push((claimant_canon, now, slope));
                    ok_send(
                        state,
                        contract,
                        claimant,
                        SIENNA!(difference)
                    )
                }
            }
        }
    }

    Response {
        StatusResponse { launched: Option<u64> }
    }

);
