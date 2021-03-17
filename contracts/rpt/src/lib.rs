#[macro_use] extern crate fadroma;
pub use secret_toolkit::snip20::handle::transfer_msg;
pub use sienna_mgmt::msg::{Query as MGMTQuery, Response as MGMTResponse, Handle as MGMTHandle};

pub type Config = Vec<(HumanAddr, Uint128)>;
fn sum_config (config: &Config) -> Uint128 {
    let mut total = Uint128::zero();
    for (_, amount) in config.iter() { total += *amount; }
    total
}

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
        total:       Uint128,
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
        let errors = 0;
        let admin = env.message.sender;
        let total = sum_config(&config);
        State { errors, admin, total, config, token_addr, token_hash, mgmt_addr, mgmt_hash }
    }

    [Query] (deps, state, msg) {
        Status () { Response::Status { errors: state.errors, config: state.config } }
    }

    [Response] {
        Status { errors: ErrorCount, config: Config }
    }

    [Handle] (deps, env, state, msg) {
        Configure (config: Config) {
            require_admin!(|env, state| {
                let address = env.contract.address;
                let time    = env.block.time;
                let query   = MGMTQuery::Claimable { address, time };
                let response = deps.querier.query::<MGMTResponse>(
                    &cosmwasm_std::QueryRequest::Wasm(
                        cosmwasm_std::WasmQuery::Smart {
                            contract_addr:      state.mgmt_addr.clone(),
                            callback_code_hash: state.mgmt_hash.clone(),
                            msg: to_binary(&query)? // TODO pad to BLOCK_SIZE
                        }
                    )
                )?;
                if let MGMTResponse::Claimable { claimable, .. } = response {
                    let total = sum_config(&config);
                    if claimable == total {
                        state.config = config;
                        state.total = total;
                        ok!(state)
                    } else {
                        err_msg(state, &format!("allocations must add up to {}, not {}",
                            &claimable,
                            &total
                        ))
                    }
                } else {
                    err_msg(state, &format!("mgmt returned wrong response"))
                }
            })
        }
        Vest () {
            // check how much can be claimed
            // claim funds from mgmt:
            let mut messages = vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    msg:  to_binary(&MGMTHandle::Claim {})?, // TODO padding
                    send: vec![],
                    contract_addr:      state.mgmt_addr.clone(),
                    callback_code_hash: state.mgmt_hash.clone(),
                })
            ];
            // then distribte them among each recipient:
            for (addr, amount) in state.config.iter() {
                messages.push(transfer(&state, &addr, *amount)?);
            }
            ok!(state, messages)
        }
    }
);

fn transfer (state: &State, addr: &HumanAddr, amount: Uint128) -> StdResult<CosmosMsg> {
    let token_hash = state.token_hash.clone();
    let token_addr = state.token_addr.clone();
    transfer_msg(addr.clone(), amount, None, BLOCK_SIZE, token_hash, token_addr)
}
