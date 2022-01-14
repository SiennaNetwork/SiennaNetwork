use lend_shared::{
    fadroma::{
        cosmwasm_std::StdResult,
        Uint256, Decimal256
    }
};

use crate::state::BorrowSnapshot;

impl BorrowSnapshot {
    pub fn borrow_balance(&self, borrow_index: Decimal256) -> StdResult<Uint256> {
        if self.0.principal.is_zero() {
            return Ok(Uint256::zero());
        }

        self.0.principal
            .decimal_mul(borrow_index)?
            .decimal_div(self.0.interest_index)
    }
}
