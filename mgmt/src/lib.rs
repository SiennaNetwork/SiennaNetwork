#[macro_use] extern crate fadroma;

contract!(

    b"config"

    InitMsg (deps, env, msg: {
        token_contract: Option<cosmwasm_std::CanonicalAddr>
    }) -> State {
        // The token contract that will be controlled
        token_contract: Option<cosmwasm_std::CanonicalAddr> =
            msg.token_contract,
        // The admin who can launch the vesting process
        admin: cosmwasm_std::CanonicalAddr =
            deps.api.canonical_address(&env.message.sender)?,
        // Whether the vesting process has been started yet
        launched: Option<u64> =
            None
    }

    QueryMsg (msg, state, deps) {
        StatusQuery (foo: bool) {
            to_binary(&crate::msg::StatusResponse { launched: state.launched })
        }
    }

    HandleMsg (msg, &mut state, env, deps) {
        Launch () {
            state.launched = Some(env.block.time);
            Ok(state)
        }
    }

    Response {
        StatusResponse { launched: Option<u64> }
    }

);
