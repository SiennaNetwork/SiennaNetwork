//! Model of vesting schedule.

use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmwasm_std::{StdResult, StdError};

macro_rules! Error {
    ($msg:expr) => { Err(StdError::GenericErr { msg: $msg.to_string(), backtrace: None }) }
}

/// Vesting schedule; contains `Pool`s that must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
}
pub fn schedule (total: u128, pools: Vec<Pool>) -> Schedule {
    Schedule { total: Uint128::from(total), pools }
}
impl Schedule {
    /// Make sure that the schedule contains valid data.
    pub fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        let mut pools: Vec<String> = vec![];
        for pool in self.pools.iter() {
            pools.push(pool.name.clone());
            match pool.validate() {
                Ok(_)  => { total += pool.total.u128() },
                Err(e) => return Err(e)
            }
        }
        if total != self.total.u128() {
            return Error!(format!("schedule: pools add up to {}, expected {}", total, self.total))
        }
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
pub fn pool (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: false }
}
pub fn pool_partial (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: true }
}
impl Pool {
    pub fn validate (&self) -> StdResult<()> {
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
        if invalid_total {
            return Error!(format!("pool {}: channels add up to {}, expected {}", self.name, total, self.total))
        }
        Ok(())
    }
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for channel in self.channels.iter() {
            portions.append(&mut channel.claimable(a, t)?);
        }
        Ok(portions)
    }
}

/// Vesting channel: contains one or more `Allocation`s and can be `Periodic`.
///
/// Here's what was casting the shadow of a off-by-one error all along,
/// and here's how it comes into play:
///
/// If `cliff > 0` then it's actually a `N+1`-day vesting, because
/// there must be an `interval` between the cliff and the 1st portion.
///
/// While maintaining this assumption, synthetic cliffs have been manually
/// added in the schedule to channels that don't divide evenly into the
/// designated portion amount.
///
/// The sizes of those cliffs have been determined experimentally with
/// a criterion that might seem esoteric: no more than 3 digits after
/// the decimal point. This is to appease the BigInt handling in
/// `tsv2json.js`.
///
/// Effectively, this means that the first time users make a claim,
/// it's going to be a smaller amount than the regular one that they'll
/// get during the following intervals, because they'll be receiving the
/// remainder of the division of the amount into portions as a "cliff".
///
/// TODO: validate expected portion sizes from spreadsheet against actual
///       ones calculated by the contract to see if this discrepancy is
///       handled in the same way?
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    pub name:   String,
    pub amount: Uint128,

    /// Each portion can be split between multiple addresses.
    pub allocations: Vec<(Seconds, Vec<Allocation>)>,

    /// Immediate channel: if the contract has launched,
    /// the recipient can claim the entire allocated amount once
    /// Periodic channel: contract calculates the maximum amount
    /// that the user can claim at the given time.
    pub periodic: Option<Periodic>
}
pub fn channel_immediate (
    amount: u128,
    address: &HumanAddr
) -> Channel {
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: None,
        allocations: vec![(0, vec![allocation(amount, address)])],
    }
}
pub fn channel_immediate_multi (
    amount: u128,
    allocations: &Vec<Allocation>
) -> Channel {
    panic!("immediate vesting with multiple recipients is not supported")
    //Channel {
        //name: String::new(),
        //amount: Uint128::from(amount),
        //periodic: None,
        //allocations: vec![(0, allocations.clone())]
    //}
}
pub fn channel_periodic (
    amount:   u128,
    address:  &HumanAddr,
    interval: Seconds,
    start_at: Seconds,
    duration: Seconds,
    cliff:    u128
) -> StdResult<Channel> {
    let cliff = Uint128::from(cliff);
    let mut channel = Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(Periodic {interval, start_at, duration, cliff}),
        allocations: vec![]
    };
    channel.allocations.push((0, vec![allocation(channel.regular_portion_size()?, address)]));
    Ok(channel)
}
pub fn channel_periodic_multi (
    amount:      u128,
    allocations: &Vec<Allocation>,
    interval:    Seconds,
    start_at:    Seconds,
    duration:    Seconds,
    cliff:       u128
) -> Channel {
    if cliff > 0 { panic!("periodic vesting with cliff and multiple recipients is not supported") }
    let cliff = Uint128::from(cliff);
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(Periodic {interval, start_at, duration, cliff}),
        allocations: vec![(0, allocations.clone())]
    }
}
impl Channel {
    pub fn validate (&self) -> StdResult<()> {
        match &self.periodic {
            None => {},
            Some(Periodic{cliff,duration,interval,..}) => {
                if *duration < 1 {
                    return Error!(format!(
                        "channel {}: periodic vesting's duration can't bw 0",
                        &self.name))
                }
                if *interval < 1 {
                    return Error!(format!(
                        "channel {}: periodic vesting's interval can't be 0",
                        &self.name))
                }
                if *cliff > self.amount {
                    return Error!(format!(
                        "channel {}: cliff {} can't be larger than total amount {}",
                        &self.name, cliff, self.amount))
                }
                for (_, allocations) in self.allocations.iter() {
                    if allocations.len() > 1 && cliff.u128() > 0 {
                        return Error!(format!(
                            "channel {}: periodic vesting with cliff and multiple recipients is not supported",
                            &self.name))
                    }
                }
            }
        }
        for (_, allocations) in self.allocations.iter() {
            let mut total_portion = 0u128;
            for Allocation { amount, .. } in allocations.iter() {
                total_portion += amount.u128()
            }
            let regular_portion_size = self.regular_portion_size()?;
            if total_portion != regular_portion_size {
                return Error!(
                    format!("channel {}: allocations add up to {}, expected {}",
                    &self.name, total_portion, regular_portion_size))
            }
        }
        Ok(())
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
            None    => self.claimable_immediate(a, t),
            Some(p) => self.claimable_periodic(a, t, p),
        }
    }

    fn claimable_immediate (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        match self.allocations.get(self.allocations.len()-1) {
            None => return Error!(format!("{}: no allocations", &self.name)),
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

    fn claimable_periodic (&self, a: &HumanAddr, t: Seconds, p: &Periodic) -> StdResult<Vec<Portion>> {

        let Periodic{start_at,cliff,interval,duration} = p;

        // Interval can't be 0 (prevent infinite loop below)
        if *interval == 0 {
            return Error!(format!("{}: interval can't be 0", &self.name))
        }

        // Nothing can be claimed before the start
        if t < *start_at {
            return Ok(vec![])
        }

        // Now comes the fun part, where we iterate over the time range
        // `start_at..min(t, start_at+duration)` in steps of `interval`,
        // and add vestings in accordance with the allocations that are
        // current for the particular moment in time.
        let mut portions = vec![];
        let mut t_cursor = *start_at;
        let cliff  = cliff.u128();
        let end_at = Seconds::min(t, *start_at + *duration);
        let mut n_portions = self.regular_portion_count()?;

        // Make sure allocations exist.
        if self.allocations.len() < 1 {
            return Error!(format!("{}: launched with no allocations", &self.name))
        }

        // Get first group of allocations.
        let (t_alloc, current_allocations) = self.allocations.get(0).unwrap();
        let mut current_allocations = current_allocations;
        if *t_alloc > t_cursor {
            return Error!(format!("{}: time of first allocations is after current time", &self.name))
        }

        // If this is the first iteration of this `loop`, and the `channel`
        // has a `cliff`, and the first group of `allocations` contains the
        // claimant `a`, then that user must receive the cliff amount.
        // (TODO: and divide rest by 1 less?)
        if cliff > 0 {
            for Allocation { addr, amount } in current_allocations.iter() {
                if addr == a {
                    // The first group of allocations must contain exactly
                    // 1 user to avoid splitting the cliff.
                    if current_allocations.len() != 1 {
                        return Error!(format!("{}: if cliff is present there must be exactly one allocation", &self.name));
                    }
                    // If the above is true, make the cliff amount claimable
                    // as the first portion, and advance the time.
                    let reason = format!("{}: cliff", &self.name);
                    portions.push(portion(cliff, a, *start_at, &reason));
                    t_cursor += interval;
                    n_portions += 1;
                    break
                }
            }
        }

        loop {
            // After the first cliff, add a new claimable portion
            // for every `interval` seconds unless `t_cursor` is
            // past the current time or the end time.
            if t_cursor > t { break }
            if t_cursor >= start_at + n_portions * interval { break }

            // Determine the group of allocations that is current
            // at time `t_cursor`. (It is assumed that groups of
            // allocations are sorted).
            for (t_alloc, allocations) in self.allocations.iter() {
                if *t_alloc > t_cursor { break }
                current_allocations = allocations;
            }
            
            // From the current group of allocations, determine
            // the actual claimable amount, and add the
            // corresponding portion.
            for Allocation { addr, amount } in current_allocations.iter() {
                if addr == a {
                    let amount = (*amount).u128();
                    let reason = format!("{}: vesting", &self.name);
                    portions.push(portion(amount, a, t_cursor, &reason));
                }
            }

            // Advance the time.
            t_cursor += interval
        }
        Ok(portions)
    }

    /// 1 if immediate, or `duration/interval` if periodic.
    /// Returns error if `duration` is not a multiple of `interval`.
    pub fn regular_portion_count (&self) -> StdResult<u64> {
        match &self.periodic {
            None {} =>
                Ok(1),
            Some(Periodic{interval,duration,cliff,..}) =>
                if duration % interval > 0 {
                    Error!(format!(
                        "channel {}: duration {} does not divide evenly in intervals of {}",
                        &self.name, duration, interval))
                } else {
                    if duration < interval {
                        return Error!(format!("channel {}: duration ({}) must be >= interval ({})", &self.name, duration, interval));
                    }
                    let mut n_portions = duration / interval;
                    if *cliff > Uint128::zero() {
                        if n_portions < 2 {
                            return Error!(format!("channel {}: periodic vesting must contain at least 1 non-cliff portion", &self.name))
                        }
                        n_portions -= 1;
                    }
                    Ok(n_portions)
                }
        }
    }
    /// Full `amount` if immediate, or `(amount-cliff)/regular_portion_count` if periodic.
    /// Returns error if amount can't be divided evenly in that number of portions.
    pub fn regular_portion_size (&self) -> StdResult<u128> {
        match &self.periodic {
            None {} =>
                Ok(self.amount.u128()),
            Some(Periodic{cliff,duration,interval,..}) => {
                let amount_after_cliff = (self.amount - *cliff).unwrap().u128();
                let mut n_portions = self.regular_portion_count()? as u128;
                if amount_after_cliff % n_portions > 0 {
                    Error!(format!(
                        "channel {}: post-cliff amount {} does not divide evenly in {} portions",
                        &self.name, amount_after_cliff, n_portions))
                } else {
                    Ok(amount_after_cliff / n_portions)
                }
            }
        }
    }
    /// Allocations can be changed on the fly without affecting past vestings.
    pub fn reallocate (&mut self, t: Seconds, allocations: Vec<Allocation>) -> StdResult<()> {
        let t_max = self.allocations.iter().fold(0, |x,y|Seconds::max(x,y.0));
        if t < t_max {
            return Error!(format!("channel {}: can not reallocate in the past ({} < {})",
                &self.name, &t, t_max))
        }
        self.allocations.push((t, allocations));
        self.validate()
    }
}
/// Configuration of periodic vesting ladder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Periodic {
    pub interval: Seconds,
    pub start_at: Seconds,
    pub duration: Seconds,
    pub cliff:    Uint128
}
/// Allocation of vesting to multiple addresses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    amount: Uint128,
    addr:   HumanAddr,
}
pub fn allocation (amount: u128, addr: &HumanAddr) -> Allocation {
    Allocation { amount: Uint128::from(amount), addr: addr.clone() }
}
/// Claimable portion / history entry.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Portion {
    pub amount:  Uint128,
    pub address: HumanAddr,
    pub vested:  Seconds,
    pub reason:  String
}
pub fn portion (amt: u128, addr: &HumanAddr, vested: Seconds, reason: &str) -> Portion {
    Portion {
        amount:  Uint128::from(amt),
        address: addr.clone(),
        vested:  vested,
        reason:  reason.to_string()
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
mod tests {
    use crate::units::*;
    use super::*;

    #[test]
    fn test_schedule () {
        assert_eq!(schedule(0, vec![]).validate(),
            Ok(()));
        assert_eq!(schedule(0, vec![]).claimable(&HumanAddr::from(""), 0),
            Ok(vec![]));
        assert_eq!(schedule(100, vec![]).validate(),
            Error!("schedule: pools add up to 0, expected 100"));
        assert_eq!(schedule(100, vec![pool("P1", 50, vec![])]).validate(),
            Error!("pool P1: channels add up to 0, expected 50"));
        assert_eq!(schedule(0, vec![pool("P1", 0, vec![]), pool("P2", 0, vec![])]).validate(),
            Ok(()));
        //assert_eq!(schedule(0, vec![pool("P1", 0, vec![]), pool("P2", 0, vec![])]).validate(),
            //Error!("schedule: duplicate pool name P1"));
    }

    #[test]
    fn test_invalid_schedules () {
        let alice = HumanAddr::from("Alice");
        assert_eq!(schedule(100, vec![
            pool("P1", 50, vec![channel_immediate(20, &alice)]),
            pool("P2", 50, vec![channel_immediate(30, &alice)])
        ]).validate(),
            Error!("pool P1: channels add up to 20, expected 50"));

        assert_eq!(schedule(100, vec![
            pool("", 50, vec![
                channel_immediate(30, &alice),
                channel_periodic_multi(20, &vec![allocation(10, &alice)
                                                 ,allocation(10, &alice)], 1, 0, 1, 0)
            ])]).validate(),
            Error!("schedule: pools add up to 50, expected 100"));
    }

    #[test]
    fn test_valid_schedule_with_main_features () {
        let alice = HumanAddr::from("Alice");
        let bob = HumanAddr::from("Bob");

        let s = schedule(110, vec![
            pool("P1", 50, vec![
                channel_immediate(29, &alice),
                channel_immediate(1, &bob),
                channel_periodic(20, &alice, 1, 0, 1, 0).unwrap()
            ]),
            pool("P2", 60, vec![
                channel_periodic(10, &alice, 1, 0, 2, 2).unwrap(),
                channel_periodic_multi(50, &vec![
                    allocation(28, &alice),
                    allocation( 3, &bob),
                    allocation(19, &alice)
                ], 1, 0, 1, 0)])]);

        assert_eq!(s.validate(),
            Ok(()));
        assert_eq!(s.claimable(&alice, 0),
            Ok(vec![
                portion(29u128, &alice, 0u64, ": immediate"),
                portion(20u128, &alice, 0u64, ": vesting"),
                portion( 2u128, &alice, 0u64, ": cliff"),
                portion(28u128, &alice, 0u64, ": vesting"),
                portion(19u128, &alice, 0u64, ": vesting")
            ]));
        assert_eq!(s.claimable(&bob, 0),
            Ok(vec![
                portion(1u128, &bob, 0u64, ": immediate"),
                portion(3u128, &bob, 0u64, ": vesting"),
            ]));
    }

    #[test]
    fn test_pool () {
        assert_eq!(pool("", 0, vec![]).validate(),
            Ok(()));
    }

    #[test]
    fn test_channel_immediate () {
        let alice = HumanAddr::from("Alice");
        assert_eq!(
            channel_immediate(100, &alice).claimable(&alice, 0),
            Ok(vec![portion(100u128, &alice, 0u64, ": immediate")]));
    }

    #[test]
    fn test_channel_periodic_no_cliff () {
        let total    = 300;
        let interval = DAY;
        let start_at = 100;
        let duration = 3*DAY;
        let cliff    = 0;
        let alice    = HumanAddr::from("Alice");
        let bob      = HumanAddr::from("Bob");

        let c = schedule(total,vec![pool("P1",total,vec![channel_periodic_multi(
            total, &vec![
                allocation(40, &alice),
                allocation(60, &bob)
            ], interval, start_at, duration, cliff)])]);

        assert_eq!(c.claimable(&alice, start_at - 1),
            Ok(vec![]));

        assert_eq!(c.claimable(&alice, start_at),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 1),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + interval),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 1*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + interval + interval / 2),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 1*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 2*interval),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 1*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 2*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 2*interval + interval / 2),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 1*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 2*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 3*interval),
            Ok(vec![portion( 40u128, &alice, start_at + 0*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 1*interval, ": vesting")
                   ,portion( 40u128, &alice, start_at + 2*interval, ": vesting")]));
    }

    #[test]
    fn test_channel_periodic_with_cliff () {
        let total    = 100;
        let interval = DAY;
        let start_at = 1;
        let alice = HumanAddr::from("Alice");
        let bob = HumanAddr::from("Bob");

        // if cliff > 0 then the first portion is the cliff
        // and the remaining amount is divided between `regular_portion_count-1`
        // duration = interval -> [cliff]
        // but that should be an immediate channel instead, so it fails
        let duration = interval;
        let cliff = 1u128;
        assert_eq!(channel_periodic(total, &alice, interval, start_at, duration, cliff),
            Error!("channel : periodic vesting must contain at least 1 non-cliff portion"));

        // duration = 2*interval -> [cliff, vesting]
        let duration = 2*interval;
        let c = channel_periodic(total, &alice, interval, start_at, duration, cliff).unwrap();

        assert_eq!(c.regular_portion_count(),
            Ok(1));

        assert_eq!(c.regular_portion_size(),
            Ok(99u128));

        assert_eq!(c.claimable(&alice, start_at),
            Ok(vec![portion(cliff, &alice, start_at, ": cliff")]));

        assert_eq!(c.claimable(&alice, start_at + 1),
            Ok(vec![portion(cliff, &alice, start_at, ": cliff")]));

        assert_eq!(c.claimable(&alice, start_at + interval),
            Ok(vec![portion(cliff,  &alice, start_at,          ": cliff")
                   ,portion(99u128, &alice, start_at+interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 10*interval),
            Ok(vec![portion(cliff,  &alice, start_at,          ": cliff")
                   ,portion(99u128, &alice, start_at+interval, ": vesting")]));

        // duration = 3*interval -> [cliff, vesting, vesting]
        let duration = 3*interval;
        assert_eq!(channel_periodic(total, &alice, interval, start_at, duration, cliff),
            Error!("channel : post-cliff amount 99 does not divide evenly in 2 portions"));
        let total = 201;
        let c = channel_periodic(total, &alice, interval, start_at, duration, cliff).unwrap();

        assert_eq!(c.regular_portion_count(),
            Ok(2));

        assert_eq!(c.regular_portion_size(),
            Ok(100u128));

        assert_eq!(c.claimable(&alice, start_at),
            Ok(vec![portion(cliff, &alice, start_at, ": cliff")]));

        assert_eq!(c.claimable(&alice, start_at + 1),
            Ok(vec![portion(cliff, &alice, start_at, ": cliff")]));

        assert_eq!(c.claimable(&alice, start_at + interval),
            Ok(vec![portion(cliff,   &alice, start_at,             ": cliff")
                   ,portion(100u128, &alice, start_at+interval,   ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 2*interval),
            Ok(vec![portion(cliff,   &alice, start_at,             ": cliff")
                   ,portion(100u128, &alice, start_at+interval,   ": vesting")
                   ,portion(100u128, &alice, start_at+2*interval, ": vesting")]));

        assert_eq!(c.claimable(&alice, start_at + 10*interval),
            Ok(vec![portion(cliff,   &alice, start_at,             ": cliff")
                   ,portion(100u128, &alice, start_at+interval,   ": vesting")
                   ,portion(100u128, &alice, start_at+2*interval, ": vesting")]));
    }

    #[test]
    fn test_reallocation () {
        let alice = HumanAddr::from("Alice");
        let bob   = HumanAddr::from("Bob");

        let interval = DAY;
        let start_at = 0;
        let duration = 10 * DAY;
        let cliff    = 0;

        let mut s = channel_periodic_multi(1000u128, &vec![
            allocation(75u128, &alice),
            allocation(25u128, &bob),
        ], interval, start_at, duration, cliff);
        let claimable = s.claimable(&alice, 0);
        assert_eq!(s.claimable(&alice, 0 * DAY),
            Ok(vec![portion(75u128, &alice, 0 * DAY, ": vesting")]));
        assert_eq!(s.claimable(&alice, 1 * DAY),
            Ok(vec![portion(75u128, &alice, 0 * DAY, ": vesting")
                   ,portion(75u128, &alice, 1 * DAY, ": vesting")]));
        assert_eq!(s.claimable(&alice, 2 * DAY),
            Ok(vec![portion(75u128, &alice, 0 * DAY, ": vesting")
                   ,portion(75u128, &alice, 1 * DAY, ": vesting")
                   ,portion(75u128, &alice, 2 * DAY, ": vesting")]));
        s.reallocate(3 * DAY + 1, vec![
            allocation(50u128, &alice),
            allocation(50u128, &bob)
        ]).unwrap();
        assert_eq!(s.claimable(&alice, 3 * DAY),
            Ok(vec![portion(75u128, &alice, 0 * DAY, ": vesting")
                   ,portion(75u128, &alice, 1 * DAY, ": vesting")
                   ,portion(75u128, &alice, 2 * DAY, ": vesting")
                   ,portion(50u128, &alice, 3 * DAY, ": vesting")]));
        assert_eq!(s.claimable(&alice, 4 * DAY),
            Ok(vec![portion(75u128, &alice, 0 * DAY, ": vesting")
                   ,portion(75u128, &alice, 1 * DAY, ": vesting")
                   ,portion(75u128, &alice, 2 * DAY, ": vesting")
                   ,portion(50u128, &alice, 3 * DAY, ": vesting")
                   ,portion(50u128, &alice, 4 * DAY, ": vesting")]));
    }
}
