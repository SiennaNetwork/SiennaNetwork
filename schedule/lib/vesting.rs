/// Here's what was casting the shadow of a off-by-one error all along, and
/// causing difficult-to-verbalize confusion around some aspects of the spec.
///
/// Let's assume there's an interval of `N` between vestings:
/// * If `cliff == 0` then all the vestings are equal: `amount / n_portions`.
/// * If `cliff > 0` then the first vesting is equal to `cliff`, and since
///   there's an interval of `N` after it, every other vesting is equal to
///   `(amount - cliff) / (n_portions / 1)`.
///
/// So, `if cliff > 0 { n_portions -= 1 }`.
///
/// How to fix that? Adding cliffs everywhere to get appropriate post-cliff
/// amounts is one option. However:
/// * The cliff sizes are reassuringly arbitrary. I like that they are
///   nice round numbers.
/// * Making them interdependent with the rest of the calculations sounds
///   painful, especially considering that the contract works in fixed
///   precision and it is not obvious at all how to pick cliffs that both:
///   * turn the remaining portions into nice round numbers, and
///   * are nice round numbers themselves.
/// * Receiving the cliff and the first vesting at the same time, or receiving
///   a cliff that's smaller than the regular vesting portion before the regular
///   vesting commences, can be confusing for claimants; and picking an
///   appropriate cliff size for every account should be up to the contract
///   owner and not the library implementor.
///
/// Therefore, remainders are used in the following way: while cliffs remain
/// arbitrary, the last vesting of every channel contains the remainder of
/// the division `(amount - cliff) / (n_portions / 1)`.

use cosmwasm_std::StdResult;
use crate::*;

impl Schedule {
    /// Get amount unlocked for address `a` at time `t`.
    pub fn claimable_by_at (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for pool in self.pools.iter() {
            portions.append(&mut pool.claimable_by_at(a, t)?);
        }
        Ok(portions)
    }
}

impl Pool {
    pub fn claimable_by_at (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for channel in self.channels.iter() {
            portions.append(&mut channel.claimable_by_at(a, t)?);
        }
        Ok(portions)
    }
}

impl Channel {
    /// Return list of portions that have become claimable for address `a` by time `t`.
    /// Immediate vestings only need the latest set of allocations to work,
    /// but periodic vestings need to iterate over the full history of allocations
    /// in order to generate portions after reallocation without rewriting history.
    /// **WARNING**: it is assumed that there is always at least 1 set of allocations,
    ///              that there is no more than 1 set of allocations per timestamp,
    ///              and that allocations are stored sorted
    pub fn claimable_by_at (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        match &self.periodic {
            Some(periodic) => periodic.claimable_by_at(&self, a, t),
            None => {
                let mut portions = vec![];
                match self.allocations.get(self.allocations.len()-1) {
                    None => return Self::err_no_allocations(&self.name),
                    Some((_, latest_allocations)) => {
                        for Allocation { addr, amount } in latest_allocations.iter() {
                            if addr == a {
                                let reason = format!("{}: immediate", &self.name);
                                portions.push(portion((*amount).u128(), a, 0, &reason));
                            }
                        }
                    }
                }
                Ok(portions)
            },
        }
    }
    /// 1 if immediate, or `duration/interval` if periodic.
    /// Returns error if `duration` is not a multiple of `interval`.
    pub fn portion_count (&self) -> StdResult<u64> {
        match &self.periodic {
            None           => Ok(1u64),
            Some(periodic) => periodic.portion_count(&self.name)
        }
    }
    /// Full `amount` if immediate, or `(amount-cliff)/portion_count` if periodic.
    /// Returns error if amount can't be divided evenly in that number of portions.
    pub fn portion_size (&self) -> StdResult<u128> {
        match &self.periodic {
            None           => Ok(self.amount.u128()),
            Some(periodic) => periodic.portion_size(&self.name, self.amount.u128())
        }
    }
    define_errors!{
        err_no_allocations (name: &str) ->
            ("channel {}: no allocations",
                name)}
}

impl Periodic {
    pub fn portion_count (&self, name: &str) -> StdResult<u64> {
        let Periodic{cliff,duration,interval,..}=self;
        if duration % interval > 0 {
            return Self::err_duration_remainder(name, self.duration, self.interval);
        }
        if duration < interval {
            return Self::err_interval_gt_duration(name, self.duration, self.interval);
        }
        let n_portions = duration / interval;
        if *cliff > Uint128::zero() {
            if n_portions < 2 {
                return Self::err_cliff_only(name)
            }
            return Ok(n_portions - 1u64)
        }
        Ok(n_portions)
    }
    pub fn portion_size (&self, name: &str, amount: u128) -> StdResult<u128> {
        let n_portions = self.portion_count(name)? as u128;
        let mut amount = amount;
        amount -= self.cliff.u128();
        Ok(amount / n_portions)
    }
    /// Critical section: generates `Portion`s according to the vesting ladder config.
    pub fn claimable_by_at (&self, ch: &Channel, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {

        let Periodic{start_at,cliff,interval,..} = self;

        // Interval can't be 0 (prevent infinite loop below)
        if *interval == 0 { return Self::err_zero_interval(&ch.name) }

        // Nothing can be claimed before the start
        if t < *start_at { return Ok(vec![]) }

        // Now comes the part where we iterate over the time range
        // `start_at..min(t, start_at+duration)` in steps of `interval`,
        // and add vestings in accordance with the allocations that are
        // current for the particular moment in time.
        let mut portions = vec![];
        let mut total_received: u128 = 0;
        let mut t_cursor = *start_at;
        let mut n_portions = self.portion_count(&ch.name)?;

        // Make sure allocations exist.
        if ch.allocations.len() < 1 { return Channel::err_no_allocations(&ch.name); }

        // Get first group of allocations.
        let (t_alloc, current_allocations) = ch.allocations.get(0).unwrap();
        let mut current_allocations = current_allocations;
        if *t_alloc > t_cursor { return Self::err_time_travel(&ch.name) }

        // If the `channel` has a `cliff`, and the first group of
        // `allocations` contains the claimant `a`, then that
        // user must receive the cliff amount.
        let cliff = cliff.u128();
        if cliff > 0 {
            for Allocation {addr, ..} in current_allocations.iter() {
                if addr == a {
                    // The first group of allocations must contain exactly
                    // 1 user to avoid splitting the cliff.
                    if current_allocations.len() != 1 {
                        return Self::err_periodic_cliff_multiple(&ch.name)
                    }
                    // If the above is true, make the cliff amount
                    // the first portion, and advance the time.
                    let reason = format!("{}: cliff", &ch.name);
                    portions.push(portion(cliff, a, *start_at, &reason));
                    t_cursor += interval;
                    n_portions += 1;
                    total_received += cliff;
                    break
                }
            }
        }

        // After the first cliff, add a new portion for every `interval` seconds until `t`
        // unless `t_cursor` is past the current time `t` or the end time `t_end`.
        let t_end = start_at + n_portions * interval;
        loop {
            if t_cursor > t || t_cursor >= t_end { break }

            // Determine the group of allocations that is current
            // at time `t_cursor`. (It is assumed that groups of
            // allocations are sorted).
            for (t_alloc, allocations) in ch.allocations.iter() {
                if *t_alloc > t_cursor { break }
                current_allocations = allocations;
            }
            
            // From the current group of allocations, determine
            // the actual claimable amount, and add the
            // corresponding portion.
            for Allocation { addr, amount } in current_allocations.iter() {
                if addr == a {
                    let amount = (*amount).u128();
                    let reason = format!("{}: vesting", &ch.name);
                    portions.push(portion(amount, a, t_cursor, &reason));
                    total_received += amount;
                }
            }

            // Advance the time.
            t_cursor += interval
        }
        // MAYBE cap this by sum, not by time?

        // If we're at/past the end, add give the remainder.
        // How does this work with multiple allocations though?
        if t_cursor >= t_end {
            let remainder = ch.amount.u128() - total_received;
            if remainder > 0 {
                // The last group of allocations must contain exactly 1 user
                // in order to avoid splitting the remainder.
                if current_allocations.len() == 1 {
                    let Allocation{addr,..} = current_allocations.get(0).unwrap();
                    if addr == a {
                        let reason = format!("{}: remainder", &ch.name);
                        portions.push(portion(remainder, a, t_cursor, &reason));
                    }
                    // If that is not the case, the admin should be able to
                    // call `Reallocate` and determine a single adress to
                    // receive the remainder.
                }
            }
        }

        Ok(portions)
    }
    define_errors!{
        /* err_zero_interval defined in validation.rs */
        /* err_periodic_cliff_multiple defined in validation.rs */
        err_duration_remainder (name: &str, duration: Seconds, interval: Seconds) ->
            ("channel {}: duration ({}s) does not divide evenly in intervals of {}s",
                name, duration, interval)
        err_interval_gt_duration (name: &str, duration: Seconds, interval: Seconds) ->
            ("channel {}: duration ({}) must be >= interval ({})",
                name, duration, interval)
        err_cliff_only (name: &str) ->
            ("channel {}: periodic vesting must contain at least 1 non-cliff portion",
                name)
        err_time_travel (name: &str) ->
            ("channel {}: time of first allocations is after current time",
                name)}
}
