use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Storage, Api, Querier, Extern,
            StdResult, StdError, HumanAddr,
            Env, HandleResponse, log
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
use crate::state::{Global, Config, Contracts, Account, TotalSupply, TotalBorrows};
use crate::checks;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    env: Env,
    underlying_asset: ContractLink<HumanAddr>,
    from: HumanAddr,
    amount: Uint256
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

    let exchange_rate = calc_exchange_rate(deps, balance.into())?;
    let mint_amount = Uint256::from(amount)
        .decimal_div(exchange_rate)?;

    TotalSupply::increase(&mut deps.storage, mint_amount)?;

    let account = Account::new(deps, &from)?;
    account.add_balance(&mut deps.storage, mint_amount)?;

    Ok(HandleResponse::default())
}

pub fn redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    env: Env,
    permit: Permit<OverseerPermissions>,
    from_sl_token: Uint256,
    from_underlying: Uint256
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

    let exchange_rate = calc_exchange_rate(deps, balance.into())?;

    let (redeem_amount, burn_amount) = if from_sl_token > Uint256::zero() {
        let redeem_amount = Uint256::from(from_sl_token).decimal_mul(exchange_rate)?;

        (redeem_amount, from_sl_token)
    } else {
        let burn_amount = Uint256::from(from_underlying).decimal_div(exchange_rate)?;

        (from_underlying, burn_amount)
    };

    checks::assert_can_withdraw(balance.into(), redeem_amount)?;

    let can_transfer = query_can_transfer(
        &deps.querier,
        Contracts::load_overseer(deps)?,
        permit,
        env.contract.address,
        burn_amount.clamp_u128()?.into()
    )?;

    if !can_transfer {
        return Err(StdError::generic_err("Account has negative liquidity and cannot redeem."));
    }

    TotalSupply::decrease(&mut deps.storage, burn_amount)?;

    let account = Account::new(deps, &env.message.sender)?;
    account.subtract_balance(&mut deps.storage, burn_amount)?;

    Ok(HandleResponse {
        messages: vec![snip20::transfer_msg(
            env.message.sender,
            redeem_amount.clamp_u128()?.into(),
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
    balance: Uint256
) -> StdResult<Decimal256> {
    let total_supply = TotalSupply::load(&deps.storage)?;

    if total_supply.is_zero() {
        let config = Config::load(deps)?;

        return Ok(config.initial_exchange_rate);
    }

    let total_borrows = TotalBorrows::load(&deps.storage)?;
    let total_reserves = Global::load_interest_reserve(&deps.storage)?;

    let total_minus_reserves = ((balance + total_borrows)? - total_reserves)?;

    Decimal256::from_ratio(total_minus_reserves.0, Uint256::from(total_supply).0)
}
