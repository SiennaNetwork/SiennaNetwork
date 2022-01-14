use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Storage, Api, Querier, Extern,
            Uint128,StdResult, StdError,
            HumanAddr, Env, HandleResponse,
            log
        },
        permit::Permit,
        secret_toolkit::snip20,
        Decimal256, Uint256, ContractLink, BLOCK_SIZE
    },
    interfaces::{
        overseer::{OverseerPermissions, query_can_transfer}
    }
};

use crate::{accrue_interest, VIEWING_KEY};
use crate::state::{GlobalData, Config, Contracts, Account};
use crate::checks;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    env: Env,
    underlying_asset: ContractLink<HumanAddr>,
    from: HumanAddr,
    amount: Uint128
) -> StdResult<HandleResponse> {
    let balance = snip20::balance_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.to_string(),
        BLOCK_SIZE,
        underlying_asset.code_hash,
        underlying_asset.address,
    )?.amount;

    accrue_interest(deps, env.block.height, balance)?;

    let exchange_rate = calc_exchange_rate(deps, balance)?;
    let mint_amount: Uint128 = Uint256::from(amount)
        .decimal_div(exchange_rate)?
        .clamp_u128()?
        .into();

    GlobalData::increase_total_supply(&mut deps.storage, mint_amount)?;

    let account = Account::new(deps, &from)?;
    account.add_balance(&mut deps.storage, mint_amount)?;

    Ok(HandleResponse::default())
}

pub fn redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    env: Env,
    permit: Permit<OverseerPermissions>,
    from_sl_token: Uint128,
    from_underlying: Uint128
) -> StdResult<HandleResponse> {
    let underlying_asset = Contracts::load_underlying(deps)?;

    let balance = snip20::balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.to_string(),
        BLOCK_SIZE,
        underlying_asset.code_hash.clone(),
        underlying_asset.address.clone(),
    )?.amount;

    accrue_interest(deps, env.block.height, balance)?;

    let exchange_rate = calc_exchange_rate(deps, balance)?;

    let (redeem_amount, burn_amount) = if from_sl_token > Uint128::zero() {
        let redeem_amount = Uint256::from(from_sl_token).decimal_mul(exchange_rate)?;

        (Uint128(redeem_amount.clamp_u128()?), from_sl_token)
    } else {
        let burn_amount = Uint256::from(from_underlying).decimal_div(exchange_rate)?;

        (from_underlying, Uint128(burn_amount.clamp_u128()?))
    };

    checks::assert_can_withdraw(balance, redeem_amount)?;

    let can_transfer = query_can_transfer(
        &deps.querier,
        Contracts::load_overseer(deps)?,
        permit,
        env.contract.address,
        burn_amount
    )?;

    if !can_transfer {
        return Err(StdError::generic_err("Account has negative liquidity and cannot redeem."));
    }

    GlobalData::decrease_total_supply(&mut deps.storage, burn_amount)?;

    let account = Account::new(deps, &env.message.sender)?;
    account.subtract_balance(&mut deps.storage, burn_amount)?;

    Ok(HandleResponse {
        messages: vec![snip20::transfer_msg(
            env.message.sender,
            redeem_amount,
            None,
            BLOCK_SIZE,
            underlying_asset.code_hash,
            underlying_asset.address
        )?],
        log: vec![
            log("action", "redeem"),
            log("redeem_amount", redeem_amount),
            log("burn_amount", burn_amount)
        ],
        data: None
    })
}

pub fn calc_exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    balance: Uint128
) -> StdResult<Decimal256> {
    let total_supply = GlobalData::load_total_supply(&deps.storage)?;

    if total_supply.is_zero() {
        let config = Config::load(deps)?;

        return Ok(config.initial_exchange_rate);
    }

    let total_borrows = GlobalData::load_total_borrows(&deps.storage)?.0;
    let total_reserves = GlobalData::load_total_reserves(&deps.storage)?.0;

    let total_minus_reserves = balance.0.checked_add(total_borrows)
        .and_then(|x|
            x.checked_sub(total_reserves)
        )
        .ok_or_else(||
            StdError::generic_err("Math overflow while calculating exchange rate.")
        )?;

    Decimal256::from_ratio(total_minus_reserves, total_supply.0)
}
