//! Model of vesting schedule.

use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmwasm_std::{StdResult, StdError};

macro_rules! Error {
    ($msg:expr) => { Err(StdError::GenericErr { msg: $msg.to_string(), backtrace: None }) }
}

/// Root schedule; contains `Pool`s that must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
}
pub fn schedule (total: u128, pools: Vec<Pool>) -> Schedule {
    Schedule { total: Uint128::from(total), pools }
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

/// Vesting channel: contains one or more `Allocation`s and can be `Periodic`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    pub name:   String,
    pub amount: Uint128,

    /// Each portion is split between these addresses.
    pub allocations: Vec<Allocation>,

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
        allocations: vec![allocation_addr(amount, address)],
    }
}
pub fn channel_immediate_multi (
    amount: u128,
    allocations: &Vec<Allocation>
) -> Channel {
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: None,
        allocations: allocations.clone()
    }
}
pub fn channel_periodic (
    amount:   u128,
    address:  &HumanAddr,
    interval: Seconds,
    start_at: Seconds,
    duration: Seconds,
    cliff:    u128
) -> Channel {
    let cliff = Uint128::from(cliff);
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(Periodic {interval, start_at, duration, cliff}),
        allocations: vec![allocation_addr(amount, address)]
    }
}
pub fn channel_periodic_multi (
    amount:      u128,
    allocations: &Vec<Allocation>,
    interval:    Seconds,
    start_at:    Seconds,
    duration:    Seconds,
    cliff:       u128
) -> Channel {
    let cliff = Uint128::from(cliff);
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(Periodic {interval, start_at, duration, cliff}),
        allocations: allocations.clone()
    }
}
impl Channel {
    fn portion_count (&self) -> StdResult<u64> {
        match &self.periodic {
            None {} =>
                Ok(1),
            Some(Periodic{interval,duration,..}) =>
                if duration % interval > 0 {
                    Error!(format!(
                        "channel {}: duration {} does not divide evenly in intervals of {}",
                        &self.name, duration, interval))
                } else {
                    Ok(duration / interval)
                }
        }
    }
    fn portion_size (&self) -> StdResult<u128> {
        match &self.periodic {
            None {} =>
                Ok(self.amount.u128()),
            Some(Periodic{cliff,duration,interval,..}) => {
                let amount_after_cliff = (self.amount - *cliff).unwrap().u128();
                let portion_count = self.portion_count()? as u128;
                if amount_after_cliff % portion_count > 0 {
                    Error!(format!(
                        "channel {}: post-cliff amount {} does not divide evenly in {} portions",
                        &self.name, amount_after_cliff, duration / interval))
                } else {
                    Ok(amount_after_cliff / portion_count)
                }
            }
        }
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
pub fn allocation (amount: u128, addr: &str) -> Allocation {
    Allocation { amount: Uint128::from(amount), addr: HumanAddr::from(addr) }
}
pub fn allocation_addr (amount: u128, addr: &HumanAddr) -> Allocation {
    Allocation { amount: Uint128::from(amount), addr: addr.clone() }
}

pub trait Named {
    fn get_name (&self) -> &str;
}
impl Named for Pool {
    fn get_name (&self) -> &str { &self.name }
}
fn check_for_duplicates (items: &Vec<impl Named>, msg: &str) -> StdResult<()> {
    let mut names: Vec<String> = vec![];
    for item in items.iter() {
        let item_name = item.get_name();
        for visited_name in names.iter() {
            if item_name == *visited_name {
                return Error!(format!("{} {}", msg, &item_name))
            }
        }
        names.push(item_name.into())
    }
    Ok(())
}

/// Allow for validation and computing of `claimable`.
pub trait Account {
    /// Make sure account contains valid data.
    fn validate  (&self) -> StdResult<()>;
    /// Get amount unlocked for address `a` at time `t`.
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>>;
}
impl Account for Schedule {
    fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        check_for_duplicates(&self.pools, "schedule: duplicate pool name")?;
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
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for pool in self.pools.iter() {
            portions.append(&mut pool.claimable(a, t)?);
        }
        Ok(portions)
    }
}
impl Account for Pool {
    fn validate (&self) -> StdResult<()> {
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
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        for channel in self.channels.iter() {
            portions.append(&mut channel.claimable(a, t)?);
        }
        Ok(portions)
    }
}
impl Account for Channel {
    fn validate (&self) -> StdResult<()> {
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
                if self.allocations.len() > 1 && cliff.u128() > 0 {
                    return Error!(format!(
                        "channel {}: cliff not supported with multiple allocations",
                        &self.name))
                }
            }
        }
        let mut total_portion = 0u128;
        for Allocation { amount, .. } in self.allocations.iter() {
            total_portion += amount.u128()
        }
        let portion_size = self.portion_size()?;
        if total_portion != portion_size {
            return Error!(
                format!("channel {}: allocations add up to {}, expected {}",
                &self.name, total_portion, portion_size))
        }
        Ok(())
    }
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> StdResult<Vec<Portion>> {
        let mut portions = vec![];
        match &self.periodic {
            None => {
                if *a == self.allocations.get(0).unwrap().addr {
                    let reason = format!("{}: immediate", &self.name);
                    portions.push(portion(self.amount.u128(), a, 0, &reason));
                }
            },
            Some(Periodic{start_at,cliff,interval,..}) => {
                if t >= *start_at {
                    let elapsed = t - start_at;
                    let n_portions = u64::min(
                        self.portion_count()?,
                        elapsed / interval
                    );
                    let mut for_me = false;
                    for Allocation { addr, amount } in self.allocations.iter() {
                        if addr == a {
                            for_me = true;
                            for n_portion in 0..n_portions {
                                let reason = format!("{}: vesting", &self.name);
                                let t_vested = start_at + n_portion * interval;
                                portions.push(portion((*amount).u128(), a, t_vested, &reason));
                            }
                        }
                    }
                    if for_me {
                        let reason = format!("{}: cliff", &self.name);
                        portions.insert(0, portion((*cliff).u128(), a, *start_at, &reason));
                    }
                }
            }
        }
        Ok(portions)
    }
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
/// History entry
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimedPortion {
    portion: Portion,
    claimed: Seconds
}
/// Log of executed claims
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct History {
    pub history: Vec<ClaimedPortion>
}
impl History {
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
        assert_eq!(schedule(0, vec![pool("P1", 0, vec![]), pool("P1", 0, vec![])]).validate(),
            Error!("schedule: duplicate pool name P1"));
    }

    #[test]
    fn test_pool () {
        assert_eq!(pool("", 0, vec![]).validate(),
            Ok(()));
    }

    #[test]
    fn test_channel () {
        let alice = HumanAddr::from("Alice");
        let bob   = HumanAddr::from("Bob");
        let allocations = vec![
            allocation_addr(40, &alice),
            allocation_addr(60, &bob)
        ];

        assert_eq!(
            channel_immediate_multi(100, &allocations).claimable(&alice, 0),
            Ok(vec![portion( 40u128, &alice, 0u64, ": immediate")]));

        assert_eq!(
            channel_periodic_multi(100, &allocations, DAY, 1, DAY, 0).claimable(&alice, 0),
            Ok(vec![]));

        assert_eq!(
            channel_periodic_multi(100, &allocations, DAY, 1, DAY, 0).claimable(&alice, 1),
            Ok(vec![portion( 40u128, &alice, 0u64, ": immediate")]));

        assert_eq!(
            channel_periodic_multi(100, &allocations, DAY, 1, 2*DAY, 0).validate(),
            Error!("channel : allocations add up to 100, expected 50"));

        assert_eq!(
            channel_periodic_multi(200, &allocations, DAY, 1, 2*DAY, 0).validate(),
            Ok(()));

        assert_eq!(
            channel_periodic_multi(200, &allocations, DAY, 1, 2*DAY, 0).claimable(&alice, 1),
            Ok(vec![portion( 40u128, &alice, 0u64, ": immediate")]));

        assert_eq!(
            channel_periodic_multi(200, &allocations, DAY, 1, 2*DAY, 0).claimable(&alice, DAY + 1),
            Ok(vec![portion( 40u128, &alice, 0*DAY, ": vesting"),
                    portion( 40u128, &alice, 1*DAY, ": vesting")]));

        assert_eq!(
            channel_periodic(201, &alice, DAY, 1, 2*DAY, 0).claimable(&alice, 1),
            Ok(vec![portion(  1u128, &alice, 0*DAY, ": cliff"),
                    portion(100u128, &alice, 0*DAY, ": vesting")]));

        assert_eq!(
            channel_periodic(201, &alice, DAY, 1, 2*DAY, 0).claimable(&alice, DAY + 1),
            Ok(vec![portion(  1u128, &alice, 0*DAY, ": cliff"),
                    portion(100u128, &alice, 0*DAY, ": vesting"),
                    portion(100u128, &alice, 1*DAY, ": vesting")]));
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
                channel_immediate_multi(20, &vec![allocation_addr(10, &alice)
                                                 ,allocation_addr(10, &alice)])
            ])]).validate(),
            Error!("schedule: pools add up to 50, expected 100"));
    }

    #[test]
    fn test_valid_schedule_with_all_features () {
        let alice = HumanAddr::from("Alice");
        let s = schedule(
            100,
            vec![pool("P1", 50,
                vec![channel_immediate(30, &alice)
                    ,channel_immediate_multi(20,
                        &vec![allocation_addr(20, &alice)])]),
                pool("P2", 50,
                    vec![channel_immediate_multi(50,
                        &vec![allocation_addr(30, &alice)
                             ,allocation_addr(20, &alice)])])
                ]);

        assert_eq!(s.validate(),
            Ok(()));

        assert_eq!(s.claimable(&alice, 0),
            Ok(vec![
                portion(30u128, &alice, 0u64, ": immediate"),
                portion(20u128, &alice, 0u64, ": immediate"),
                portion(20u128, &alice, 0u64, ": immediate"),
                portion(30u128, &alice, 0u64, ": immediate")
            ]));
    }

    #[test]
    fn test_history () {
    }
}
