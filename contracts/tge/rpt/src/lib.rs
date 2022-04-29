use std::ops::Sub;

use fadroma::{
    admin,
    admin::assert_admin,
    cosmwasm_std::{self, HandleResponse, InitResponse, StdResult, Uint128},
    derive_contract::{contract, init},
    require_admin,
    schemars::{self, JsonSchema},
    secret_toolkit::snip20,
    space_pad, to_binary, to_cosmos_msg, Api, ContractLink, CosmosMsg, Env, Extern, HumanAddr,
    LogAttribute, Querier, StdError, Storage, WasmQuery, BLOCK_SIZE,
};
pub mod state;
use fadroma::{derive_contract::*, killswitch};
pub use linear_map::LinearMap;
use serde::{Deserialize, Serialize};
pub use sienna_mgmt::{HandleMsg as MGMTHandle, ProgressResponse, QueryMsg as MGMTQuery};
use state::{Portion, State};

/// Default value for Secret Network block size (used for padding)

/// Into what parts to split the received amount
pub type Distribution<T> = LinearMap<T, Uint128>;

/// Code hashes for MGMT and SNIP20
pub type CodeHash = String;

/// Error messages
#[macro_export]
macro_rules! RPTError {
    (CORRUPTED) => {
        "Contract has entered a state that violates core assumptions."
    };
    (TOTAL: $x:expr, $y:expr) => {
        format!("Allocations must add up to {}, not {}", &$x, &$y)
    };
    (MGMT) => {
        "Main vesting contract returned unexpected response."
    };
}

#[contract(entry, component(path = "admin"), component(path = "killswitch"))]
pub trait RPT {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        distribution: Distribution<HumanAddr>,
        portion: Portion,
        token: ContractLink<HumanAddr>,
        mgmt: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse> {
        validate(portion, &distribution)?;
        State::save_distribution(deps, distribution)?;
        State::save_portion(deps, portion)?;
        State::save_token(deps, token)?;
        State::save_mgmt(deps, mgmt)?;

        let response = admin::Admin::new(&admin::DefaultImpl, admin, deps, env)?;

        Ok(response)
    }
    #[query]
    fn configuration() -> StdResult<ConfigResponse> {
        Ok(ConfigResponse {
            distribution: State::load_distribution(deps)?,
            mgmt: State::load_mgmt(deps)?,
            portion: State::load_portion(deps)?,
            token: State::load_token(deps)?,
        })
    }

    #[handle_guard]
    fn guard(msg: &HandleMsg) -> StdResult<()> {
        let operational = killswitch::is_operational(deps);

        if operational.is_err() && matches!(msg, HandleMsg::Killswitch(_) | HandleMsg::Admin(_)) {
            Ok(())
        } else {
            operational
        }
    }

    #[handle]
    #[require_admin]
    fn configure(
        portion: Portion,
        distribution: Distribution<HumanAddr>,
    ) -> StdResult<HandleResponse> {
        validate(portion, &distribution)?;

        State::save_portion(deps, portion)?;
        State::save_distribution(deps, distribution)?;

        Ok(HandleResponse::default())
    }

    #[handle]
    fn vest() -> StdResult<HandleResponse> {
        let mgmt = State::load_mgmt(deps)?;
        let token = State::load_token(deps)?;
        let portion = State::load_portion(deps)?.u128();
        let distribution = State::load_distribution(deps)?;
        let claimable = query_claimable(&deps, &env, &mgmt)?.u128();
        let portions = claimable / portion;
        let remainder = claimable % portion;

        let messages = build_messages(distribution, portions, token, mgmt)?;

        let mut logs = vec![];
        if remainder > 0 {
            logs.push(LogAttribute {
                key: "remainder (locked forever)".to_string(),
                value: remainder.to_string(),
                encrypted: false,
            })
        }

        Ok(HandleResponse {
            data: None,
            log: logs,
            messages,
        })
    }
}

fn query_claimable<S, A, Q>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    mgmt: &ContractLink<HumanAddr>,
) -> StdResult<Uint128>
where
    S: Storage,
    A: Api,
    Q: Querier,
{
    let mut msg = to_binary(&MGMTQuery::Progress {
        address: env.contract.address.clone(),
        time: env.block.time,
    })?;

    space_pad(&mut msg.0, BLOCK_SIZE);

    let progress = deps
        .querier
        .query::<ProgressResponse>(&fadroma::QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: mgmt.address.clone(),
            callback_code_hash: mgmt.code_hash.clone(),
            msg,
        }))?;
    Ok(progress.unlocked.sub(progress.claimed)?)
}

#[cfg(feature = "batch_transfer")]
fn build_messages(
    distribution: Distribution<HumanAddr>,
    portions: u128,
    token: ContractLink<HumanAddr>,
    mgmt: ContractLink<HumanAddr>,
) -> StdResult<Vec<CosmosMsg>> {
    let transfers = distribution
        .0
        .iter()
        .map(|(addr, amount)| {
            snip20::batch::TransferAction::new(
                addr.clone(),
                Uint128(amount.u128() * portions),
                None,
            )
        })
        .collect::<Vec<_>>();

    let batch_transfer_msg = snip20::batch_transfer_msg(
        transfers,
        None,
        BLOCK_SIZE,
        token.code_hash.clone(),
        token.address.clone(),
    )?;

    let claim_msg = to_cosmos_msg(
        mgmt.address.clone(),
        mgmt.code_hash.clone(),
        &MGMTHandle::Claim {},
    )?;

    Ok(vec![claim_msg, batch_transfer_msg])
}
#[cfg(not(feature = "batch_transfer"))]
fn build_messages(
    distribution: Distribution<HumanAddr>,
    portions: u128,
    token: ContractLink<HumanAddr>,
    mgmt: ContractLink<HumanAddr>,
) -> StdResult<Vec<CosmosMsg>> {
    let mut transfers = distribution
        .0
        .iter()
        .map(|(addr, amount)| {
            let transfer = snip20::transfer_msg(
                addr.clone(),
                Uint128(amount.u128() * portions),
                None,
                None,
                BLOCK_SIZE,
                token.code_hash.clone(),
                token.address.clone(),
            )
            .unwrap();

            transfer
        })
        .collect::<Vec<_>>();

    let claim_msg = to_cosmos_msg(
        mgmt.address.clone(),
        mgmt.code_hash.clone(),
        &MGMTHandle::Claim {},
    )?;

    let mut messages = vec![claim_msg];
    messages.append(&mut transfers);

    Ok(messages)
}

fn validate<T>(portion: Uint128, config: &Distribution<T>) -> StdResult<()> {
    let total = sum_config(&config);
    if portion == total {
        Ok(())
    } else {
        Err(StdError::GenericErr {
            msg: RPTError!(TOTAL: portion, total),
            backtrace: None,
        })
    }
}

fn sum_config<T>(map: &LinearMap<T, Uint128>) -> Uint128 {
    let mut total = Uint128::zero();
    for (_, amount) in map.0.iter() {
        total += *amount;
    }
    total
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigResponse {
    pub portion: Uint128,
    pub distribution: Distribution<HumanAddr>,
    pub token: ContractLink<HumanAddr>,
    pub mgmt: ContractLink<HumanAddr>,
}
