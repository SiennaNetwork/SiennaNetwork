use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Extern, Storage, Api, Querier,
            StdResult, HumanAddr, StdError
        },
        Uint256
    },
    interfaces::overseer::{
        query_config, query_account_liquidity
    },
    core::MasterKey
};

use crate::state::{Contracts, Global, TotalBorrows};

pub fn assert_borrow_allowed<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    sender: HumanAddr,
    block: u64,
    self_addr: HumanAddr,
    amount: Uint256
) -> StdResult<()> {
    if let Some(cap) = Global::load_borrow_cap(&deps.storage)? {
        let total = TotalBorrows::load(&deps.storage)?;

        let new = total.0.checked_add(amount.0).ok_or_else(||
            StdError::generic_err("Total borrows amount overflowed.")
        )?;

        if new > cap.0 {
            return Err(StdError::generic_err("The market borrow cap has been reached."));
        }
    }

    let liquidity = query_account_liquidity(
        &deps.querier,
        Contracts::load_overseer(deps)?,
        MasterKey::load(&deps.storage)?,
        sender,
        Some(self_addr),
        Some(block),
        Uint256::zero(),
        amount
    )?;

    if liquidity.shortfall > Uint256::zero() {
        Err(StdError::generic_err("Insufficient liquidity."))
    } else {
        Ok(())
    }
}

pub fn assert_liquidate_allowed<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    borrower: HumanAddr,
    borrower_balance: Uint256,
    block: u64,
    amount: Uint256
) -> StdResult<()> {
    if amount == Uint256::zero() {
        return Err(StdError::generic_err("Repay amount cannot be zero."));
    }

    let overseer = Contracts::load_overseer(deps)?;

    let liquidity = query_account_liquidity(
        &deps.querier,
        overseer.clone(),
        MasterKey::load(&deps.storage)?,
        borrower,
        None,
        Some(block),
        Uint256::zero(),
        Uint256::zero()
    )?;

    if liquidity.shortfall == Uint256::zero() {
        return Err(StdError::generic_err("Borrower cannot be liquidated."));
    }

    let config = query_config(
        &deps.querier,
        overseer
    )?;

    let max = borrower_balance.decimal_mul(config.close_factor)?;

    if amount > max {
        Err(StdError::generic_err("Repay amount is too high."))
    } else {
        Ok(())
    }
}

pub fn assert_can_withdraw(
    balance: Uint256,
    amount: Uint256
) -> StdResult<()> {
    if balance < amount {
        Err(StdError::generic_err(format!(
            "The protocol has an insufficient amount of the underlying asset at this time. supply: {}, needed: {}",
            balance,
            amount
        )))
    } else {
        Ok(())
    }
}
