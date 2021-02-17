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

use crate::*;

type MaybePortions = StdResult<Vec<Portion>>;

trait Vesting {
    /// Get amount unlocked for address `a` at time `t`.
    fn claimable_by_at (&self, a: &HumanAddr, t: Seconds) -> MaybePortions {
        let all = self.all()?;
        all.retain(|Portion{address,vested,..}|address==a&&*vested<=t);
        Ok(all)
    }
    fn all (&self) -> MaybePortions;
}

impl Vesting for Schedule {
    /// Get list of all portions that will be unlocked by this schedule
    fn all (&self) -> MaybePortions {
        let mut portions = vec![];
        for pool in self.pools.iter() {
            portions.append(&mut pool.all()?);
        }
        Ok(portions)
    }
}

impl Vesting for Pool {
    /// Get list of all portions that will be unlocked by this pool
    fn all (&self) -> MaybePortions {
        let mut portions = vec![];
        for pool in self.channels.iter() {
            portions.append(&mut pool.all()?);
        }
        Ok(portions)
    }
}

impl Vesting for Channel {
    /// Get list of all portions that will be unlocked by this channel
    /// Immediate vestings only need the latest set of allocations to work,
    /// but periodic vestings need to iterate over the full history of allocations
    /// in order to generate portions after reallocation without rewriting history.
    /// **WARNING**: it is assumed that there is always at least 1 set of allocations,
    ///              that there is no more than 1 set of allocations per timestamp,
    ///              and that allocations are stored sorted
    fn all (&self) -> MaybePortions {
        match &self.periodic {
            Some(periodic) => periodic.all(&self),
            None => {
                let mut portions = vec![];
                if self.allocations.len() < 1 {
                    return Self::err_no_allocations(&self.name);
                }
                let latest_allocations =
                    self.allocations.get(self.allocations.len()-1).unwrap();
                Ok(latest_allocations.vest_immediate(&self.name, self.amount.u128())?)
            }
        }
    }
}
impl Channel {
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
    /// Generate all portions for channel `ch`
    /// given `self` is the portioning config
    pub fn all (&self, ch: &Channel) -> MaybePortions {
        // assume battle formation
        let Channel { name
                    , amount:      total_amount
                    , allocations: all_allocations
                    , .. } = ch;
        let Periodic { start_at
                     , cliff
                     , interval
                     , .. } = self;
        // late-validate some assumptions
        // MAYBE pre-validation should somehow be interconnected
        //       with the place where an invalid value breaks an assumption?
        // ALTHOUGH this does not matter much if `all()` is called off-chain
        //          and its errors constitute just another layer of validation,
        //          as this refactor intends.
        if all_allocations.len() < 1 { return Channel::err_no_allocations(&ch.name); }
        if *interval == 0 { return Self::err_zero_interval(&ch.name) }
        // let's go
        let mut active_allocations = all_allocations.get(0).unwrap();
        let mut all_portions = vec![];
        let mut remaining = total_amount.u128();
        // "scroll" allocations to start of vesting
        // FIXME an assumption that we're not currently validating
        //       is that the allocations are always stored sorted
        let all_allocations = all_allocations.iter();
        // let's keep the iterator around
        for a in all_allocations {
            let AllocationSet{t,..} = a;
            if t > start_at {
                // GOTCHA this may cause allocations to go missing
                // if there's only one of them within an interval
                // because the iterator loops once too many
                // is there a rewind/peek?
                break
            }
            if *t > active_allocations.t {
                active_allocations = a
            }
        }
        // now the active `AllocationSet` is the last one given
        // before `start_at` and the ones before it don't matter.
        // let's use its `cliff` allocations to vest the cliff,
        let mut t_cursor = start_at;
        match active_allocations.vest_cliff(&ch, &self)? {
            None => { /* no cliff, start with 1st regular portion */ },
            Some((vested, portions)) => {
                all_portions.append(&mut portions);
                remaining -= vested;
                *t_cursor += interval; // tempus fugit
            }
        };
        // there we go. now, let's repeat this - `self.portion_count` times
        // (assumedly), but it can actually go for longer than that if the
        // channel runs on partial allocations. so just repeat this loop until
        // the channel's total amount is vested
        loop {
            // before each regular vesting, new reallocations may have happened
            // so let's fast-forward the iterator to the current allocation set
            for a in all_allocations {
                let AllocationSet{t,..} = a;
                if t > t_cursor {
                    break
                }
                if *t > active_allocations.t {
                    active_allocations = a
                }
            }
            // now let's see if we have enough remaining
            // for another regular portion
            if remaining > active_allocations.regular_portion_size() {
                match active_allocations.vest_regular(&ch, *t_cursor, remaining)? {
                    None => { return Self::err_empty_regular_vesting() },
                    Some((vested, portions)) => {
                        all_portions.append(&mut portions);
                        remaining -= vested;
                        *t_cursor += interval;
                    }
                }
            } else if remaining > active_allocations.final_portion_size() {
                Self::err_remaining_after_final()
            } else if remaining == active_allocations.final_portion_size() {
                match active_allocations.vest_final(&ch, *t_cursor, remaining)? {
                    None => { /*???*/ },
                    Some((vested, portions)) => {
                        all_portions.append(&mut portions);
                        remaining -= vested;
                        break
                    }
                }
            } else if remaining < active_allocations.final_portion_size() {
                Self::err_too_little_remaining()
            }
    }

    //fn add_regular () -> MaybePortions {
        //let mut portions = vec![];
        //loop {
            //if t_cursor > t || t_cursor >= t_end { break }

            //// Determine the group of allocations that is current
            //// at time `t_cursor`. (It is assumed that groups of
            //// allocations are sorted).
            //for (t_alloc, allocations) in ch.allocations.iter() {
                //if *t_alloc > t_cursor { break }
                //current_allocations = allocations;
            //}
            
            //// From the current group of allocations, determine
            //// the actual claimable amount, and add the
            //// corresponding portion.
            //for Allocation { addr, amount } in current_allocations.iter() {
                //if addr == a {
                    //let amount = (*amount).u128();
                    //let reason = format!("{}: vesting", &ch.name);
                    //portions.push(portion(amount, a, t_cursor, &reason));
                    //total_received += amount;
                //}
            //}

            //// Advance the time.
            //t_cursor += interval
        //}
        //Ok(portions)
    //}
    //fn add_remainder () -> MaybePortions {
        //let mut portions = vec![];
        //let remainder = ch.amount.u128() - total_received;
        //if remainder > 0 {
            //// The last group of allocations must contain exactly 1 user
            //// in order to avoid splitting the remainder.
            //if current_allocations.len() == 1 {
                //let Allocation{addr,..} = current_allocations.get(0).unwrap();
                //if addr == a {
                    //let reason = format!("{}: remainder", &ch.name);
                    //portions.push(portion(remainder, a, t_cursor, &reason));
                //}
                //// If that is not the case, the admin should be able to
                //// call `Reallocate` and determine a single adress to
                //// receive the remainder.
            //}
        //}
        //Ok(portions)
    //}
    ///// Critical section: generates `Portion`s according to the vesting ladder config.
    //pub fn claimable_by_at (&self, ch: &Channel, a: &HumanAddr, t: Seconds) -> MaybePortions {

        //let Periodic{start_at,cliff,interval,..} = self;

         //Interval can't be 0 (prevent infinite loop below)
        //if *interval == 0 { return Self::err_zero_interval(&ch.name) }

         //Nothing can be claimed before the start
        //if t < *start_at { return Ok(vec![]) }

         //Now comes the part where we iterate over the time range
         //`start_at..min(t, start_at+duration)` in steps of `interval`,
         //and add vestings in accordance with the allocations that are
         //current for the particular moment in time.
        //let mut portions = vec![];
        //let mut total_received: u128 = 0;
        //let mut t_cursor = *start_at;
        //let mut n_portions = self.portion_count(&ch.name)?;

         //Make sure allocations exist.
        //if ch.allocations.len() < 1 { return Channel::err_no_allocations(&ch.name); }

         //Get first group of allocations.
        //let (t_alloc, current_allocations) = ch.allocations.get(0).unwrap();
        //let mut current_allocations = current_allocations;
        //if *t_alloc > t_cursor { return Self::err_time_travel(&ch.name) }

         //If the `channel` has a `cliff`, and the first group of
         //`allocations` contains the claimant `a`, then that
         //user must receive the cliff amount.
        //let cliff = cliff.u128();
        //if cliff > 0 {
             //The first group of allocations must contain exactly
             //1 user to avoid splitting the cliff.
            //if current_allocations.len() != 1 {
                //return Self::err_periodic_cliff_multiple(&ch.name)
            //}
            //for Allocation {addr, ..} in current_allocations.iter() {
                //if addr == a {
                     //If the above is true, make the cliff amount
                     //the first portion, and advance the time.
                    //let reason = format!("{}: cliff", &ch.name);
                    //portions.push(portion(cliff, a, *start_at, &reason));
                    //t_cursor += interval;
                    //n_portions += 1;
                    //total_received += cliff;
                    //break
                //}
            //}
        //}

         //After the first cliff, add a new portion for every `interval` seconds until `t`
         //unless `t_cursor` is past the current time `t` or the end time `t_end`.
        //let t_end = start_at + n_portions * interval;
        //loop {
            //if t_cursor > t || t_cursor >= t_end { break }

             //Determine the group of allocations that is current
             //at time `t_cursor`. (It is assumed that groups of
             //allocations are sorted).
            //for (t_alloc, allocations) in ch.allocations.iter() {
                //if *t_alloc > t_cursor { break }
                //current_allocations = allocations;
            //}
            
             //From the current group of allocations, determine
             //the actual claimable amount, and add the
             //corresponding portion.
            //for Allocation { addr, amount } in current_allocations.iter() {
                //if addr == a {
                    //let amount = (*amount).u128();
                    //let reason = format!("{}: vesting", &ch.name);
                    //portions.push(portion(amount, a, t_cursor, &reason));
                    //total_received += amount;
                //}
            //}

             //Advance the time.
            //t_cursor += interval
        //}
         //MAYBE cap this by sum, not by time?

         //If we're at/past the end, add give the remainder.
         //How does this work with multiple allocations though?
        //if t_cursor >= t_end {
            //let remainder = ch.amount.u128() - total_received;
            //if remainder > 0 {
                 //The last group of allocations must contain exactly 1 user
                 //in order to avoid splitting the remainder.
                //if current_allocations.len() == 1 {
                    //let Allocation{addr,..} = current_allocations.get(0).unwrap();
                    //if addr == a {
                        //let reason = format!("{}: remainder", &ch.name);
                        //portions.push(portion(remainder, a, t_cursor, &reason));
                    //}
                     //If that is not the case, the admin should be able to
                     //call `Reallocate` and determine a single adress to
                     //receive the remainder.
                //}
            //}
        //}

        //Ok(portions)
    //}
    /// GOTCHA: Partial reallocations should extend the duration of the vesting,
    ///         increasing `portion_count` accordingly.
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
    define_errors!{
        /* err_zero_interval defined in validation.rs */
        /* err_periodic_cliff_multiple defined in validation.rs */
        err_duration_remainder (name: &str, duration: Seconds, interval: Seconds) ->
            ("channel {}: duration ({}s) does not divide evenly in intervals of {}s",
                name, duration, interval)
        err_interval_gt_duration (name: &str, duration: Seconds, interval: Seconds) ->
            ("channel {}: duration ({}) must be >= interval ({})",
                name, duration, interval)
        err_profligate (name: &str, tried_to_vest: u128, total_amount: Uint128) ->
            ("channel {}: tried to vest {} which is more than the total of {}",
                name, tried_to_vest, total_amount.u128())
        err_cliff_only (name: &str) ->
            ("channel {}: periodic vesting must contain at least 1 non-cliff portion",
                name)
        err_time_travel (name: &str) ->
            ("channel {}: time of first allocations is after current time",
                name)
        err_empty_regular_vesting () -> ("")
    }
}
impl AllocationSet {
    pub fn vest_immediate (
        &self,
        name: &str,
        total: u128
    ) -> MaybePortions {
        // these only make sense in periodic vesting scenario
        // so their presence for an immediate-release channel is an error
        self.assert_no_cliff(name)?;
        self.assert_no_remainder(name)?;
        let mut sum_of_allocations = 0u128;
        let mut portions = vec![];
        for Allocation{amount, addr} in self.regular.iter() {
            portions.push(Portion {
                amount:  *amount,
                address: *addr,
                vested:  0,
                reason:  format!("{}: cliff", name)
            });
            sum_of_allocations += amount.u128();
            if sum_of_allocations > total {
                return Self::err_profligate()
            }
        }
        Ok(portions) 
    }
    fn assert_no_cliff (&self, name: &str) -> StdResult<()> {
        if self.cliff.len() > 0 {
            Self::err_cliff_allocations(name)
        } else {
            Ok(())
        }
    }
    fn assert_no_remainder (&self, name: &str) -> StdResult<()> {
        if self.remainder.len() > 0 {
            Self::err_remainder_allocations(&name)
        } else {
            Ok(())
        }
    }
    pub fn vest_cliff (
        &self,
        c: &Channel,
        p: &Periodic
    ) -> StdResult<Option<(u128, Vec<Portion>)>> {
        let total = self.assert_total_not_exceeded(p.cliff.u128(), &self.cliff)?;
        if total == p.cliff.u128() && total == 0 {
            Ok(None)
        } else {
            Ok((total, self.cliff.iter().map(|Allocation{amount,addr}| Portion {
                amount:  *amount,
                address: *addr,
                vested:  p.start_at,
                reason:  format!("{}: cliff", &c.name)
            }).collect()))
        }
    }
    pub fn vest_regular (
        &self,
        c: &Channel,
        t: Seconds,
        max: u128, // current maximum portion size according to caller
    ) -> StdResult<Option<(u128, Vec<Portion>)>> {
        self.assert_total_not_exceeded(max, &self.regular)?;
        Ok(Some((total, self.regular.iter().map(|Allocation{amount,addr}| Portion {
            amount:  *amount,
            address: *addr,
            vested:  t,
            reason:  format!("{}: cliff", &c.name)
        }).collect())))
    }
    pub fn vest_remainder (
        &self,
        c: &Channel,
        p: &Periodic,
        remainder: u128, // current maximum portion size according to caller
    ) -> StdResult<Option<(u128, Vec<Portion>)>> {
        let total = self.assert_total_not_exceeded(remainder, &self.remainder)?;
        if total < remainder {
            Self::err_remainder_not_fully_allocated(&c.name)
        } else {
            Ok(Some((total, self.regular.iter().map(|Allocation{amount,addr}| Portion {
                amount:  *amount,
                address: *addr,
                vested:  p.start_at,
                reason:  format!("{}: cliff", &c.name)
            }).collect())))
        }
    }
    fn assert_total_not_exceeded (
        &self,
        expected_total: u128,
        allocations: &Vec<Allocation>
    ) -> StdResult<u128> {
        let mut sum_of_allocations = 0u128;
        for Allocation{amount,..} in allocations.iter() {
            sum_of_allocations += amount.u128();
            if sum_of_allocations > expected_total {
                return Self::err_profligate()
            }
        }
        Ok(sum_of_allocations)
    }
    define_errors!{
        err_profligate () -> ("")
        err_cliff_allocations (name: &str) -> ("")
        err_remainder_allocations (name: &str) -> ("") }
}
