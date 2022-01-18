use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            StdResult, Storage, Api, Querier,
            Extern, Uint128, StdError
        },
        Uint256, Decimal256
    },
    interfaces::interest_model::query_borrow_rate
};

use crate::MAX_BORROW_RATE;
use crate::state::{
    BorrowSnapshot, Global, TotalBorrows, Contracts, Constants};

pub struct AccruedInterest {
    pub total_borrows: Uint256,
    pub total_reserves: Uint256,
    pub borrow_index: Decimal256
}

impl BorrowSnapshot {
    pub fn current_balance(&self, borrow_index: Decimal256) -> StdResult<Uint256> {
        if self.0.principal.is_zero() {
            return Ok(Uint256::zero());
        }

        self.0.principal
            .decimal_mul(borrow_index)?
            .decimal_div(self.0.interest_index)
    }

    pub fn add_balance(
        &mut self,
        borrow_index: Decimal256,
        amount: Uint256
    ) -> StdResult<()> {
        let balance = self.current_balance(borrow_index)?;

        self.0.principal = (balance + amount)?;
        self.0.interest_index = borrow_index;

        Ok(())
    }

    pub fn subtract_balance(
        &mut self,
        borrow_index: Decimal256,
        amount: Uint256
    ) -> StdResult<()> {
        let balance = self.current_balance(borrow_index)?;

        self.0.principal = balance.0.saturating_sub(amount.0).into();
        self.0.interest_index = borrow_index;

        Ok(())
    }
}

pub fn accrue_interest<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    current_block: u64,
    balance_prior: Uint128
) -> StdResult<()> {
    let result = calc_accrued_interest(deps, current_block, balance_prior)?;

    if let Some(interest) = result {
        TotalBorrows::save(&mut deps.storage, &interest.total_borrows)?;
    
        Global::save_interest_reserve(&mut deps.storage, &interest.total_reserves)?;
        Global::save_borrow_index(&mut deps.storage, &interest.borrow_index)?;
        Global::save_accrual_block_number(&mut deps.storage, current_block)?;
    }

    Ok(())
}

/// Calculate accrued interest for the given block. If no block is supplied,
/// loads the last cached values from storage.
pub fn accrued_interest_at<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    block: Option<u64>,
    balance_prior: Uint128
) -> StdResult<AccruedInterest> {
    let result = if let Some(block) = block {
        calc_accrued_interest(deps, block, balance_prior)?
    } else {
        None
    };

    if let Some(interest) = result {
        Ok(interest)
    } else {
        Ok(AccruedInterest {
            total_borrows: TotalBorrows::load(&deps.storage)?,
            total_reserves: Global::load_interest_reserve(&deps.storage)?,
            borrow_index: Global::load_borrow_index(&deps.storage)?
        })
    }
}

fn calc_accrued_interest<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    current_block: u64,
    balance_prior: Uint128
) -> StdResult<Option<AccruedInterest>> {
    let config = Constants::load(deps)?;
    // Initial block number
    let last_accrual_block = Global::load_accrual_block_number(&deps.storage)?;

    if last_accrual_block == current_block {
        return Ok(None);
    }

    // Previous values from storage
    let borrows_prior = TotalBorrows::load(&deps.storage)?;
    let reserves_prior = Global::load_interest_reserve(&deps.storage)?;
    let borrow_index_prior = Global::load_borrow_index(&deps.storage)?;

    // Current borrow interest rate
    let interest_model = Contracts::load_interest_model(deps)?;
    let borrow_rate = query_borrow_rate(
        &deps.querier,
        interest_model,
        Decimal256::from_uint256(balance_prior)?,
        Decimal256::from_uint256(borrows_prior)?,
        Decimal256::from_uint256(reserves_prior)?,
    )?;

    if borrow_rate >= MAX_BORROW_RATE {
        // TODO: Should this be capped instead of returning an error?
        return Err(StdError::generic_err("Borrow rate is absurdly high"));
    }

    // Calculate the number of blocks elapsed since last accrual
    let block_delta = current_block
        .checked_sub(last_accrual_block)
        .ok_or_else(|| StdError::generic_err(format!(
            "Current block must be equal or bigger than {}",
            last_accrual_block
        )))?;

    let simple_interest_factor = (borrow_rate * Decimal256::from_uint256(block_delta)?)?;
    let interest_accumulated = borrows_prior.decimal_mul(simple_interest_factor)?;

    Ok(Some(AccruedInterest {
        total_borrows: (interest_accumulated + borrows_prior)?,
        total_reserves: (interest_accumulated.decimal_mul(config.reserve_factor)? + reserves_prior)?,
        borrow_index: ((borrow_index_prior * simple_interest_factor)? + borrow_index_prior)?
    }))
}
