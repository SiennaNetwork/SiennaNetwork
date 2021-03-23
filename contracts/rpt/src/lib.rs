#[macro_use] extern crate fadroma;

// TODO(fadroma): we don't really need these to be public (see note in `mgmt`)
pub use secret_toolkit::{snip20::handle::transfer_msg, utils::space_pad};
pub use sienna_mgmt::msg::{Query as MGMTQuery, Response as MGMTResponse, Handle as MGMTHandle};
pub use linear_map::LinearMap;

/// Into what parts to split the received amount
pub type Config = LinearMap<HumanAddr, Uint128>;

/// Sum of configured allocations (to check against actual portion size)
fn sum_config<T> (map: &LinearMap<T, Uint128>) -> Uint128 {
    let mut total = Uint128::zero();
    for (_, amount) in map.0.iter() { total += *amount; }
    total
}

/// Code hashes for MGMT and SNIP20
pub type CodeHash = String;

/// Default value for Secret Network block size (used for padding)
pub const BLOCK_SIZE: usize = 256;

/// Authentication
#[macro_export] macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if $env.message.sender != $state.admin {
            err_auth($state)
        } else {
            $body
        }
    }
}

/// Error messages
#[macro_export] macro_rules! RPTError {
    (CORRUPTED) => { "broken" };  // Contract has entered a state that violates core assumptions.
    (TOTAL: $x:expr, $y:expr) => { format!("allocations must add up to {}, not {}", &$x, &$y) };
    (MGMT) => { "mgmt returned wrong response" };
}

contract!(
    [State] {
        admin:      HumanAddr,

        pool:       String,
        account:    String,
        config:     Config,

        token_addr: HumanAddr,
        token_hash: CodeHash,

        mgmt_addr:  HumanAddr,
        mgmt_hash:  CodeHash
    }

    /// Requires MGMT and SNIP20 to be deployed. Their addresses and hashes,
    /// as well as the pool and account names, can't be changed after init.
    [Init] (deps, env, msg: {
        pool:       String,
        account:    String,
        config:     Config,
        token_addr: HumanAddr,
        token_hash: CodeHash,
        mgmt_addr:  HumanAddr,
        mgmt_hash:  CodeHash
    }) {
        let admin = env.message.sender;
        State { admin, pool, account, config, token_addr, token_hash, mgmt_addr, mgmt_hash }
    }

    [Query] (deps, state, msg) {
        Status () { Response::Status { config: state.config } }
    }

    [Response] {
        Status { config: Config }
    }

    [Handle] (deps, env, state, msg) {

        /// Set how funds will be split.
        Configure (config: Config) {
            is_admin(&state, &env)?;
            let response = query_portion_size(&state, &deps.querier)?;
            match validate_config(response, &config) {
                Ok(_) => {
                    state.config = config;
                    ok!(state)
                },
                Err(e) => err_msg(state, &e)
            }
        }

        /// Receive and distribute funds.
        /// `WARNING` a cliff on the RPT account would totally confuse this
        Vest () {
            // 1. check if the amount that will be claimed
            //    corresponds to the sum of the splits
            let response = query_portion_size(&state, &deps.querier)?;
            if let Err(e) = validate_config(response, &state.config) {
                return err_msg(state, &e);
            }
            // 2. claim funds:
            let mut messages = vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    msg:  to_binary(&MGMTHandle::Claim {})?, // TODO padding
                    send: vec![],
                    contract_addr:      state.mgmt_addr.clone(),
                    callback_code_hash: state.mgmt_hash.clone(),
                })
            ];
            // 3. then distribte them among each recipient:
            for (addr, amount) in state.config.0.iter() {
                messages.push(transfer(&state, &addr, *amount)?);
            }
            ok!(state, messages)
        }
    }
);

fn is_admin (state: &State, env: &Env) -> StatefulResult<()> {
    if state.admin.clone() == env.message.sender {
        return Ok(((), None))
    } else {
        Err(StatefulError((StdError::Unauthorized { backtrace: None }, None)))
    }
}

fn query_portion_size<Q: Querier> (state: &State, querier: &Q) -> StdResult<MGMTResponse> {
    use cosmwasm_std::{QueryRequest, WasmQuery};
    let mut msg = to_binary(&MGMTQuery::Portion {
        pool_name:    state.pool.clone(),
        account_name: state.account.clone()
    })?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    let contract_addr      = state.mgmt_addr.clone();
    let callback_code_hash = state.mgmt_hash.clone();
    let query = QueryRequest::Wasm(WasmQuery::Smart { contract_addr, callback_code_hash, msg });
    querier.query::<MGMTResponse>(&query)
}

fn validate_config (response: MGMTResponse, config: &Config) -> Result<(), String> {
    if let MGMTResponse::Portion { portion } = response {
        let total = sum_config(&config);
        if total == portion {
            Ok(())
        } else {
            Err(RPTError!(TOTAL: portion, total).to_string())
        }
    } else {
        Err(RPTError!(MGMT).to_string())
    }
}

fn transfer (state: &State, addr: &HumanAddr, amount: Uint128) -> StdResult<CosmosMsg> {
    let token_hash = state.token_hash.clone();
    let token_addr = state.token_addr.clone();
    transfer_msg(addr.clone(), amount, None, BLOCK_SIZE, token_hash, token_addr)
}
