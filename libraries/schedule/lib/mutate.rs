use crate::*;
impl Schedule {
    pub fn add_account (&self, pool_name: String, account: Account) -> UsuallyOk {
        unimplemented!()
    }
}
impl Pool {
    pub fn add_account (&mut self, account: Account) -> UsuallyOk {
        if !self.partial {
            return self.err_pool_full()
        }
        let unallocated = self.total - self.subtotal()?
        if account.amount > unallocated {
            return self.err_account_too_big(account.amount, unallocated)
        }
        account.validate()?;
        self.accounts.push(account);
        let unallocated = self.total - self.subtotal()?;
        if unallocated == 0 {
            self.partial = false
        }
        Ok(())
    }
}
