/// # SIENNA/Hack.bg Schedule v2.0
///
/// ## Conceptual model
/// * `Schedule`: the root object.
///   * Has a `total`.
///   * Contains `Pool`s adding up to that total.
/// * `Pool`: subdivision of schedule,
///   * Contains `Account`s.
///   * If `partial` is true, `Account`s can be at runtime, up to the total.
///   * Otherwise, requires `Account`s to add up to exactly the total in order to pass validation.
/// * `Account`: subdivision of a `Pool` (corresponds to `Channel`+`Periodic` from v1)

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
//use snafu::GenerateBacktrace;
pub use cosmwasm_std::{Uint128, HumanAddr, StdResult, StdError};

pub mod errors; pub use errors::*;
pub mod validate;
pub mod vesting;
pub mod mutate;

/// Unit of time
pub type Seconds = u64;

/// Unit of account
pub const ONE_SIENNA: u128 = 1000000000000000000u128;

/// The most basic return type that may contain an error
pub type UsuallyOk = StdResult<()>;

/// Contains `Pool`s that must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
}
impl Schedule {
    pub fn new (pools: &[Pool]) -> Self {
        let mut s = Schedule { total: Uint128::zero(), pools: pools.to_vec() };
        s.total = Uint128::from(s.subtotal());
        s
    }
    /// Sum of all contained pools (expected to equal `self.total`)
    pub fn subtotal (&self) -> u128 {
        self.pools.iter().fold(0, |total, pool| total + pool.total.u128())
    }
}

/// Subdivision of `Schedule`, contains `Account`s, may be `partial`.
/// If `partial == false`, they must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    /// if `true`, adding new `Account`s is allowed at runtime, up to `total`.
    /// otherwise, accounts must add up to `total` at creation.
    pub partial:  bool,
    pub name:     String,
    pub total:    Uint128,
    pub accounts: Vec<Account>,
}
impl Pool {
    pub fn partial (name: &str, total: u128, accounts: &[Account]) -> Self {
        let accounts = accounts.to_vec();
        Pool { partial: true, name: name.into(), total: total.into(), accounts }
    }
    pub fn full (name: &str, accounts: &[Account]) -> Self {
        let accounts = accounts.to_vec();
        let mut total = Uint128::zero();
        for &Account{amount,..} in accounts.iter() { total += amount }
        Pool { partial: false, name: name.into(), total, accounts }
    }
    /// Sum of all contained accounts - expected to equal total
    pub fn subtotal (&self) -> u128 {
        self.accounts.iter().fold(0, |total, acc| total + acc.amount.u128())
    }
    /// Remaining unallocated funds
    pub fn unallocated (&self) -> u128 {
        self.total.u128() - self.subtotal()
    }
}

/// Individual vesting config.
/// Immediate release is thought of as a special case of vesting where:
/// * `head == total`
/// * `duration == interval == 0`,
/// * only `head_allocations` is considered.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    /// Human-readable name
    pub name:   String,
    /// Recipient address
    pub address: HumanAddr,
    /// Funds that this account will release
    pub amount: Uint128,
    /// If `> 0`, releases this much money the first time, pushing back the regular portions
    pub cliff: Uint128,
    /// How many seconds after contract launch to begin vesting
    pub start_at: Seconds,
    /// How many seconds to wait between portions
    pub interval: Seconds,
    /// If `> 0`, vesting stops after this much seconds regardless of how much is left of `total`.
    pub duration: Seconds,
}
impl Account {
    pub fn immediate (name: &str, address: &HumanAddr, amount: u128) -> Self {
        Self {
            name:     name.into(),
            address:  address.clone(),
            amount:   amount.into(),
            cliff:    0u128.into(),
            start_at: 0,
            interval: 0,
            duration: 0
        }
    }
    pub fn periodic (
        name: &str, address: &HumanAddr, amount: u128,
        cliff: u128, start_at: Seconds, interval: Seconds, duration: Seconds
    ) -> Self {
        Self {
            name:    name.into(),
            address: address.clone(),
            amount:  amount.into(),
            cliff:   cliff.into(),
            start_at,
            interval,
            duration
        }
    }
}
