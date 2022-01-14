use lend_shared::{
    fadroma::{
        cosmwasm_std::StdResult,
        Uint256
    }
};

use crate::state::BorrowSnapshot;

impl BorrowSnapshot {
    pub fn borrow_balance(&self, borrow_index: Uint256) -> StdResult<Uint256> {
        if self.0.principal.is_zero() {
            return Ok(Uint256::zero());
        }

        (self.0.principal * borrow_index)? / self.0.interest_index
    }
}
