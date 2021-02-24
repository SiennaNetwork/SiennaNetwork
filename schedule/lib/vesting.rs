//! # Vesting
//!
//! This module implements the logic that turns a nested configuration into
//! a flat list of `Portion`s, each one describing a transactions that needs
//! to be executed in the future.
//!
//! `AllocationSet` implements the functions:
//! * `vest_immediate`
//! * `vest_cliff`
//! * `vest_regular`
//! * `vest_remainder`
//!
//! These output `Portion`s, which are subsequently "bubbled up" all the way
//! to the contract which will execute them, by the `all()` methods of these
//! structs:
//!
//! * `Schedule`
//! * `Pool`
//! * `Channel`
//! * `Periodic`
//!
//! All but `Periodic` implement their `all()` method via the `Vesting` trait.
//!
//! Just like `validate`, this module contains more than `rustdoc` renders.
//! Aside from the `Vesting` trait, more info about the aforementioned methods
//! can be found in the docs for the corresponding struct.

use crate::*;

pub trait Vesting {
    /// Get amount unlocked for address `a` at time `t`.
    fn claimable_by_at (&self, a: &HumanAddr, t: Seconds) -> UsuallyPortions {
        let mut all = self.all()?;
        all.retain(|Portion{address,vested,..}|address==a&&*vested<=t);
        Ok(all)
    }
    fn all (&self) -> UsuallyPortions;
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
        for channel in self.channels.iter() {
            portions.append(&mut channel.all()?);
        }
        Ok(portions)
    }
}

impl Vesting for Channel {
    /// Get list of all portions that this channel will ever make available,
    /// given its history of configurations.
    fn all (&self) -> UsuallyPortions {
        // assume battle formation
        let Channel { name, amount
                    , start_at, cliff, cliff_allocations
                    , interval, regular_allocations
                    , duration, remainder_allocations } = self;

        // let's go
        let mut t_cursor = self.start_at;
        let mut all_portions = vec![];
        let mut remaining = (*amount).u128();

        // 1. vest the cliff.
        let reason = format!("{}: cliff", name);
        // add portions from cliff allocations
        for allocation in cliff_allocations.iter() {
            if allocation.amount.u128() > remaining {
                return Self::err_broke();
            }
            all_portions.push(allocation.to_portion(t_cursor, &reason));
            remaining -= allocation.amount.u128();
        }
        if remaining == 0u128 || *interval == 0 || *duration == 0 {
            return Ok(all_portions)
        }

        // 2. vest regular portions
        loop {
            // move time forward
            t_cursor += interval;
            // duration is optional but if we cross it then it ends here
            if *duration > 0u64 && t_cursor > self.start_at+self.duration {
                break
            }
            // add portions from regular allocations
            let reason = format!("{}: regular", name);
            for allocation in regular_allocations.iter() {
                if allocation.amount.u128() > remaining {
                    return Self::err_broke();
                }
                all_portions.push(allocation.to_portion(t_cursor, &reason));
                remaining -= allocation.amount.u128();
            }
        }

        // 3. vest remainders
        if remaining > 0 {
            // add portions from remainder allocations
            let reason = format!("{}: remainder", name);
            for allocation in remainder_allocations.iter() {
                if allocation.amount.u128() > remaining {
                    return Self::err_broke();
                }
                all_portions.push(allocation.to_portion(t_cursor, &reason));
                remaining -= allocation.amount.u128();
            }
        }

        Ok(all_portions)

    }
}
impl Channel {
    /// 1 if immediate, or `duration/interval` if periodic.
    /// Returns error if `duration` is not a multiple of `interval`.
    pub fn portion_count (&self) -> u128 {
        if self.interval == 0 {
            0 
        } else {
            (self.amount.u128() - self.cliff.u128()) / self.interval as u128
        }
    }
    /// Full `amount` if immediate, or `(amount-cliff)/portion_count` if periodic.
    /// Returns error if amount can't be divided evenly in that number of portions.
    pub fn portion_size (&self) -> u128 {
        if self.interval == 0 {
            0 
        } else {
            self.amount.u128() / self.portion_count()
        }
    }
    define_errors!{
        err_broke () -> ("")
        err_unspent () -> ("")
    }
}
