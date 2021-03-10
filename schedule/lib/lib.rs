/// # SIENNA Schedule v1.0
///
/// ## The `Schedule` object
///
/// The root object. Has a `total`, contains `Pools` adding up to that total.
///
/// ```
/// # use sienna_schedule::units::*;
/// #[macro_use] extern crate sienna_schedule; use sienna_schedule::constructors::*;
/// # fn main () {
/// valid!(schedule(0, vec![]));
/// claim!(schedule(0, vec![]), HumanAddr::from(""), 0);
/// invalid!(schedule(100, vec![]), "schedule: pools add up to 0, expected 100");
/// invalid!(schedule(100, vec![pool("P1", 50, vec![])]), "pool P1: channels add up to 0, expected 50");
/// valid!(schedule(0, vec![pool("P1", 0, vec![]), pool("P2", 0, vec![])]));
/// # }
/// ```
///
/// ## The `Pool` object
/// Subdivision of `Schedule`, contains `Channel`s.
/// * If `partial` is true, makes room for adding channels at a later time, up to the total.
/// * Otherwise, requires `Channel`s to add up to the total from the start
///
/// ```
/// #[macro_use] extern crate sienna_schedule; use sienna_schedule::constructors::*;
/// # fn main () {
/// valid!(pool("", 0, vec![]), 0);
/// # }
/// ```
///
/// ## The `Channel` object
/// Subdivision of a `Pool`.
/// * If it contains `Periodic` scheduling, it releases periodic `Portion`s,
///   starting at `start_at` seconds since launch and then every `interval`
///   seconds over a specified `duration`, optionally with special
///   `cliff` and `remainder` vestings.
/// * Otherwise it releases the whole `amount` as one big `Portion` upon contract launch.
///
/// ## The `Allocation` object
/// Regular vesting of channels (but not cliff or remainder vestings) can
/// optionally be split into multiple `Portions` for multiple addresses.
///
/// ```
/// # use sienna_schedule::units::*;
/// #[macro_use] extern crate sienna_schedule; use sienna_schedule::constructors::*;
/// # fn main () {
/// let alice = HumanAddr::from("Alice");
/// invalid!(schedule(100, vec![
///     pool("P1", 50, vec![channel_immediate(20, &alice)]),
///     pool("P2", 50, vec![channel_immediate(30, &alice)])
/// ]),
///     "pool P1: channels add up to 20, expected 50");
/// invalid!(schedule(100, vec![
///     pool("P1", 50, vec![channel_immediate(50, &alice)]),
///     pool("P2", 50, vec![channel_immediate(30, &alice)])
/// ]),
///     "pool P2: channels add up to 30, expected 50");
/// invalid!(schedule(100, vec![
///     pool("", 50, vec![
///         channel_immediate(30, &alice),
///         channel_periodic_multi(20, &vec![allocation(10, &alice)
///                                         ,allocation(10, &alice)], 1, 0, 1, 0)
///     ])]),
///         "schedule: pools add up to 50, expected 100");
/// # }
/// ```
///
/// ## Notice to future generations
/// Here stood a long-winded and partially confused explanation
/// which was part of my initial analysis of the vesting logic.
///
/// A naive reading of the budget allocated in the project brief
/// could cause a one-by-off error if you don't count the cliff
/// as a separate portion, resulting in portion allocations that
/// fail to divide evenly.
///
/// Take heed that the cliff counts as a separate portion, and
/// subtract it from the total amount before determining
/// portion size as `(amount - cliff) / duration`
/// and portion count as `duration / interval`.

use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmwasm_std::StdResult;

pub mod macros;
pub mod units; pub use units::*;
pub mod constructors; pub use constructors::*;

/// Vesting schedule; contains `Pool`s that must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
}
impl Schedule {
    fn err_total (&self, total: u128) -> StdResult<()> {
        Error!(format!("schedule: pools add up to {}, expected {}",
            total, self.total))
    }
    /// Make sure that the schedule contains valid data.
    pub fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for pool in self.pools.iter() {
            match pool.validate() {
                Ok(_)  => { total += pool.total.u128() },
                Err(e) => return Err(e)
            }
        }
        if total != self.total.u128() { return self.err_total(total) }
        Ok(())
    }
    /// Get amount unlocked for address `a` at time `t`.
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for pool in self.pools.iter() {
            portions.append(&mut pool.claimable(a, t)?);
        }
        Ok(portions)
    }
}

/// Vesting pool; contains `Channel`s that must add up to `total`
/// if `partial == false`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    pub name:     String,
    pub total:    Uint128,
    pub partial:  bool,
    pub channels: Vec<Channel>,
}
impl Pool {
    fn err_total<T> (&self, total: u128) -> StdResult<T> {
        Error!(format!("pool {}: channels add up to {}, expected {}",
            &self.name, total, self.total))
    }
    fn err_too_big<T> (&self, amount: u128, unallocated: u128) -> StdResult<T> {
        Error!(format!("pool {}: tried to add channel with size {}, which is more than the remaining {} of this pool's total {}",
            &self.name, amount, unallocated, self.total.u128()))
    }
    pub fn validate (&self) -> StdResult<u128> {
        let mut total = 0u128;
        for channel in self.channels.iter() {
            match channel.validate() {
                Ok(_)  => { total += channel.amount.u128() },
                Err(e) => return Err(e)
            }
        }
        let invalid_total = if self.partial {
            total > self.total.u128()
        } else {
            total != self.total.u128()
        };
        if invalid_total { return self.err_total(total) }
        Ok(total)
    }
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for channel in self.channels.iter() {
            portions.append(&mut channel.claimable(a, t)?);
        }
        Ok(portions)
    }
    pub fn add_channel (&mut self, ch: Channel) -> StdResult<()> {
        ch.validate()?;
        let allocated = self.validate()?;
        let unallocated = self.total.u128() - allocated;
        if ch.amount.u128() > unallocated {
            return self.err_too_big(ch.amount.u128(), unallocated);
        }
        self.channels.push(ch);
        Ok(())
    }
}

/// Portions generator: can be immediate or `Periodic`; contains `Allocation`s (maybe partial).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    pub name:   String,
    pub amount: Uint128,

    /// Each portion can be split between multiple addresses.
    /// The full history of reallocations is stored here.
    pub allocations: Vec<(Seconds, Vec<Allocation>)>,

    /// `None` -> 1 vesting at launch, `Some(Periodic)` -> multiple vestings.
    ///
    /// This is an `Option` instead of `Channel` being an `Enum` because
    /// `serde_json_wasm` doesn't support non-C-style enums.
    ///
    /// Immediate vesting:  the recipient can claim the entire allocated amount
    /// once (after the contract has been launched).
    ///
    /// Periodic vesting: contract calculates the maximum amount that the user
    /// can claim at the given time.
    pub periodic: Option<Periodic>
}
impl Channel {
    fn err_total (&self, total: u128, portion: u128) -> StdResult<()> {
        Error!(format!("channel {}: allocations add up to {}, expected {}",
            &self.name, total, portion))
    }
    fn err_no_allocations<T> (&self) -> StdResult<T> {
        Error!(format!("channel {}: no allocations",
            &self.name))
    }
    fn err_realloc_time_travel (&self, t: Seconds, t_max: Seconds) -> StdResult<()> {
        Error!(format!("channel {}: can not reallocate in the past ({} < {})",
            &self.name, &t, t_max))
    }
    fn err_realloc_cliff (&self) -> StdResult<()> {
        Error!(format!("channel {}: reallocations for channels with cliffs are not supported",
            &self.name))
    }
    pub fn validate (&self) -> StdResult<()> {
        if let Some(ref periodic) = self.periodic {
            periodic.validate(&self)?;
        }
        for (_, allocations) in self.allocations.iter() {
            let mut total_portion = 0u128;
            for Allocation { amount, .. } in allocations.iter() {
                total_portion += amount.u128()
            }
            let portion_size = self.portion_size()?;
            if total_portion > portion_size {
                return self.err_total(total_portion, portion_size);
            }
        }
        Ok(())
    }
    /// Allocations can be changed on the fly without affecting past vestings.
    pub fn reallocate (&mut self, t: Seconds, allocations: Vec<Allocation>) -> StdResult<()> {
        if let Some(Periodic{cliff: Uint128(0),..}) = self.periodic {
            let latest_allocation = self.allocations.iter().fold(0, |x,y|Seconds::max(x,y.0));
            if t < latest_allocation { return self.err_realloc_time_travel(t, latest_allocation) }
            self.allocations.push((t, allocations));
            self.validate()
        } else {
            return self.err_realloc_cliff();
        }
    }
    /// Return list of portions that have become claimable for address `a` by time `t`.
    /// Immediate vestings only need the latest set of allocations to work,
    /// but periodic vestings need to iterate over the full history of allocations
    /// in order to generate portions after reallocation without rewriting history.
    /// **WARNING**: it is assumed that there is always at least 1 set of allocations,
    ///              that there is no more than 1 set of allocations per timestamp,
    ///              and that allocations are stored sorted
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        match &self.periodic {
            None           => self.claimable_immediate(a),
            Some(periodic) => periodic.claimable(&self, a, t),
        }
    }
    fn claimable_immediate (&self, a: &HumanAddr) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        match self.allocations.last() {
            None => return self.err_no_allocations(),
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
}

/// Configuration of periodic vesting ladder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Periodic {
    pub interval:  Seconds,
    pub start_at:  Seconds,
    pub duration:  Seconds,
    pub cliff:              Uint128,
    pub expected_portion:   Uint128,
    pub expected_remainder: Uint128
}
impl Periodic {
    fn err_zero_duration (&self, name: &str) -> StdResult<()> {
        Error!(format!("channel {}: periodic vesting's duration can't be 0",
            name))
    }
    fn err_zero_interval<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: periodic vesting's interval can't be 0",
            name))
    }
    fn err_cliff_gt_total (&self, name: &str, cliff: u128, amount: u128) -> StdResult<()> {
        Error!(format!( "channel {}: cliff {} can't be larger than total amount {}",
            name, cliff, amount))
    }
    fn err_periodic_cliff_multiple<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: cliffs not supported alongside split allocations",
            name))
    }
    fn err_periodic_remainder_multiple<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: remainders not supported alongside split allocations",
            name))
    }
    fn err_duration_remainder<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: duration ({}s) does not divide evenly in intervals of {}s",
            name, self.duration, self.interval))
    }
    fn err_interval_gt_duration<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: duration ({}) must be >= interval ({})",
            name, self.duration, self.interval))
    }
    fn err_cliff_only<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: periodic vesting must contain at least 1 non-cliff portion",
            name))
    }
    fn err_time_travel<T> (&self, name: &str) -> StdResult<T> {
        Error!(format!("channel {}: time of first allocations is after current time",
            name))
    }
    pub fn portion_count (&self, name: &str) -> StdResult<u64> {
        let Periodic{cliff,duration,interval,..}=self;
        if duration % interval > 0 { return self.err_duration_remainder(name); }
        if duration < interval { return self.err_interval_gt_duration(name); }
        let n_portions = duration / interval;
        if cliff.u128() > 0 {
            if n_portions < 2 { return self.err_cliff_only(name) }
            return Ok(n_portions - 1u64)
        }
        Ok(n_portions)
    }
    pub fn portion_size (&self, name: &str, amount: u128) -> StdResult<u128> {
        let n_portions = self.portion_count(name)? as u128;
        Ok((amount - self.cliff.u128()) / n_portions)
    }
    pub fn validate (&self, ch: &Channel) -> StdResult<()> {
        let Periodic{cliff,duration,interval,..} = self;
        if *duration < 1u64 { return self.err_zero_duration(&ch.name) }
        if *interval < 1u64 { return self.err_zero_interval(&ch.name) }
        if *cliff > ch.amount { return self.err_cliff_gt_total(&ch.name, cliff.u128(), ch.amount.u128()) }
        for (_, allocations) in ch.allocations.iter() {
            if allocations.len() > 1 && cliff.u128() > 0 { return self.err_periodic_cliff_multiple(&ch.name) }
        }
        self.portion_size(&ch.name, ch.amount.u128())?;
        Ok(())
    }
    /// Critical section: generates `Portion`s according to the vesting ladder config.
    pub fn claimable (&self, ch: &Channel, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {

        let &Periodic { start_at, cliff: Uint128(cliff), interval, .. } = self;

        // Interval can't be 0 (prevent infinite loop below)
        if interval == 0 { return self.err_zero_interval(&ch.name) }

        // Nothing can be claimed before the start
        if t < start_at { return Ok(vec![]) }

        // Now comes the part where we iterate over the time range
        // `start_at..min(t, start_at+duration)` in steps of `interval`,
        // and add vestings in accordance with the allocations that are
        // current for the particular moment in time.
        let mut portions       = vec![];
        let mut total_received = 0u128;
        let mut total_vested   = 0u128;
        let mut t_cursor       = start_at;
        let mut n_portions     = self.portion_count(&ch.name)?;

        // Make sure allocations exist.
        if ch.allocations.is_empty() { return ch.err_no_allocations(); }

        // Get first group of allocations.
        let (t_alloc, current_allocations) = ch.allocations.get(0).unwrap();
        let mut current_allocations = current_allocations;
        if *t_alloc > t_cursor { return self.err_time_travel(&ch.name) }

        // If the `channel` has a `cliff`, and the first group of
        // `allocations` contains the claimant `a`, then that
        // user must receive the cliff amount.
        if cliff > 0 {
            for Allocation {addr, ..} in current_allocations.iter() {
                if addr == a {
                    // The first group of allocations must contain exactly
                    // 1 user to avoid splitting the cliff.
                    if current_allocations.len() != 1 {
                        return self.err_periodic_cliff_multiple(&ch.name)
                    }
                    // If the above is true, make the cliff amount
                    // the first portion, and advance the time.
                    let reason = format!("{}: cliff", &ch.name);
                    portions.push(portion(cliff, a, start_at, &reason));
                    t_cursor += interval;
                    n_portions += 1;
                    total_received += cliff;
                    total_vested += cliff;
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
                let amount = (*amount).u128();
                if addr == a {
                    let reason = format!("{}: vesting", &ch.name);
                    portions.push(portion(amount, a, t_cursor, &reason));
                    total_received += amount;
                }
                total_vested += amount;
            }

            // Advance the time.
            t_cursor += interval
        }
        // MAYBE cap this by sum, not by time?

        // If we're at/past the end, add give the remainder.
        // How does this work with multiple allocations though?
        if t_cursor >= t_end {
            let remainder = ch.amount.u128() - total_vested;
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
                } else {
                    println!("{:#?}", &ch);
                    return self.err_periodic_remainder_multiple(&ch.name);
                }
            }
        }

        Ok(portions)
    }
}

/// Allocation of vesting to multiple addresses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    amount: Uint128,
    addr:   HumanAddr,
}

/// Claimable portion
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Portion {
    pub amount:  Uint128,
    pub address: HumanAddr,
    pub vested:  Seconds,
    pub reason:  String
}
impl std::fmt::Display for Portion {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{} {} to {} at {}>", self.reason, self.amount, self.address, self.vested)
    }
}

/// Log of executed claims
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct History {
    pub history: Vec<ClaimedPortion>
}
impl History {
    pub fn new () -> Self { Self { history: vec![] } }
    /// Takes list of portions, returns the ones which aren't marked as claimed
    pub fn unclaimed (&mut self, claimable: Vec<Portion>) -> Vec<Portion> {
        // TODO sort by timestamp and validate that there is no overlap
        //      between claimed/unclaimed because that would signal an error
        let claimed_portions: Vec<Portion> =
            self.history.iter().map(|claimed| claimed.portion.clone()).collect();
        claimable.into_iter()
            .filter(|portion| !claimed_portions.contains(portion)).collect()
    }
    /// Marks a portion as claimed
    pub fn claim (&mut self, claimed: Seconds, portions: Vec<Portion>) {
        for portion in portions.iter() {
            self.history.push(ClaimedPortion {claimed, portion: portion.clone()} )
        }
    }
}

/// History entry
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimedPortion {
    portion: Portion,
    claimed: Seconds
}

#[cfg(test)]
mod tests;
