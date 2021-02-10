//! Model of vesting schedule.

use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmwasm_std::{StdResult, StdError};

macro_rules! Error { ($msg:expr) => { Err(StdError::GenericErr { msg: $msg.to_string(), backtrace: None }) } }

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
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: true }
}
pub fn pool_partial (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: false }
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
    address: &str
) -> Channel {
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: None,
        allocations: vec![allocation(amount, address)],
    }
}
pub fn channel_immediate_multi (
    amount: u128,
    allocations: Vec<Allocation>
) -> Channel {
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: None,
        allocations
    }
}
pub fn channel_periodic (
    amount:   u128,
    address:  &str,
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
        allocations: vec![allocation(amount, address)]
    }
}
pub fn channel_periodic_multi (
    amount:      u128,
    allocations: Vec<Allocation>,
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
        allocations
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

/// Allow for validation and computing of `claimable`.
pub trait Account {
    /// Make sure account contains valid data.
    fn validate  (&self) -> StdResult<()>;
    /// Get amount unlocked for address `a` at time `t`.
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128;
}

impl Account for Schedule {
    fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for pool in self.pools.iter() {
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
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128 {
        let mut claimable = 0;
        for pool in self.pools.iter() {
            claimable += pool.claimable(a, t)
        }
        return claimable
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
            return Error!(format!("pool ${}: channels add up to {}, expected {}", self.name, total, self.total))
        }
        Ok(())
    }
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128 {
        let mut claimable = 0;
        for channel in self.channels.iter() {
            claimable += channel.claimable(a, t)
        }
        return claimable
    }
}

impl Account for Channel {
    fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for Allocation { amount, .. } in self.allocations.iter() {
            total += amount.u128()
        }
        match &self.periodic {
            None => {},
            Some(Periodic{start_at,cliff,duration,interval}) => {
                if duration % interval > 0 {
                    return Error!(format!(
                        "channel {}: duration {} does not divide evenly in intervals of {}",
                        &self.name, duration, interval))
                }
                if (self.amount - *cliff).unwrap().u128() % Uint128::from(duration / interval).u128() > 0 {
                    return Error!(format!(
                        "channel {}: post-cliff amount {} does not divide evenly in {} portions",
                        &self.name, (self.amount - *cliff).unwrap(), duration / interval))
                }
            }
        }
        if total != self.portion() {
            return Error!(
                format!("channel {}: allocations add up to {}, expected {}", &self.name, total, self.portion()))
        }
        Ok(())
    }
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128 {
        let mut claimable = 0;
        for Allocation { addr, amount } in self.allocations.iter() {
            if addr == a {
                claimable += self.vest((*amount).u128(), t)
            }
        }
        return claimable
    }
}

impl Channel {
    fn portion (&self) -> u128 {
        match &self.periodic {
            None {} =>
                self.amount.u128(),
            Some(Periodic{interval,start_at,duration,cliff}) =>
                (self.amount - *cliff).unwrap().u128() / (duration / interval) as u128
        }
    }
    fn vest (&self, amount: u128, t: Seconds) -> u128 {
        match &self.periodic {
            None {} =>
                amount,
            Some(Periodic{interval,start_at,duration,cliff}) => {
                // Can't vest before the cliff
                if t < *start_at { return 0 }
                crate::periodic(amount, cliff.u128(), *interval, *duration, t - start_at)
            }
        }
    }
}

#[test]
fn test_channel () {
    assert_eq!(channel_immediate_multi(100, vec![
        allocation(40, &"Alice"),
        allocation(60, &"Bob")
    ]).claimable(&HumanAddr::from("Alice"), 0),
        40);

    assert_eq!(channel_periodic_multi(100, vec![
        allocation(40, &"Alice"),
        allocation(60, &"Bob")
    ], DAY, 1, DAY, 0).claimable(&HumanAddr::from("Alice"), 0),
        0);

    assert_eq!(channel_periodic_multi(100, vec![
        allocation(40, &"Alice"),
        allocation(60, &"Bob")
    ], DAY, 1, DAY, 0).claimable(&HumanAddr::from("Alice"), 1),
        40);

    // for allocations to make sense:
    todo!("allocations must be divided per channel and not from the total")
}

#[test]
fn test_pool () {
    assert_eq!(pool("", 0, vec![]).validate(), Ok(()));
}

#[test]
fn test_schedule_pool_channel_and_allocation () {
    assert_eq!(schedule(0, vec![]).validate(),
        Ok(()));

    assert_eq!(schedule(0, vec![]).claimable(&HumanAddr::from(""), 0),
        0);

    assert_eq!(schedule(100, vec![]).validate(),
        Error!("schedule: pools add up to 0, expected 100"));

    assert_eq!(schedule(100, vec![pool("", 50, vec![])]).validate(),
        Error!("pool: channels add up to 0, expected 50"));

    assert_eq!(schedule(100, vec![pool("", 50, vec![
                channel_immediate(20, &"")])]).validate(),
        Error!("pool: channels add up to 20, expected 50"));

    assert_eq!(schedule(100, vec![pool("", 50, vec![
                channel_immediate(30, &""),
                channel_immediate_multi(20, vec![allocation(10, &"")
                                               ,allocation(10, &"")])])]).validate(),
        Error!("schedule: pools add up to 50, expected 100"));

    assert_eq!(schedule(100, vec![
            pool("", 50, vec![
                channel_immediate(30, &""),
                channel_immediate_multi(20, vec![allocation(20, &"")])
            ]),
            pool("", 50, vec![
                channel_immediate_multi(30, vec![allocation(30, &"")]),
                channel_immediate_multi(20, vec![allocation(20, &"")])])]).validate(),
        Ok(()));

    assert_eq!(schedule(100, vec![
        pool("", 50, vec![
            channel_immediate_multi(30, vec![allocation(30, &"")]),
            channel_immediate_multi(20, vec![allocation(20, &"")])
        ]),
        pool("", 50, vec![
            channel_immediate_multi(30, vec![allocation(30, &"")]),
            channel_immediate_multi(20, vec![allocation(20, &"")])
        ])
    ]).claimable(&HumanAddr::from(""), 0),
        100);
}
