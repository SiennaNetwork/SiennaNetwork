#[macro_use] extern crate fadroma;
#[macro_use] extern crate lazy_static;
pub use secret_toolkit::snip20::handle::transfer_msg;

pub type Config = Vec<(HumanAddr, Uint128)>;
pub type CodeHash = String;
pub type ErrorCount = u64;
pub const BLOCK_SIZE: usize = 256;

/// Auth
#[macro_export] macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if $env.message.sender != $state.admin {
            err_auth($state)
        } else {
            $body
        }
    }
}

contract!(
    [State] {
        errors:      ErrorCount,
        admin:       HumanAddr,
        config:      Config,
        token_addr:  HumanAddr,
        token_hash:  CodeHash,
        mgmt_addr:   HumanAddr,
        mgmt_hash:   CodeHash
    }

    [Init] (deps, env, msg: {
        config:      Config,
        token_addr:  HumanAddr,
        token_hash:  CodeHash,
        mgmt_addr:   HumanAddr,
        mgmt_hash:   CodeHash
    }) {
        State {
            errors: 0,
            admin:       env.message.sender,
            config:      msg.config,
            token_addr:  msg.token_addr,
            token_hash:  msg.token_hash,
            mgmt_addr:   msg.mgmt_addr,
            mgmt_hash:   msg.mgmt_hash,
        }
    }

    [Query] (deps, state, msg) {
        Status () {
            Response::Status {
                errors: state.errors,
                config: state.config
            }
        }
    }

    [Response] {
        Status {
            errors: ErrorCount,
            config: Config
        }
    }

    [Handle] (deps, env, state, msg) {
        Configure (config: Config) {
            require_admin!(|env, state| {
                state.config = config;
                return ok!(state)
            })
        }
        Vest () {
            // claim funds from mgmt:
            let mut messages = vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    msg:  to_binary(&sienna_mgmt::msg::Handle::Claim {})?,
                    send: vec![],
                    contract_addr:      state.mgmt_addr.clone(),
                    callback_code_hash: state.mgmt_hash.clone(),
                })
            ];
            // then distribte them among each recipient:
            for (addr, amount) in state.config.iter() {
                messages.push(transfer_msg(
                    addr.clone(), amount.clone(), None, BLOCK_SIZE,
                    state.token_hash.clone(),
                    state.token_addr.clone()
                ).unwrap());
            }
            ok!(state, messages)
        }
    }
);

