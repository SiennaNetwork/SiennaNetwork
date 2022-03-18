use std::ops::Sub;

use fadroma::{
    admin,
    admin::assert_admin,
    cosmwasm_std::{self, HandleResponse, InitResponse, StdResult, Uint128},
    derive_contract::{contract, init},
    require_admin, schemars,
    secret_toolkit::snip20,
    space_pad, to_binary, Api, Binary, ContractLink, CosmosMsg, Env, Extern, HumanAddr,
    LogAttribute, Querier, StdError, Storage, WasmMsg, WasmQuery, BLOCK_SIZE,
};
pub mod state;
use fadroma::{derive_contract::*, killswitch};
pub use linear_map::LinearMap;
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

#[contract(component(path = "admin"), component(path = "killswitch"))]
pub trait RPT {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        portion: Portion,
        token: ContractLink<HumanAddr>,
        mgmt: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse> {
        State::save_portion(deps, portion)?;
        State::save_token(deps, token)?;
        State::save_mgmt(deps, mgmt)?;

        let response = admin::Admin::new(&admin::DefaultImpl, admin, deps, env)?;

        Ok(response)
    }
    #[handle_guard]
    fn guard(_msg: &HandleMsg) -> StdResult<()> {
        killswitch::is_operational(deps)
    }

    #[handle]
    #[require_admin]
    fn set_distribution(distribution: Distribution<HumanAddr>) -> StdResult<HandleResponse> {
        validate(State::load_portion(deps)?, &distribution)?;

        State::save_distribution(deps, distribution)?;

        Ok(HandleResponse::default())
    }
    #[handle]
    fn vest() -> StdResult<HandleResponse> {
        let mgmt = State::load_mgmt(deps)?;
        let token = State::load_token(deps)?;
        let portion = State::load_portion(deps)?.u128();
        let distribution = State::load_distribution(deps)?;
        let mut msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: mgmt.address.clone(),
            callback_code_hash: mgmt.code_hash.clone(),
            send: vec![],
            msg: claim_msg()?,
        })];

        let claimable = query_claimable(&deps, &env, &mgmt)?.u128();
        let portions = claimable / portion;
        let remainder = claimable % portion;

        for (addr, amount) in distribution.0.iter() {
            let msg = snip20::transfer_msg(
                addr.clone(),
                Uint128(amount.u128() * portions),
                None,
                None,
                BLOCK_SIZE,
                token.code_hash.clone(),
                token.address.clone(),
            )?;
            msgs.push(msg);
        }

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
            messages: msgs,
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

fn claim_msg() -> StdResult<Binary> {
    let mut msg = to_binary(&MGMTHandle::Claim {})?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    Ok(msg)
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

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
#[macro_use]
extern crate wasm_bindgen;

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
mod wasm {
    fadroma_bind_js::bind_js!(cosmwasm_std, crate);
}
