#[macro_use] extern crate fadroma;

// TODO(fadroma): we don't really need these to be public (see note in `mgmt`)
pub use secret_toolkit::{snip20::handle::transfer_msg, utils::space_pad};
pub use sienna_mgmt::msg::{Query as MGMTQuery, Response as MGMTResponse, Handle as MGMTHandle};
pub use linear_map::LinearMap;
pub use cosmwasm_std::{QueryRequest, WasmQuery};

/// Into what parts to split the received amount
pub type Config<T> = LinearMap<T, Uint128>;
fn canonize <A:Api> (api: &A, config: Config<HumanAddr>) -> StdResult<Config<CanonicalAddr>> {
    let config: Result<Vec<_>,_> = config.0.iter().map(
        |(human, amount)| match api.canonical_address(human) {
            Ok(canon) => Ok((canon, *amount)),
            Err(e)    => Err(e)
        }).collect();
    Ok(LinearMap(config?))
}
fn humanize <A:Api> (api: &A, config: Config<CanonicalAddr>) -> StdResult<Config<HumanAddr>> {
    let config: Result<Vec<_>,_> = config.0.iter().map(
        |(canon, amount)| match api.human_address(canon) {
            Ok(human) => Ok((human, *amount)),
            Err(e)    => Err(e)
        }).collect();
    Ok(LinearMap(config?))
}

/// Code hashes for MGMT and SNIP20
pub type CodeHash = String;

/// Pair of address and code hash
pub type ContractLink<T> = (T, CodeHash);

/// Error messages
#[macro_export] macro_rules! RPTError {
    (CORRUPTED) => { "broken" };  // Contract has entered a state that violates core assumptions.
    (TOTAL: $x:expr, $y:expr) => { format!("allocations must add up to {}, not {}", &$x, &$y) };
    (MGMT) => { "mgmt returned wrong response" };
}

contract!(
    [State] {
        admin:   CanonicalAddr,
        pool:    String,
        account: String,
        config:  Config<CanonicalAddr>,
        token:   ContractLink<CanonicalAddr>,
        mgmt:    ContractLink<CanonicalAddr>
    }

    /// Requires MGMT and SNIP20 to be deployed. Their addresses and hashes,
    /// as well as the pool and account names, can't be changed after init.
    [Init] (deps, env, msg: {
        pool:    String,
        account: String,
        config:  Config<HumanAddr>,
        token:   ContractLink<HumanAddr>,
        mgmt:    ContractLink<HumanAddr>
    }) {
        let admin  = deps.api.canonical_address(&env.message.sender)?;
        let config = canonize(&deps.api, config)?;
        let token  = (deps.api.canonical_address(&token.0)?, token.1);
        let mgmt   = (deps.api.canonical_address(&mgmt.0)?,  mgmt.1);
        State { admin, pool, account, config, token, mgmt }
    }

    [Query] (deps, state, msg) {
        Status () {
            Ok(Response::Status { config: humanize(&deps.api, state.config)? })
        }
    }

    [Response] {
        Status { config: Config<HumanAddr> }
    }

    [Handle] (deps, env, state, msg) {

        /// Set how funds will be split.
        Configure (config: Config<HumanAddr>) {
            is_admin(&deps, &env, &state)?;
            let response = query_portion_size(&deps, &state)?;
            match validate_config(response, &config) {
                Err(e) => err_msg(state, &e),
                Ok(_) => {
                    state.config = canonize(&deps.api, config)?;
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
            let response = query_portion_size(&deps, &state)?;
            if let Err(e) = validate_config(response, &state.config) {
                return err_msg(state, &e);
            }
            // 2. claim funds:
            let mut msg = to_binary(&MGMTHandle::Claim {})?;
            space_pad(&mut msg.0, BLOCK_SIZE);
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      deps.api.human_address(&state.mgmt.0)?,
                callback_code_hash: state.mgmt.1.clone(),
                send:               vec![],
                msg,
            }));
            // 3. then distribte them among each recipient:
            for (addr, amount) in state.config.0.iter() {
                messages.push(transfer(&deps.api, &state, &addr, *amount)?);
            }
            ok!(state, messages)
        }
    }
);

fn is_admin <S:Storage,A:Api,Q:Querier> (
    deps: &Extern<S,A,Q>, env: &Env, state: &State
) -> StatefulResult<()> {
    if state.admin == deps.api.canonical_address(&env.message.sender)? {
        return Ok(((), None))
    } else {
        Err(StatefulError((StdError::Unauthorized { backtrace: None }, None)))
    }
}

/// Default value for Secret Network block size (used for padding)
pub const BLOCK_SIZE: usize = 256;

fn query_portion_size <S:Storage,A:Api,Q:Querier> (
    deps: &Extern<S,A,Q>, state: &State
) -> StdResult<MGMTResponse> {
    let mut msg = to_binary(&MGMTQuery::Portion {
        pool_name:    state.pool.clone(),
        account_name: state.account.clone()
    })?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr:      deps.api.human_address(&state.mgmt.0)?,
        callback_code_hash: state.mgmt.1.clone(),
        msg,
    });
    deps.querier.query::<MGMTResponse>(&query)
}

fn validate_config <T> (response: MGMTResponse, config: &Config<T>) -> Result<(), String> {
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
fn sum_config <T> (map: &LinearMap<T, Uint128>) -> Uint128 {
    let mut total = Uint128::zero();
    for (_, amount) in map.0.iter() { total += *amount; }
    total
}

fn transfer <A:Api> (
    api: &A, state: &State, recipient: &CanonicalAddr, amount: Uint128
) -> StdResult<CosmosMsg> {
    let (token_addr, token_hash) = state.token.clone();
    let token_addr = api.human_address(&token_addr)?;
    let recipient  = api.human_address(&recipient)?;
    transfer_msg(recipient, amount, None, BLOCK_SIZE, token_hash, token_addr)
}
