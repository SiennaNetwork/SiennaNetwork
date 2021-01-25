mod types; use types::*;
mod schedule; use schedule::slope_at;
mod progress; use progress::{FulfilledClaims, progress_at};
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

macro_rules! SIENNA { ($x:tt) => { cosmwasm_std::coins($x, "SIENNA") } }

contract!(

    State {
        admin:      Admin,
        token:      Token,
        launched:   Launched,
        vested:     FulfilledClaims,
        recipients: ConfiguredRecipients
    }

    // Initializing an instance of the contract:
    // * requires the address of a SNIP20-compatible token contract
    //   to be passed as an argument
    // * makes the initializer the admin
    InitMsg (deps, env, msg: {
        token: crate::Token
    }) {
        State {
            admin:      deps.api.canonical_address(&env.message.sender).unwrap(),
            token:      msg.token,
            launched:   None,
            recipients: vec![],
            vested:     vec![]
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
            if !is_admin(&state, sender) { return err_auth() }
            if has_launched(&state) { return err("already underway") }
            state.recipients = recipients;
            Ok((state, cosmwasm_std::HandleResponse::default()))
        }

        // After configuring the instance, launch confirmation must be given.
        // An instance can be launched only once.
        // TODO emergency vote to stop everything and refund the initializer
        // TODO launch transaction should receive/mint its budget?
        Launch () {
            if !is_admin(&state, sender) { return err_auth() }
            if has_launched(&state) { return err("already underway") }
            state.launched = Some(env.block.time);
            Ok((state, cosmwasm_std::HandleResponse::default()))
        }

        // Recipients can call the Claim method to receive
        // the gains that have accumulated so far.
        Claim () {
            match &state.launched {
                None => err_auth(),
                Some(launch) => {
                    let contract = env.contract.address;
                    let sender   = env.message.sender;
                    let now      = env.block.time;

                    let slope = slope_at(launch, now, sender);
                    let progress = progress_at(&state.vested, &sender, now);
                    let difference = slope - progress;
                    if difference > 0 {
                        state.vested.push((sender, now, slope));
                        Ok((state, cosmwasm_std::HandleResponse {
                            log: vec![],
                            data: None,
                            messages: vec![cosmwasm_std::BankMsg::Send {
                                from_address: contract,
                                to_address:   sender,
                                amount:       SIENNA!(difference)
                            }],
                        }))
                    } else {
                        return err("nothing for you")
                    }
                }
            }
        }
    }

    Response {
        StatusResponse { launched: Option<u64> }
    }

);

fn err (msg: &str) -> cosmwasm_std::StdResult<State> {
    Err(cosmwasm_std::StdError::GenericErr { msg: String::from(msg), backtrace: None })
}

fn err_auth () -> cosmwasm_std::StdResult<State> {
    Err(cosmwasm_std::StdError::Unauthorized { backtrace: None })
}

fn has_launched (state: &State) -> bool {
    match state.launched { None => false, Some(_) => true }
}

fn has_not_launched (state: &State) -> bool {
    !has_launched(state)
}

fn is_admin (state: &State, addr: cosmwasm_std::CanonicalAddr) -> bool {
    addr == state.admin
}
