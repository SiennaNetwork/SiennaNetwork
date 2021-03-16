use crate::*;
impl Schedule {
    pub fn add_account (&mut self, pool_name: String, account: Account) -> UsuallyOk {
        for pool in self.pools.iter_mut() {
            if pool.name == pool_name {
                return pool.add_account(account)
            }
        }
        self.err_pool_not_found(pool_name)
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
        self.validate()?;
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
        let mut P = Pool::partial("P", 200, &[Account::immediate("A", &Alice, 100)]);
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
    #[test] fn test_add_to_schedule () {
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let Carol = HumanAddr::from("Carol");
        let mut S = Schedule::new(&[
            Pool::partial("P1", 100, &[]),
            Pool::full("P2", &[Account::immediate("A", &Alice, 100)]),
        ]);
        assert_eq!(S.add_account("P1".to_string(), Account::immediate("B", &Bob, 50)),
                   Ok(()));
        let A = Account::immediate("B", &Bob, 100);
        assert_eq!(S.add_account("P1".to_string(), A.clone()),
                   S.pools.get(0).unwrap().err_account_too_big(&A));
        assert_eq!(S.add_account("P1".to_string(), Account::immediate("C", &Carol, 50)),
                   Ok(()));
        assert_eq!(S.add_account("P1".to_string(), A.clone()),
                   S.pools.get(0).unwrap().err_pool_full());
        assert_eq!(S.add_account("P2".to_string(), A.clone()),
                   S.pools.get(1).unwrap().err_pool_full());
    }
}
