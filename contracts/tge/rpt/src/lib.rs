use fadroma::{
    admin,
    cosmwasm_std::{self, HandleResponse, InitResponse, StdResult, Uint128},
    derive_contract::{contract, init},
    schemars,
};
use fadroma::{derive_contract::*, killswitch};
pub use linear_map::LinearMap;
pub use sienna_mgmt::{HandleMsg as MGMTHandle, ProgressResponse, QueryMsg as MGMTQuery};
pub use sienna_migration::{can_set_status, is_operational, ContractStatus, ContractStatusLevel};

/// Default value for Secret Network block size (used for padding)

/// Into what parts to split the received amount
pub type Config<T> = LinearMap<T, Uint128>;

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
    fn new() -> StdResult<InitResponse> {
        Ok(InitResponse::default())
    }
    #[handle]
    fn configure() -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }
    #[handle]
    fn vest() -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }
}

// fn query_claimable<S, A, Q>(
//     deps: &Extern<S, A, Q>,
//     env: &Env,
//     mgmt: &ContractLink<CanonicalAddr>,
// ) -> StdResult<Uint128>
// where
//     S: Storage,
//     A: Api,
//     Q: Querier,
// {
//     let msg = MGMTQuery::Progress {
//         address: env.contract.address.clone(),
//         time: env.block.time,
//     };
//     let mut msg = to_binary(&msg)?;
//     space_pad(&mut msg.0, BLOCK_SIZE);
//     let response =
//         deps.querier
//             .query::<ProgressResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
//                 contract_addr: deps.api.human_address(&mgmt.0)?,
//                 callback_code_hash: mgmt.1.clone(),
//                 msg,
//             }))?;

//     Ok((response.unlocked - response.claimed)?)
// }

// fn validate<T>(portion: Uint128, config: &Config<T>) -> StdResult<()> {
//     let total = sum_config(&config);
//     if portion == total {
//         Ok(())
//     } else {
//         Err(StdError::GenericErr {
//             msg: RPTError!(TOTAL: portion, total),
//             backtrace: None,
//         })
//     }
// }

// fn sum_config<T>(map: &LinearMap<T, Uint128>) -> Uint128 {
//     let mut total = Uint128::zero();
//     for (_, amount) in map.0.iter() {
//         total += *amount;
//     }
//     total
// }

// fn is_admin<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     env: &Env,
//     state: &State,
// ) -> StdResult<()> {
//     if state.admin == deps.api.canonical_address(&env.message.sender)? {
//         Ok(())
//     } else {
//         Err(StdError::Unauthorized { backtrace: None })
//     }
// }

// fn transfer<A: Api>(
//     api: &A,
//     state: &State,
//     recipient: &CanonicalAddr,
//     amount: Uint128,
// ) -> StdResult<CosmosMsg> {
//     let (token_addr, token_hash) = &state.token;
//     let token_addr = api.human_address(&token_addr)?;
//     let recipient = api.human_address(&recipient)?;
//     snip20::transfer_msg(
//         recipient,
//         amount,
//         None,
//         None,
//         BLOCK_SIZE,
//         token_hash.clone(),
//         token_addr,
//     )
// }

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
#[macro_use]
extern crate wasm_bindgen;

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
mod wasm {
    fadroma_bind_js::bind_js!(cosmwasm_std, crate);
}
