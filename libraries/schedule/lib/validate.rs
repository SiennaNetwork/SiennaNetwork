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

/// Trait for something that undergoes validation
/// returning `Ok` or an error.
pub trait Validate {
    /// Default implementation is a no-op
    fn validate (&self) -> StdResult<()> { Ok(()) }
}

impl Validate for Schedule {
    /// Schedule must contain valid pools that add up to the schedule total
    fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for pool in self.pools.iter() {
            match pool.validate() {
                Ok(_)  => { total += pool.total.u128() },
                Err(e) => return Err(e)
            }
        }
        if total != self.total.u128() {
            return Self::err_total(total, self.total)
        }
        Ok(())
    }
}
impl Schedule {
    define_errors!{
        err_total (actual: u128, expected: Uint128) ->
            ("schedule: pools add up to {}, expected {}",
                actual, expected)}
}

impl Validate for Pool {
    fn validate (&self) -> StdResult<()> {
        let total = self.accounts_total()?;
        let invalid_total = if self.partial {
            total > self.total.u128()
        } else {
            total != self.total.u128()
        };
        if invalid_total { return Self::err_total(&self.name, total, &self.total) }
        Ok(())
    }
}
impl Pool {
    define_errors!{
        err_total (name: &str, actual: u128, expected: &Uint128) ->
            ("pool {}: accounts add up to {}, expected {}",
                name, actual, expected)}
}

impl Validate for Account {
    fn validate (&self) -> StdResult<()> {
        //match &self.allocations {
            //AccountConfig::Immediate(config) => {
                //config.validate()?;
            //},
            //AccountConfig::Periodic(config) => {
                //for (_, periodic, allocations) in config.iter() {
                    //periodic.validate(&self)?;
                    //allocations.validate()?;
                //}
            //}
        //}
        Ok(())
    }
}
impl Account {
    pub fn validate_periodic (&self, acc: &Account) -> StdResult<()> {
        let &Account{cliff,duration,interval,..} = self;
        if duration < 1u64 { return Self::err_zero_duration(&acc.name) }
        if interval < 1u64 { return Self::err_zero_interval(&acc.name) }
        if cliff > acc.amount { return Self::err_cliff_gt_total(&acc.name, &cliff, &acc.amount) }
        Ok(())
    }
    define_errors!{
        err_zero_duration (name: &str) ->
            ("account {}: periodic vesting's duration can't be 0",
                name)
        err_zero_interval (name: &str) ->
            ("account {}: periodic vesting's interval can't be 0",
                name)
        err_cliff_gt_total (name: &str, cliff: &Uint128, total: &Uint128) ->
            ("account {}: cliff ({}) can't be larger than total ({})",
                name, cliff, total)
    }
}
