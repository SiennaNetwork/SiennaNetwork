//! # Vesting
//!
//! This module implements the logic that turns a nested configuration into
//! a flat list of `Portion`s, each one describing a transactions that needs
//! to be executed in the future.
//!
//! `AllocationSet` implements the functions:
//! * `vest_immediate`
//! * `vest_head`
//! * `vest_body`
//! * `vest_tail`
//!
//! These output `Portion`s, which are subsequently "bubbled up" all the way
//! to the contract which will execute them, by the `all()` methods of these
//! structs:
//!
//! * `Schedule`
//! * `Pool`
//! * `Account`
//! * `Periodic`
//!
//! All but `Periodic` implement their `all()` method via the `Vesting` trait.
//!
//! Just like `validate`, this module contains more than `rustdoc` renders.
//! Aside from the `Vesting` trait, more info about the aforementioned methods
//! can be found in the docs for the corresponding struct.

use crate::*;

pub trait Vesting {
    /// Get list of all portions that will be unlocked
    fn all (&self) -> UsuallyPortions;
    /// Get amount unlocked for address `a` at time `t`.
    fn claimable_by_at (&self, a: &HumanAddr, t: Seconds) -> UsuallyPortions {
        let mut all = self.all()?;
        all.retain(|Portion{address,vested,..}|address==a&&*vested<=t);
        Ok(all)
    }
}

impl Vesting for Schedule {
    /// Get list of all portions that will be unlocked by this schedule
    fn all (&self) -> UsuallyPortions {
        let mut portions = vec![];
        for pool in self.pools.iter() {
            portions.append(&mut pool.all()?);
        }
        Ok(portions)
    }
}

impl Vesting for Pool {
    /// Get list of all portions that will be unlocked by this pool
    fn all (&self) -> UsuallyPortions {
        let mut portions = vec![];
        for account in self.accounts.iter() {
            portions.append(&mut account.all()?);
        }
        Ok(portions)
    }
}

impl Vesting for Account {
    /// Generate list of all portions that this account will
    /// ever make available, given its history of configurations.
    fn all (&self) -> UsuallyPortions {
        let mut t_cursor = self.start_at;
        let mut all_portions = vec![];
        let mut remaining = self.total.u128();
        self.vest( // 1. vest the head.
            t_cursor,
            &format!("{}: head", &self.name),
            &mut remaining,
            &mut all_portions,
            &self.head_allocations
        )?;
        if remaining == 0u128 || self.interval == 0 {
            return Ok(all_portions)
        }
        loop { // 2. vest the body
            t_cursor += self.interval; // move time forward
            // duration is optional, but if we're past it then the body ends here
            if self.duration > 0u64 && t_cursor > self.start_at + self.duration {
                break
            }
            self.vest( // add portions from body allocations
                t_cursor,
                &format!("{}: body", &self.name),
                &mut remaining,
                &mut all_portions,
                &self.body_allocations
            )?;
            if remaining < sum_allocations(&self.body_allocations) {
                // not enough money for a full body portion, move onto tail
                break
            }
        }
        if remaining > 0 { // 3. vest the tail
            self.vest(
                t_cursor,
                &format!("{}: tail", &self.name),
                &mut remaining,
                &mut all_portions,
                &self.tail_allocations
            )?;
        }
        Ok(all_portions)
    }
}
impl Account {
    fn vest (
        &self,
        t_cursor:    Seconds,
        reason:      &str,
        remaining:   &mut u128,
        portions:    &mut Portions,
        allocations: &Allocations,
    ) -> UsuallyOk {
        for allocation in allocations.iter() {
            if allocation.amount.u128() > *remaining {
                // make sure the account hasn't run out of money
                return self.err_broke();
            }
            if allocation.amount == Uint128::zero() {
                // ignore empty portions
                continue
            }
            let portion = allocation.to_portion(t_cursor, &reason);
            *remaining -= portion.amount.u128();
            portions.push(portion);
        }
        Ok(())
    }
    /// 1 if immediate, or `duration/interval` if periodic.
    /// Returns error if `duration` is not a multiple of `interval`.
    pub fn portion_count (&self) -> u128 {
        if self.interval == 0 {
            0
        } else {
            (self.total.u128() - self.head.u128()) / self.interval as u128
        }
    }
    /// Full `total` if immediate, or `(total-head)/portion_count` if periodic.
    /// Returns error if total can't be divided evenly in that number of portions.
    pub fn portion_size (&self) -> u128 {
        if self.interval == 0 {
            0
        } else {
            self.total.u128() / self.portion_count()
        }
    }
    define_errors!{
        err_broke (&self, ) -> (
            "{}: account would run out of money",
            &self.name)
        err_unspent () -> ("")
    }
}
