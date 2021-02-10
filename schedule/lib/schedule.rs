use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmwasm_std::{StdResult, StdError};

macro_rules! Error { ($msg:expr) => { Err(StdError::GenericErr { msg: $msg.to_string(), backtrace: None }) } }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
}
pub fn schedule (total: u128, pools: Vec<Pool>) -> Schedule {
    Schedule { total: Uint128::from(total), pools }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    pub name:     String,
    pub total:    Uint128,
    pub partial:  bool,
    pub releases: Vec<Release>,
}
pub fn pool (name: &str, total: u128, releases: Vec<Release>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), releases, partial: true }
}
pub fn pool_partial (name: &str, total: u128, releases: Vec<Release>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), releases, partial: false }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Release {
    pub name:        String,
    pub mode:        ReleaseMode,
    pub amount:      Uint128,
    pub allocations: Vec<Allocation>,
}
pub fn release_immediate (
    amount: u128,
    address: &str
) -> Release {
    Release {
        name: String::new(),
        mode: ReleaseMode::Immediate {},
        amount: Uint128::from(amount),
        allocations: vec![allocation(amount, address)]
    }
}
pub fn release_immediate_multi (
    amount: u128,
    allocations: Vec<Allocation>
) -> Release {
    Release {
        name: String::new(),
        mode: ReleaseMode::Immediate {},
        amount: Uint128::from(amount),
        allocations
    }
}
pub fn release_periodic (
    amount:   u128,
    address:  &str,
    interval: Seconds,
    start_at: Seconds,
    duration: Seconds,
    cliff:    u128
) -> Release {
    let cliff = Uint128::from(cliff);
    Release {
        name:   String::new(),
        mode:   ReleaseMode::Periodic {interval, start_at, duration, cliff},
        amount: Uint128::from(amount),
        allocations: vec![allocation(amount, address)]
    }
}
pub fn release_periodic_multi (
    amount:      u128,
    allocations: Vec<Allocation>,
    interval:    Seconds,
    start_at:    Seconds,
    duration:    Seconds,
    cliff:       u128
) -> Release {
    let cliff = Uint128::from(cliff);
    Release {
        name:   String::new(),
        mode:   ReleaseMode::Periodic {interval, start_at, duration, cliff},
        amount: Uint128::from(amount),
        allocations
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ReleaseMode {
    /// Immediate release: if the contract has launched,
    /// the recipient can claim the entire allocated amount once
    Immediate {},
    /// Periodic release: contract calculates the maximum amount
    /// that the user can claim at the given time
    Periodic {
        interval: Seconds,
        start_at: Seconds,
        duration: Seconds,
        cliff:    Uint128
    }
}

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

pub trait Account {
    /// Make sure account contains valid data
    fn validate  (&self) -> StdResult<()>;
    /// Get amount unlocked for address `a` at time `t`
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
        for release in self.releases.iter() {
            match release.validate() {
                Ok(_)  => { total += release.amount.u128() },
                Err(e) => return Err(e)
            }
        }
        let invalid_total = if self.partial {
            total > self.total.u128()
        } else {
            total != self.total.u128()
        };
        if invalid_total {
            return Error!(format!("pool ${}: releases add up to {}, expected {}", self.name, total, self.total))
        }
        Ok(())
    }
    fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128 {
        let mut claimable = 0;
        for release in self.releases.iter() {
            claimable += release.claimable(a, t)
        }
        return claimable
    }
}

impl Release {
    fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for Allocation { amount, .. } in self.allocations.iter() {
            total += amount.u128()
        }
        match &self.mode {
            _ => {},
            ReleaseMode::Periodic{interval,start_at,duration,cliff} => {
                if duration % interval > 0 {
                    return Error!(format!("release {}: duration {} does not divide evenly by {}", &self.name, duration, interval))
                }
                if (self.amount - *cliff).unwrap().u128() % Uint128::from(duration / interval).u128() > 0 {
                    return Error!(format!("release {}: post-cliff amount {} does not divide evenly by {}", &self.name, (self.amount - *cliff).unwrap(), duration / interval))
                }
            }
        }
        if total != self.portion() {
            return Error!(
                format!("release {}: allocations add up to {}, expected {}", &self.name, total, self.portion()))
        }
        Ok(())
    }
    fn portion (&self) -> u128 {
        match &self.mode {
            ReleaseMode::Immediate {} =>
                self.amount.u128(),
            ReleaseMode::Periodic{interval,start_at,duration,cliff} =>
                (self.amount - *cliff).unwrap().u128() / (duration / interval) as u128
        }
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
    fn vest (&self, amount: u128, t: Seconds) -> u128 {
        match &self.mode {
            ReleaseMode::Immediate {} =>
                amount,

            ReleaseMode::Periodic { interval, start_at, duration, cliff } => {
                // Can't vest before the cliff
                if t < *start_at { return 0 }
                crate::periodic(amount, *interval, t - start_at, *duration, cliff.u128())
            }
        }
    }
}

#[test]
fn test_release () {
    assert_eq!(release_immediate_multi(100, vec![
        allocation(40, &"Alice"),
        allocation(60, &"Bob")
    ]).claimable(&HumanAddr::from("Alice"), 0),
        40);

    assert_eq!(release_periodic_multi(100, vec![
        allocation(40, &"Alice"),
        allocation(60, &"Bob")
    ], DAY, 1, DAY, 0).claimable(&HumanAddr::from("Alice"), 0),
        0);

    assert_eq!(release_periodic_multi(100, vec![
        allocation(40, &"Alice"),
        allocation(60, &"Bob")
    ], DAY, 1, DAY, 0).claimable(&HumanAddr::from("Alice"), 1),
        40);

    // for allocations to make sense:
    todo!("allocations must be divided per release and not from the total")
}

#[test]
fn test_pool () {
    assert_eq!(pool("", 0, vec![]).validate(), Ok(()));
}

#[test]
fn test_schedule_pool_release_and_allocation () {
    assert_eq!(schedule(0, vec![]).validate(),
        Ok(()));

    assert_eq!(schedule(0, vec![]).claimable(&HumanAddr::from(""), 0),
        0);

    assert_eq!(schedule(100, vec![]).validate(),
        Error!("schedule: pools add up to 0, expected 100"));

    assert_eq!(schedule(100, vec![pool("", 50, vec![])]).validate(),
        Error!("pool: releases add up to 0, expected 50"));

    assert_eq!(schedule(100, vec![pool("", 50, vec![
                release_immediate(20, &"")])]).validate(),
        Error!("pool: releases add up to 20, expected 50"));

    assert_eq!(schedule(100, vec![pool("", 50, vec![
                release_immediate(30, &""),
                release_immediate_multi(20, vec![allocation(10, &"")
                                               ,allocation(10, &"")])])]).validate(),
        Error!("schedule: pools add up to 50, expected 100"));

    assert_eq!(schedule(100, vec![
            pool("", 50, vec![
                release_immediate(30, &""),
                release_immediate_multi(20, vec![allocation(20, &"")])
            ]),
            pool("", 50, vec![
                release_immediate_multi(30, vec![allocation(30, &"")]),
                release_immediate_multi(20, vec![allocation(20, &"")])])]).validate(),
        Ok(()));

    assert_eq!(schedule(100, vec![
        pool("", 50, vec![
            release_immediate_multi(30, vec![allocation(30, &"")]),
            release_immediate_multi(20, vec![allocation(20, &"")])
        ]),
        pool("", 50, vec![
            release_immediate_multi(30, vec![allocation(30, &"")]),
            release_immediate_multi(20, vec![allocation(20, &"")])
        ])
    ]).claimable(&HumanAddr::from(""), 0),
        100);
}
