#[macro_use] extern crate fadroma;

contract!(

    b"config"

    InitMsg (deps, env, msg: {
        token_contract: Option<cosmwasm_std::CanonicalAddr>
    }) -> State {
        // Send from this address to launch the vesting process
        // TODO make configurable
        admin:          cosmwasm_std::CanonicalAddr =
            deps.api.canonical_address(&env.message.sender)?,

        // The token contract that will be controlled
        // TODO see how this can be generated for testing
        token_contract: Option<cosmwasm_std::CanonicalAddr> =
            msg.token_contract,

        // Whether the vesting process has begun
        launched:       Option<u64> =
            None
    }

    QueryMsg (msg, state, deps) {
        StatusQuery () {
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
