use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Extern, Storage, Api, Querier, StdResult,
            Uint128, HumanAddr, StdError
        },
        permit::Permit,
        Uint256
    },
    interfaces::overseer::{
        OverseerPermissions, query_account_liquidity
    }
};

use crate::state::{Contracts, Global, TotalBorrows};

pub fn assert_borrow_allowed<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    permit: Permit<OverseerPermissions>,
    self_addr: HumanAddr,
    amount: Uint256
) -> StdResult<()> {
    // Is this here really needed?
    // https://github.com/compound-finance/compound-protocol/blob/4a8648ec0364d24c4ecfc7d6cae254f55030d65f/contracts/Comptroller.sol#L347-L363

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
        permit,
        Some(self_addr),
        Uint128::zero(),
        amount.low_u128().into()
    )?;

    if liquidity.shortfall > Uint256::zero() {
        Err(StdError::generic_err("Insufficient liquidity."))
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
