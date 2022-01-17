use lend_shared::{
    fadroma::{
        cosmwasm_std::StdResult,
        Uint256, Decimal256
    }
};

use crate::state::BorrowSnapshot;

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
