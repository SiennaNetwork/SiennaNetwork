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


trait Vesting {
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
    fn all (&self) -> UsuallyPortions {
        match &self.periodic {
            Some(periodic) => Ok(periodic.all(&self)?.0),
            None => {
                if self.allocations.len() < 1 {
                    return Self::err_no_allocations(&self.name);
                }
                let latest_allocations =
                    self.allocations.get(self.allocations.len()-1).unwrap();
                latest_allocations.vest_immediate(&self, self.amount.u128())
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
    pub fn all (&self, ch: &Channel) -> UsuallyPortionsWithTotal {
        // assume battle formation
        let Channel { amount:      total_amount
                    , allocations: all_allocations
                    , .. } = ch;
        let Periodic { start_at
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
        let mut all_portions = vec![];
        let mut remaining_amount = total_amount.u128();
        // "scroll" allocations to start of vesting
        // FIXME an assumption that we're not currently validating
        //       is that the allocations are always stored sorted
        let mut active_allocations = all_allocations.get(0).unwrap();
        // from now on we'll only be moving forward in time with allocations
        // so let's make the collection of `AllocationSet`s a mutable iterator
        let mut all_allocations = all_allocations.iter().peekable();
        // if there are more recent allocations before the cliff,
        // switch to those
        loop {
            match all_allocations.peek() {
                None => break,
                Some(AllocationSet{t,..}) => if t > start_at { break }
            }
            match all_allocations.next() {
                None => break,
                Some(a) => if a.t > active_allocations.t { active_allocations = a }
            }
        }
        // now the active `AllocationSet` is the last one given
        // before `start_at` and the ones before it don't matter.
        // let's use its `cliff` allocations to vest the cliff,
        let mut t_cursor = *start_at;
        match active_allocations.vest_cliff(&ch, &self)? {
            None => { /* no cliff, start with 1st regular portion */ },
            Some((mut portions, vested)) => {
                all_portions.append(&mut portions);
                remaining_amount -= vested;
                t_cursor += interval; // tempus fugit
            }
        };
        // there we go. now, let's repeat this - `self.portion_count` times
        // (assumedly), but it can actually go for longer than that if the
        // channel runs on partial allocations. so just repeat this loop until
        // the channel's total amount is vested
        loop {
            // before each regular vesting, new reallocations may have happened
            // so let's fast-forward the iterator to the current allocation set
            loop {
                match all_allocations.peek() {
                    None => break,
                    Some(AllocationSet{t,..}) => if *t > t_cursor { break }
                }
                match all_allocations.next() {
                    None => break,
                    Some(a) => if a.t > active_allocations.t { active_allocations = a }
                }
            }
            if remaining_amount > AllocationSet::sum(&active_allocations.regular) {
                // if there's enough left for for a regular portion,
                // vest that and advance the time
                let (mut portions, vested) = active_allocations.vest_regular(
                    &ch, t_cursor, remaining_amount)?;
                all_portions.append(&mut portions);
                remaining_amount -= vested;
                t_cursor += interval;
            } else if remaining_amount > AllocationSet::sum(&active_allocations.remainder) {
                // if there's not enough left for a regular or remainder portion
                return Self::err_too_much_remaining()
            } else if remaining_amount == AllocationSet::sum(&active_allocations.remainder) {
                // if there's exactly enough left for a remainder portion,
                // vest that and stop. (this is meant to include the
                // `remaining_amount == 0 && active_allocations.remainder = []` case)
                let (mut portions, vested) = active_allocations.vest_remainder(
                    &ch, &self, remaining_amount)?;
                all_portions.append(&mut portions);
                remaining_amount -= vested;
                break
            } else if remaining_amount < AllocationSet::sum(&active_allocations.remainder) {
                // if there's too little left for a remainder portion
                return Self::err_too_little_remaining()
            }
        }
        Ok((all_portions, remaining_amount))
    }

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
        err_too_much_remaining () -> ("")
        err_too_little_remaining () -> ("")
    }
}

impl AllocationSet {
    /// For things that don't work with a periodic channel, like immediate vesting
    /// MAYBE the type system can do something about this?
    /// GOTCHA `serde-wasm` doesn't support non-C-like structs though
    fn assert_not_periodic (c: &Channel) -> UsuallyOk {
        match c.periodic {
            Some(_) => Self::err_immediate_periodic(),
            None    => Ok(())
        }
    }
    fn assert_no_periodic_allocations (&self) -> UsuallyOk {
        if self.cliff.len() > 0 || self.remainder.len() > 0 {
            // these only make sense in periodic vesting scenario
            // so their presence for an immediate-release channel is an error
            Self::err_immediate_periodic_allocations()
        } else {
            Ok(())
        }
    }
    /// For channels that vest everything immediately
    // TODO governance formatted as diff between (config.json, tx.json) pairs
    pub fn vest_immediate (&self, c: &Channel, total: u128) -> UsuallyPortions {
        Self::assert_not_periodic(&c)?;
        self.assert_no_periodic_allocations();
        let total = Self::sum(&self.regular);
        if total < c.amount.u128() {
            Self::err_immediate_partial_allocation()
        } else if total > c.amount.u128() {
            Self::err_profligate()
        } else {
            let reason = format!("{}: immediate", &c.name);
            Ok(Self::portions(&self.regular, 0, &reason))
        }
    }

    /// For things that don't work with an immediate channel, like periodic vesting
    fn assert_not_immediate (c: &Channel) -> UsuallyOk {
        match c.periodic {
            None    => Self::err_periodic_immediate(),
            Some(_) => Ok(())
        }
    }
    fn assert_total_not_exceeded (
        &self,
        actual_total: u128,
        expected_total: u128,
    ) -> UsuallyOk {
        if actual_total > expected_total {
            return Self::err_profligate()
        }
        Ok(())
    }
    pub fn vest_cliff (
        &self,
        c: &Channel,
        p: &Periodic
    ) -> PerhapsPortionsWithTotal {
        Self::assert_not_immediate(&c)?;
        let total = Self::sum(&self.cliff);
        self.assert_total_not_exceeded(p.cliff.u128(), total)?;
        if total == p.cliff.u128() && total == 0 {
            Ok(None)
        } else {
            let reason = format!("{}: cliff", &c.name);
            Ok(Some((Self::portions(&self.cliff, p.start_at, &reason), total)))
        }
    }
    pub fn vest_regular (
        &self,
        c: &Channel,
        t: Seconds,
        max: u128, // current maximum portion size according to caller
    ) -> UsuallyPortionsWithTotal {
        Self::assert_not_immediate(&c)?;
        let total = Self::sum(&self.regular);
        self.assert_total_not_exceeded(max, total)?;
        let reason = format!("{}: vesting", &c.name);
        Ok((Self::portions(&self.regular, t, &reason), total))
    }
    pub fn vest_remainder (
        &self,
        c: &Channel,
        p: &Periodic,
        remainder: u128, // current maximum portion size according to caller
    ) -> UsuallyPortionsWithTotal {
        Self::assert_not_periodic(&c)?;
        let total = Self::sum(&self.remainder);
        self.assert_total_not_exceeded(remainder, total)?;
        if total < remainder {
            Self::err_remainder_not_fully_allocated()
        } else {
            let t = p.start_at + p.duration;
            let reason = format!("{}: cliff", &c.name);
            Ok((Self::portions(&self.remainder, t, &reason), total))
        }
    }
    define_errors!{
        err_profligate () -> (
            "this would give too much")
        err_immediate_periodic () -> (
            "immediate vesting tried on periodic channel")
        err_immediate_periodic_allocations () -> (
            "immediate vesting tried alongside cliff/remainder allocation subsets")
        err_immediate_partial_allocation () -> (
            "")
        err_periodic_immediate () -> (
            "periodic vesting tried on immediate channel")
        err_remainder_not_fully_allocated () -> (
            "") }
}
