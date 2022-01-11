use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Extern, Storage, Api, Querier, StdResult,
            Uint128, HumanAddr, StdError
        },
        storage::{save, load},
        permit::Permit,
        Uint256
    },
    interfaces::overseer::{
        OverseerPermissions, query_account_liquidity
    }
};

use crate::Config;

// TODO: Move to state.rs
// **************************************************************
pub struct GlobalData;

impl GlobalData {
    const KEY_BORROW_CAP: &'static[u8] = b"borrow_cap";
    const KEY_TOTAL_BORROWS: &'static[u8] = b"total_borrows";

    #[inline]
    pub fn save_borrow_cap(
        storage: &mut impl Storage,
        borrow_cap: &Uint128
    ) -> StdResult<()> {
        save(storage, Self::KEY_BORROW_CAP, borrow_cap)
    }
    
    #[inline]
    pub fn load_borrow_cap(
        storage: &impl Storage,
    ) -> StdResult<Option<Uint128>> {
        load(storage, Self::KEY_BORROW_CAP)
    }
    
    pub fn increase_total_borrows(
        storage: &mut impl Storage,
        amount: Uint128
    ) -> StdResult<Uint128> {
        let current = Self::load_total_borrows(storage)?;
        let new = current.0.checked_add(amount.0).ok_or_else(||
            StdError::generic_err("Total borrows amount overflowed.")
        )?;

        let new = Uint128(new);
        Self::save_total_borrows(storage, &new)?;

        Ok(new)
    }
    
    pub fn decrease_borrow_cap(
        storage: &mut impl Storage,
        amount: Uint128
    ) -> StdResult<Uint128> {
        let current = Self::load_total_borrows(storage)?;
        let new = (current - amount)?;

        Self::save_total_borrows(storage, &new)?;

        Ok(new)
    }

    #[inline]
    pub fn load_total_borrows(storage: &impl Storage) -> StdResult<Uint128> {
        Ok(load(storage, Self::KEY_TOTAL_BORROWS)?.unwrap_or_default())
    }

    #[inline]
    fn save_total_borrows(
        storage: &mut impl Storage,
        total: &Uint128
    ) -> StdResult<()> {
        save(storage, Self::KEY_TOTAL_BORROWS, total)
    }
}
// **************************************************************

pub fn assert_borrow_allowed<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    permit: Permit<OverseerPermissions>,
    self_addr: HumanAddr,
    amount: Uint128
) -> StdResult<()> {
    // Is this here really needed?
    // https://github.com/compound-finance/compound-protocol/blob/4a8648ec0364d24c4ecfc7d6cae254f55030d65f/contracts/Comptroller.sol#L347-L363

    if let Some(cap) = GlobalData::load_borrow_cap(&deps.storage)? {
        let total = GlobalData::load_total_borrows(&deps.storage)?;

        let new = total.0.checked_add(amount.0).ok_or_else(||
            StdError::generic_err("Total borrows amount overflowed.")
        )?;

        if new > cap.0 {
            return Err(StdError::generic_err("The market borrow cap has been reached."));
        }
    }

    let config = Config::load(deps)?;

    let liquidity = query_account_liquidity(
        &deps.querier,
        config.overseer_contract,
        permit,
        Some(self_addr),
        Uint128::zero(),
        amount
    )?;

    if liquidity.shortfall > Uint256::zero() {
        Err(StdError::generic_err("Insufficient liquidity."))
    } else {
        Ok(())
    }
}
