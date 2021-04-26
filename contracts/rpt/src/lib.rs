#[macro_use] extern crate fadroma;

// TODO(fadroma): we don't really need these to be public (see note in `mgmt`)
pub use secret_toolkit::{snip20::handle::transfer_msg, utils::space_pad};
pub use sienna_mgmt::msg::{Query as MGMTQuery, Response as MGMTResponse, Handle as MGMTHandle};
pub use linear_map::LinearMap;
pub use cosmwasm_std::{QueryRequest, WasmQuery};

/// Default value for Secret Network block size (used for padding)
pub const BLOCK_SIZE: usize = 256;

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
    (CORRUPTED) => { "Contract has entered a state that violates core assumptions." };
    (TOTAL: $x:expr, $y:expr) => { format!("Allocations must add up to {}, not {}", &$x, &$y) };
    (MGMT) => { "Main vesting contract returned unexpected response." };
}

contract!(
    [State] {
        /// The admin of the contract. Immutable.
        admin:   CanonicalAddr,
        /// The portion size of the RPT pool. Immutable as per requirements.
        portion: Uint128,
        /// How the portion is split. Must add up to `portion`.
        config:  Config<CanonicalAddr>,
        /// A link to the token.
        token:   ContractLink<CanonicalAddr>,
        /// A ling to the management contract which gives tokens.
        mgmt:    ContractLink<CanonicalAddr>
    }

    /// Requires MGMT and SNIP20 to be deployed. Their addresses and hashes,
    /// as well as the pool and account names, can't be changed after init.
    [Init] (deps, env, msg: {
        portion: Uint128,
        config:  Config<HumanAddr>,
        token:   ContractLink<HumanAddr>,
        mgmt:    ContractLink<HumanAddr>
    }) {
        validate(portion, &config)?;
        let config = canonize(&deps.api, config)?;
        let admin  = deps.api.canonical_address(&env.message.sender)?;
        let token  = (deps.api.canonical_address(&token.0)?, token.1);
        let mgmt   = (deps.api.canonical_address(&mgmt.0)?,  mgmt.1);
        State { admin, portion, config, token, mgmt }
    }

    [Query] (deps, state, msg) -> Response {
        Status () {
            Ok(Response::Status {
                config: humanize(&deps.api, state.config)?,
                token:  (deps.api.human_address(&state.token.0)?, state.token.1.clone()),
                mgmt:   (deps.api.human_address(&state.mgmt.0)?,  state.mgmt.1.clone())
            })
        }
    }

    [Response] {
        Status {
            config: Config<HumanAddr>,
            token:  ContractLink<HumanAddr>,
            mgmt:   ContractLink<HumanAddr>
        }
    }

    [Handle] (deps, env, state, msg) -> Response {

        /// Set how funds will be split.
        Configure (config: Config<HumanAddr>) {
            is_admin(&deps, &env, &state)?;
            validate(state.portion, &config)?;
            state.config = canonize(&deps.api, config)?;
            ok!(state)
        }

        /// Receive and distribute funds.
        /// `WARNING` a cliff on the RPT account could confuse this?
        Vest () {
            let claimable = query_claimable(&deps, &env, &state.mgmt)?;
            let mut messages = vec![];
            let mut msg = to_binary(&MGMTHandle::Claim {})?;
            space_pad(&mut msg.0, BLOCK_SIZE);
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      deps.api.human_address(&state.mgmt.0)?,
                callback_code_hash: state.mgmt.1.clone(),
                send:               vec![],
                msg,
            }));
            let claimable = claimable.u128();
            let portion   = state.portion.u128();
            let portions  = claimable / portion;
            let remainder = claimable % portion;
            for (addr, amount) in state.config.0.iter() {
                let msg = transfer(&deps.api, &state, addr, Uint128::from(amount.u128()*portions));
                messages.push(msg?);
            }
            ok!(state, messages, if remainder > 0 {
                vec![LogAttribute { key: "remainder (locked forever)".to_string(), value: remainder.to_string() }]
            } else {
                vec![]
            })
        }
    }
);

fn query_claimable <S:Storage,A:Api,Q:Querier> (
    deps: &Extern<S,A,Q>, env: &Env, mgmt: &ContractLink<CanonicalAddr>
) -> StdResult<Uint128> {
    let msg = MGMTQuery::Progress { address: env.contract.address.clone(), time: env.block.time };
    let mut msg = to_binary(&msg)?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    let response = deps.querier.query::<MGMTResponse>(
        &QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr:      deps.api.human_address(&mgmt.0)?,
            callback_code_hash: mgmt.1.clone(),
            msg,
        })
    );
    if let MGMTResponse::Progress { unlocked, claimed, .. } = response? {
        Ok((unlocked - claimed)?)
    } else {
        Err(StdError::GenericErr { msg: RPTError!(MGMT).to_string(), backtrace: None })
    }
}

fn validate <T> (portion: Uint128, config: &Config<T>) -> StdResult<()> {
    let total = sum_config(&config);
    if portion == total {
        Ok(())
    } else {
        Err(StdError::GenericErr { msg: RPTError!(TOTAL: portion, total), backtrace: None })
    }
}

fn sum_config <T> (map: &LinearMap<T, Uint128>) -> Uint128 {
    let mut total = Uint128::zero();
    for (_, amount) in map.0.iter() { total += *amount; }
    total
}

fn is_admin <S:Storage,A:Api,Q:Querier> (
    deps: &Extern<S,A,Q>, env: &Env, state: &State
) -> StatefulResult<()> {
    if state.admin == deps.api.canonical_address(&env.message.sender)? {
        return Ok(((), None))
    } else {
        Err(StatefulError((StdError::Unauthorized { backtrace: None }, None)))
    }
}

fn transfer <A:Api> (
    api: &A, state: &State, recipient: &CanonicalAddr, amount: Uint128
) -> StdResult<CosmosMsg> {
    let (token_addr, token_hash) = &state.token;
    let token_addr = api.human_address(&token_addr)?;
    let recipient  = api.human_address(&recipient)?;
    transfer_msg(recipient, amount, None, BLOCK_SIZE, token_hash.clone(), token_addr)
}

