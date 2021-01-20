#[macro_use] extern crate fadroma;

use cosmwasm_std::CanonicalAddr;

/// Creator of contract.
/// TODO make configurable
type Admin = cosmwasm_std::CanonicalAddr;

/// The token contract that will be controlled.
/// TODO see how this can be generated for testing
type Token = Option<cosmwasm_std::CanonicalAddr>;

/// Whether the vesting process has begun and when.
type Launched = Option<u64>;

/// List of recipient addresses with vesting configs.
type Recipients = Vec<Recipient>;

/// `message!` defines a struct and makes it CosmWasm-serializable
message!(Recipient {
    address:  CanonicalAddr,
    cliff:    u64,
    vestings: u64,
    interval: u64,
    claimed:  u64
});

contract!(

    State {
        admin:      Admin,
        token:      Token,
        launched:   Launched,
        recipients: Recipients
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
            recipients: vec![]
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

        // After initializing the contract,
        // recipients need to be configured by the admin.
        SetRecipients (recipients: crate::Recipients) {
            if !is_admin(&state, sender) { return err_auth() }
            if has_launched(&state) { return err("already underway") }
            state.recipients = recipients;
            Ok(state)
        }

        // After configuring the instance, launch confirmation must be given.
        // An instance can be launched only once.
        // TODO emergency vote to stop everything and refund the initializer
        // TODO launch transaction should receive/mint its budget?
        Launch () {
            if !is_admin(&state, sender) { return err_auth() }
            if has_launched(&state) { return err("already underway") }
            state.launched = Some(env.block.time);
            Ok(state)
        }

        // Recipients can call the Claim method to receive
        // the gains that have accumulated so far.
        Claim () {
            if !has_launched(&state) { return err_auth() }
            if !can_claim(&state, sender) { return err("nothing for you") }
            //return Err("not implemented");
            Ok(state)
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

fn is_admin (state: &State, addr: CanonicalAddr) -> bool {
    addr == state.admin
}

fn can_claim (state: &State, addr: CanonicalAddr) -> bool {
    false
}
