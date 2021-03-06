use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Storage, Api, Querier, Extern,
            StdResult, StdError, HumanAddr,
            Env, HandleResponse, log
        },
        secret_toolkit::snip20,
        Decimal256, Uint256, BLOCK_SIZE
    },
    interfaces::overseer::query_can_transfer,
    core::MasterKey
};

use crate::accrue_interest;
use crate::state::{
    Constants, Contracts, Account, TotalSupply
};
use crate::ops::LatestInterest;
use crate::checks;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    mut interest: LatestInterest,
    underlying_balance: Uint256,
    from: HumanAddr,
    amount: Uint256
) -> StdResult<HandleResponse> {
    let exchange_rate = calc_exchange_rate(
        deps,
        underlying_balance,
        interest.total_borrows(&deps.storage)?,
        interest.total_reserves(&deps.storage)?
    )?;
    let mint_amount = Uint256::from(amount)
        .decimal_div(exchange_rate)?;

    TotalSupply::increase(&mut deps.storage, mint_amount)?;

    let account = Account::of(deps, &from)?;
    account.add_balance(&mut deps.storage, mint_amount)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "deposit"),
            log("mint_amount", mint_amount)
        ],
        data: None
    })
}

pub fn redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    env: Env,
    from_sl_token: Uint256,
    from_underlying: Uint256
) -> StdResult<HandleResponse> {
    let underlying_asset = Contracts::load_underlying(deps)?;

    let balance = snip20::balance_query(
        &deps.querier,
        env.contract.address.clone(),
        Constants::load_vk(&deps.storage)?,
        BLOCK_SIZE,
        underlying_asset.code_hash.clone(),
        underlying_asset.address.clone(),
    )?.amount;

    let mut latest = accrue_interest(deps, env.block.height, balance)?;

    let exchange_rate = calc_exchange_rate(
        deps,
        balance.into(),
        latest.total_borrows(&deps.storage)?,
        latest.total_reserves(&deps.storage)?
    )?;

    let (redeem_amount, burn_amount) = if from_sl_token > Uint256::zero() {
        let redeem_amount = from_sl_token.decimal_mul(exchange_rate)?;

        (redeem_amount, from_sl_token)
    } else {
        let burn_amount = from_underlying.decimal_div(exchange_rate)?;

        (from_underlying, burn_amount)
    };

    checks::assert_can_withdraw(balance.into(), redeem_amount)?;

    let can_transfer = query_can_transfer(
        &deps.querier,
        Contracts::load_overseer(deps)?,
        MasterKey::load(&deps.storage)?,
        env.message.sender.clone(),
        env.contract.address,
        env.block.height,
        burn_amount.clamp_u128()?.into()
    )?;

    if !can_transfer {
        return Err(StdError::generic_err("Account has negative liquidity and cannot redeem."));
    }

    TotalSupply::decrease(&mut deps.storage, burn_amount)?;

    let account = Account::of(deps, &env.message.sender)?;
    account.subtract_balance(&mut deps.storage, burn_amount)?;

    Ok(HandleResponse {
        messages: vec![snip20::transfer_msg(
            env.message.sender,
            redeem_amount.clamp_u128()?.into(),
            None,
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
    balance: Uint256,
    total_borrows: Uint256,
    total_reserves: Uint256
) -> StdResult<Decimal256> {
    let total_supply = TotalSupply::load(&deps.storage)?;

    if total_supply.is_zero() {
        let config = Constants::load_config(&deps.storage)?;

        return Ok(config.initial_exchange_rate);
    }

    let total_minus_reserves = ((balance + total_borrows)? - total_reserves)?;

    Decimal256::from_ratio(total_minus_reserves.0, Uint256::from(total_supply).0)
}
