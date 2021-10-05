use amm_shared::fadroma::scrt::{
    callback::ContractInstance,
    cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdResult, Storage, Uint128},
    toolkit::snip20,
    utils::viewing_key::ViewingKey,
    BLOCK_SIZE,
};

/// Query the token for number of its decimals
pub(crate) fn get_token_decimals(
    querier: &impl Querier,
    instance: ContractInstance<HumanAddr>,
) -> StdResult<u8> {
    let result =
        snip20::token_info_query(querier, BLOCK_SIZE, instance.code_hash, instance.address)?;

    Ok(result.decimals)
}

/// Query the token for number of its decimals
pub(crate) fn get_token_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    this_contract: HumanAddr,
    instance: ContractInstance<HumanAddr>,
    viewing_key: ViewingKey,
) -> StdResult<Uint128> {
    let balance = snip20::balance_query(
        &deps.querier,
        this_contract,
        viewing_key.0,
        BLOCK_SIZE,
        instance.code_hash,
        instance.address,
    )?;

    Ok(balance.amount)
}
