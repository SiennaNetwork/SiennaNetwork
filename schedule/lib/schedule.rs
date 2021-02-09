use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmwasm_std::{StdResult, StdError};

macro_rules! Error { ($msg:expr) => { Err(StdError::GenericErr { msg: $msg.to_string(), backtrace: None }) } }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total: Uint128,
    pub pools: Vec<Pool>
}
impl Schedule {
    pub fn new (total: u128, pools: Vec<Pool>) -> Self {
        Self { total: Uint128::from(total), pools }
    }

    pub fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for pool in self.pools.iter() {
            match pool.validate() {
                Ok(_)  => { total += pool.total.u128() },
                Err(e) => return Err(e)
            }
        }
        if total != self.total.u128() {
            return Error!(format!("schedule's pools add up to {}, expected {}", total, self.total))
        }
        Ok(())
    }

    /// Get amount unlocked for address `a` at time `t`
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128 {
        let mut claimable = 0;
        for Pool { accounts, .. } in self.pools.iter() {
            for account in accounts.iter() {
                claimable += account.claimable(a, t)
            }
        }
        return claimable
    }
}

#[test]
fn test_schedule () {
    assert_eq!(Schedule::new(0, vec![]).validate(),
        Ok(()));

    assert_eq!(Schedule::new(0, vec![]).claimable(&HumanAddr::from(""), 0),
        0);

    assert_eq!(Schedule::new(100, vec![]).validate(),
        Error!("schedule's pools add up to 0, expected 100"));

    assert_eq!(Schedule::new(100, vec![Pool::new(50, vec![])]).validate(),
        Error!("pool's accounts add up to 0, expected 50"));

    assert_eq!(Schedule::new(100, vec![Pool::new(50, vec![
                Account::new_immediate(20, vec![])])]).validate(),
        Error!("account's allocations add up to 0, expected 20"));

    assert_eq!(Schedule::new(100, vec![Pool::new(50, vec![
                Account::new_immediate(20, vec![Allocation::new(20, HumanAddr::from(""))])])]).validate(),
        Error!("pool's accounts add up to 20, expected 50"));

    assert_eq!(Schedule::new(100, vec![Pool::new(50, vec![
                Account::new_immediate(30, vec![Allocation::new(30, HumanAddr::from(""))]),
                Account::new_immediate(20, vec![Allocation::new(20, HumanAddr::from(""))])])]).validate(),
        Error!("schedule's pools add up to 50, expected 100"));

    assert_eq!(Schedule::new(100, vec![
            Pool::new(50, vec![
                Account::new_immediate(30, vec![Allocation::new(30, HumanAddr::from(""))]),
                Account::new_immediate(20, vec![Allocation::new(20, HumanAddr::from(""))])]),
            Pool::new(50, vec![
                Account::new_immediate(30, vec![Allocation::new(30, HumanAddr::from(""))]),
                Account::new_immediate(20, vec![Allocation::new(20, HumanAddr::from(""))])])]).validate(),
        Ok(()));

    assert_eq!(Schedule::new(100, vec![
            Pool::new(50, vec![
                Account::new_immediate(30, vec![Allocation::new(30, HumanAddr::from(""))]),
                Account::new_immediate(20, vec![Allocation::new(20, HumanAddr::from(""))])]),
            Pool::new(50, vec![
                Account::new_immediate(30, vec![Allocation::new(30, HumanAddr::from(""))]),
                Account::new_immediate(20, vec![Allocation::new(20, HumanAddr::from(""))])])]).claimable(&HumanAddr::from(""), 0),
        100);
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    pub total:    Uint128,
    pub accounts: Vec<Account>,
}
impl Pool {
    pub fn new (total: u128, accounts: Vec<Account>) -> Self {
        Self { total: Uint128::from(total), accounts }
    }
    pub fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for account in self.accounts.iter() {
            match account.validate() {
                Ok(_)  => { total += account.amount.u128() },
                Err(e) => return Err(e)
            }
        }
        if total != self.total.u128() {
            return Err(StdError::GenericErr {
                backtrace: None,
                msg: format!("pool's accounts add up to {}, expected {}", total, self.total)
            })
        }
        Ok(())
    }
}
#[test]
fn test_pool () {
    assert_eq!(Pool::new(0, vec![]).validate(), Ok(()));
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub amount:     Uint128,
    pub vesting:    Vesting,
    pub recipients: Vec<Allocation>,
}
impl Account {
    pub fn new (
        amount: u128,
        vesting: Vesting,
        recipients: Vec<Allocation>
    ) -> Self {
        Self { amount: Uint128::from(amount), vesting, recipients }
    }
    pub fn new_immediate (
        a: u128,
        r: Vec<Allocation>
    ) -> Self {
        let v = Vesting::Immediate {};
        Account::new(a, v, r) 
    }
    pub fn new_periodic (
        a: u128,
        r: Vec<Allocation>,
        interval: Interval,
        start_at: Seconds,
        duration: Seconds,
        cliff:    Percentage
    ) -> Self {
        let v = Vesting::Periodic {interval, start_at, duration, cliff};
        Account::new(a, v, r)
    }
    pub fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        for Allocation { addr, amount } in self.recipients.iter() {
            total += amount.u128()
        }
        if total != self.amount.u128() {
            return Err(StdError::GenericErr {
                backtrace: None,
                msg: format!("account's allocations add up to {}, expected {}", total, self.amount)
            })
        }
        Ok(())
    }
    pub fn claimable (&self, a: &HumanAddr, t: Seconds) -> u128 {
        let mut claimable = 0;
        for Allocation { addr, amount } in self.recipients.iter() {
            if addr == a {
                claimable += self.vest((*amount).u128(), t)
            }
        }
        return claimable
    }
    fn vest (&self, amount: u128, t: Seconds) -> u128 {
        match &self.vesting {
            // Immediate vesting: if the contract has launched,
            // the recipient can claim the entire allocated amount
            Vesting::Immediate {} => amount,

            // Periodic vesting: need to calculate the maximum amount
            // that the user can claim at the given time.
            Vesting::Periodic { interval, start_at, duration, cliff } => {
                let interval = match interval {
                    Interval::Daily   => DAY,
                    Interval::Monthly => MONTH
                };
                // Can't vest before the cliff
                if t < *start_at { return 0 }
                periodic(amount, interval, t - start_at, *duration, *cliff)
            }
        }
    }
}
#[test]
fn test_account () {
    assert_eq!(Account::new_immediate(0, vec![]).validate(),
        Ok(()));

    assert_eq!(Account::new_immediate(100, vec![
        Allocation::new(40, HumanAddr::from("Alice")),
        Allocation::new(60, HumanAddr::from("Bob"))
    ]).claimable(&HumanAddr::from("Alice"), 0),
        40);

    assert_eq!(Account::new_periodic(100, vec![
        Allocation::new(40, HumanAddr::from("Alice")),
        Allocation::new(60, HumanAddr::from("Bob"))
    ], Interval::Daily, 1, DAY, 0).claimable(&HumanAddr::from("Alice"), 0),
        0);

    assert_eq!(Account::new_periodic(100, vec![
        Allocation::new(40, HumanAddr::from("Alice")),
        Allocation::new(60, HumanAddr::from("Bob"))
    ], Interval::Daily, 1, DAY, 0).claimable(&HumanAddr::from("Alice"), 1),
        40);
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    amount: Uint128,
    addr:   HumanAddr,
}
impl Allocation {
    pub fn new (amount: u128, addr: HumanAddr) -> Self {
        Self { amount: Uint128::from(amount), addr }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Vesting {
    Immediate {},
    Periodic {
        interval: Interval,
        start_at: Seconds,
        duration: Seconds,
        cliff:    Percentage
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Interval {
    Daily,
    Monthly
}

fn periodic (
    amount:   u128,
    interval: Seconds,
    elapsed:  Seconds,
    duration: Seconds,
    cliff:    Percentage,
) -> u128 {

    // mutable for clarity:
    let mut vest = 0;

    // start with the cliff amount
    let cliff = cliff as u128;
    if cliff * amount % 100 > 0 { warn_cliff_remainder() }
    let cliff_amount = (cliff * amount / 100) as u128;
    vest += cliff_amount;

    // then for every `interval` since `t_start`
    // add an equal portion of the remaining amount

    // then, from the remaining amount and the number of vestings
    // determine the size of the portion
    let post_cliff_amount = amount - cliff_amount;
    let n_total: u128 = (duration / interval).into();
    if post_cliff_amount % n_total > 0 { warn_vesting_remainder() }
    let portion = post_cliff_amount / n_total;

    // then determine how many vesting periods have elapsed,
    // up to the maximum; `duration - interval` and `1 + n_elapsed`
    // are used to ensure vesting happens at the begginning of an interval
    let t_elapsed = Seconds::min(elapsed, duration - interval);
    let n_elapsed = t_elapsed / interval;
    let n_elapsed: u128 = (1 + n_elapsed).into();
    //if t_elapsed % interval > interval / 2 { n_elapsed += 1; }

    // then add that amount to the cliff amount
    vest += portion * n_elapsed;

    //println!("periodic {}/{}={} -> {}", n_elapsed, n_total, n_elapsed/n_total, vest);
    vest
}

fn warn_cliff_remainder () {
    //println!("WARNING: division with remainder for cliff amount")
}

fn warn_vesting_remainder () {
    //println!("WARNING: division with remainder for vesting amount")
}

#[test]
fn test_periodic () {
    assert_eq!(periodic( 0, 1, 0, 1, 0),  0);
    assert_eq!(periodic( 1, 1, 0, 1, 0),  1);
    assert_eq!(periodic(15, 1, 0, 3, 0),  5);
    assert_eq!(periodic(15, 1, 1, 3, 0), 10);
    assert_eq!(periodic(15, 1, 2, 3, 0), 15);
    assert_eq!(periodic(15, 1, 3, 3, 0), 15);
    assert_eq!(periodic(15, 1, 4, 3, 0), 15);
}
