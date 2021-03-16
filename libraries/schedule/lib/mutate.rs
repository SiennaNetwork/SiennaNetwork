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
        if account.amount.u128() > self.unallocated() {
            return self.err_account_too_big(&account)
        }
        account.validate()?;
        self.accounts.push(account);
        if self.unallocated() == 0 {
            self.partial = false
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use cosmwasm_std::HumanAddr;
    use crate::{Schedule, Pool, Account, Validate};
    #[test] fn test_add_to_full () {
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let mut P = Pool::full("P", &[Account::immediate("A", &Alice, 100)]);
        assert_eq!(P.add_account(Account::immediate("B", &Bob, 100)),
                   P.err_pool_full());
    }
    #[test] fn test_add_to_partial_becomes_full () {
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let Carol = HumanAddr::from("Carol");
        let mut P = Pool::partial("P", 100, &[Account::immediate("A", &Alice, 100)]);
        assert_eq!(P.add_account(Account::immediate("B", &Bob, 200)),
                   P.err_account_too_big(&Account::immediate("B", &Bob, 200)));
        assert_eq!(P.partial,
                   true);
        assert_eq!(P.add_account(Account::immediate("B", &Bob, 100)),
                   Ok(()));
        assert_eq!(P.partial,
                   false);
        assert_eq!(P.add_account(Account::immediate("C", &Carol, 1)),
                   P.err_pool_full());
    }
}
