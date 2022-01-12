use lend_shared::fadroma::{
    log, secret_toolkit::snip20, to_binary, Api, Binary, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128, BLOCK_SIZE,
    uint256::Uint256,
};

pub fn deposit_underlying<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    // Borrower id created by overseer
    borrower: Binary,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    unimplemented!()
}


pub fn withdraw_underlying<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    // Borrower id created by overseer
    borrower: Binary,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    unimplemented!()
}