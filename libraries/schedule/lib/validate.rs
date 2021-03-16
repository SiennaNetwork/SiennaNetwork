//! # Input validation
//!
//! The `Schedule`, `Pool`, and `Account` structs
//! implement the `Validate` trait, providing a `validate` method.
//! * `Schedule`
//! * `Pool`
//! * `Account`
//!
//! Unfortunately, `rustdoc` does not allow for the `impl`s that are defined
//! in this module to be rendered on this doc page, because they implement
//! `struct`s defined in another file.
//!
//! Documentation of the methods (and errors) defined in this file
//! can be found in the documentation for those structs.
//!
//! ## Layers of validation:
//!
//! 1. The schema, representing the vesting schedule in terms of the structs
//!    defined by this crate. This is deserialized from a static input;
//!    any deviations from the schema cause the input to be rejected.
//!
//! 2. The `validate` module, which checks that sums don't exceed totals.
//!
//! 3. The runtime assertions in the `vesting`, which prevent
//!    invalid configurations from generating output.
//!
//! 4. For a runnihttps://github.com/qarmin/czkawkang contract, valid outputs are further filtered by the
//!    `history` module, which rejects configurations that change already
//!    vested/claimed portions.

use crate::*;

/// Trait for something that undergoes validation, returning `Ok` or an error.
pub trait Validate {
    /// Default implementation is a no-op
    fn validate (&self) -> UsuallyOk { Ok(()) }
}
impl Validate for Schedule {
    /// Schedule must contain valid pools that add up to the schedule total
    fn validate (&self) -> UsuallyOk {
        for pool in self.pools.iter() {
            pool.validate()?;
        }
        if self.subtotal() != self.total.u128() {
            return self.err_total()
        }
        Ok(())
    }
}
impl Validate for Pool {
    fn validate (&self) -> UsuallyOk {
        for account in self.accounts.iter() {
            account.validate()?;
        }
        let invalid_total = if self.partial {
            self.subtotal() > self.total.u128()
        } else {
            self.subtotal() != self.total.u128()
        };
        if invalid_total { return self.err_total() }
        Ok(())
    }
}
impl Validate for Account {
    fn validate (&self) -> UsuallyOk {
        if self.amount == Uint128::zero() {
            return self.err_empty()
        }
        if self.cliff > self.amount {
            return self.err_cliff_too_big()
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use cosmwasm_std::HumanAddr;
    use crate::{Schedule, Pool, Account, Validate};
    #[test] fn test_amount_eq_zero () {
        let A = Account::periodic("A", &HumanAddr::from(""), 0, 0, 0, 0, 0);
        assert_eq!(A.validate(),
                   A.err_empty());
        assert_eq!(Schedule::new(&[Pool::full("P", &[A.clone()])]).validate(),
                   A.err_empty());
        assert_eq!(Schedule::new(&[Pool::partial("P", 0, &[A.clone()])]).validate(),
                   A.err_empty());
        assert_eq!(Schedule::new(&[Pool::partial("P", 1, &[A.clone()])]).validate(),
                   A.err_empty());
    }
    #[test] fn test_cliff_gt_amount () {
        let A = Account::periodic("A", &HumanAddr::from(""), 1, 2, 0, 0, 0);
        assert_eq!(A.validate(),
                   A.err_cliff_too_big());
        assert_eq!(Schedule::new(&[Pool::full("P", &[A.clone()])]).validate(),
                   A.err_cliff_too_big());
        assert_eq!(Schedule::new(&[Pool::partial("P", 0, &[A.clone()])]).validate(),
                   A.err_cliff_too_big());
        assert_eq!(Schedule::new(&[Pool::partial("P", 1, &[A.clone()])]).validate(),
                   A.err_cliff_too_big());
    }
    #[test] fn test_account_gt_pool () {
        let A = Account::periodic("A", &HumanAddr::from(""), 2, 0, 0, 0, 0);
        let P = Pool{
            partial:  false,
            name:     "P".to_string(),
            total:    1u128.into(),
            accounts: vec![A.clone()],
        };
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(A.validate(),
                   Ok(()));
        assert_eq!(S.validate(),
                   P.err_total());
        assert_eq!(Schedule::new(&[Pool::partial("P", 1, &[A.clone()])]).validate(),
                   P.err_total());
    }
    #[test] fn test_pools_lt_schedule () {
        let S = Schedule {
            total: 1u128.into(),
            pools: vec![]
        };
        assert_eq!(S.validate(),
                   S.err_total());
    }
    #[test] fn test_pools_gt_schedule () {
        let A = Account::periodic("A", &HumanAddr::from(""), 2, 0, 0, 0, 0);
        let S = Schedule {
            total: 1u128.into(),
            pools: vec![Pool::partial("P1", 1, &[]), Pool::full("P2", &[A])]
        };
        assert_eq!(S.validate(),
                   S.err_total());
    }
}
