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
        let mut total = 0u128;
        for pool in self.pools.iter() {
            pool.validate()?;
            total += pool.total.u128();
        }
        if total != self.total.u128() {
            return Self::err_total(total, self.total)
        }
        Ok(())
    }
}
impl Validate for Pool {
    fn validate (&self) -> UsuallyOk {
        let total = self.subtotal()?;
        let invalid_total = if self.partial {
            total > self.total.u128()
        } else {
            total != self.total.u128()
        };
        if invalid_total { return Self::err_total(&self.name, total, &self.total) }
        Ok(())
    }
}
impl Validate for Account {
    fn validate (&self) -> UsuallyOk {
        if self.cliff > self.amount {
            return self.err_cliff_too_big()
        }
        Ok(())
    }
}
