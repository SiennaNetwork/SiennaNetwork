#[macro_use] extern crate fadroma;

// TODO(fadroma): we don't really need these to be public (see note in `mgmt`)
pub use secret_toolkit::{snip20::handle::transfer_msg, utils::space_pad};
pub use sienna_mgmt::msg::{Query as MGMTQuery, Response as MGMTResponse, Handle as MGMTHandle};
pub use linear_map::LinearMap;

/// Into what parts to split the received amount
pub type Config = LinearMap<HumanAddr, Uint128>;

/// Code hashes for MGMT and SNIP20
pub type CodeHash = String;

/// Pair of address and code hash
pub type ContractLink = (HumanAddr, CodeHash);

/// Error messages
#[macro_export] macro_rules! RPTError {
    (CORRUPTED) => { "broken" };  // Contract has entered a state that violates core assumptions.
    (TOTAL: $x:expr, $y:expr) => { format!("allocations must add up to {}, not {}", &$x, &$y) };
    (MGMT) => { "mgmt returned wrong response" };
}

contract!(
    [State] {
        admin:   HumanAddr,
        pool:    String,
        account: String,
        config:  Config,
        token:   ContractLink,
        mgmt:    ContractLink
    }

    /// Requires MGMT and SNIP20 to be deployed. Their addresses and hashes,
    /// as well as the pool and account names, can't be changed after init.
    [Init] (deps, env, msg: {
        pool:    String,
        account: String,
        config:  Config,
        token:   ContractLink,
        mgmt:    ContractLink
    }) {
        let admin = env.message.sender;
        State { admin, pool, account, config, token, mgmt }
    }

    [Query] (_deps, state, msg) {
        Status () {
            Response::Status { config: state.config }
        }
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
                Err(e) => err_msg(state, &e),
                Ok(_) => {
                    state.config = config;
                    ok!(state)
                },
            }
        }

        /// Receive and distribute funds.
        /// `WARNING` a cliff on the RPT account would totally confuse this
        Vest () {
            let mut messages = vec![];
            // 1. check if the amount that will be claimed
            //    corresponds to the sum of the splits
            let response = query_portion_size(&state, &deps.querier)?;
            if let Err(e) = validate_config(response, &state.config) {
                return err_msg(state, &e);
            }
            // 2. claim funds:
            let (contract_addr, callback_code_hash) = state.mgmt.clone();
            let mut msg = to_binary(&MGMTHandle::Claim {})?;
            space_pad(&mut msg.0, BLOCK_SIZE);
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                msg, send: vec![], contract_addr, callback_code_hash
            }));
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

/// Default value for Secret Network block size (used for padding)
pub const BLOCK_SIZE: usize = 256;

fn query_portion_size<Q: Querier> (state: &State, querier: &Q) -> StdResult<MGMTResponse> {
    use cosmwasm_std::{QueryRequest, WasmQuery};
    let mut msg = to_binary(&MGMTQuery::Portion {
        pool_name:    state.pool.clone(),
        account_name: state.account.clone()
    })?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    let (contract_addr, callback_code_hash) = state.mgmt.clone();
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

/// Sum of configured allocations (to check against actual portion size)
fn sum_config<T> (map: &LinearMap<T, Uint128>) -> Uint128 {
    let mut total = Uint128::zero();
    for (_, amount) in map.0.iter() { total += *amount; }
    total
}

fn transfer (state: &State, recipient: &HumanAddr, amount: Uint128) -> StdResult<CosmosMsg> {
    let (addr, hash) = state.token.clone();
    transfer_msg(recipient.clone(), amount, None, BLOCK_SIZE, hash, addr)
}
