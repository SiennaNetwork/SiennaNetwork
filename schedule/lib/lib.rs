//! * Lets you instantiate a `Schedule` in terms of `Pool`s of `Account`s.
//!     * `Pool`s allow `Account`s to be added in the future
//! * Lets you generate a flat list of transactions as specified by that
//!   schedule.
//!     * `Account`s can release their amounts immediately,
//!       or vest them, to one or several addresses.
//!         * Separate `head`, `body` and `tail` allocations allow for
//!           cliffs and remainders to be flexibly implemented
//! * `TODO` Lets you compare a partially-executed schedule with a new proposal,
//!   and determine if the alteration to the schedule is allowed
//!     * In strict mode, no vested portion is allowed to be canceled.
//!       (this makes the schedule append-only?)
//!     * In non-strict mode, no claimed portion is allowed to be canceled.
//!
//! ## How to use
//!
//! * `TODO` Use the executables in `bin` to ingest a `tsv`, `ods` or `json`
//!   file describing a schedule, and  generate a pair of `schedule.json`
//!   (what the parser understood) and `transactions.json` (what transactions
//!   were generated).
//!   * Load `transactions.json` into `mgmt` to update the schedule
//!     * `TODO` historical validation should be executed in the contract
//!   * Load `schedule.json` + `transactions.json` into `gov` to let users
//!     vote on amendments to the schedule.
//! * The `schedule!` macro, documented in `macros`, exists to allow schedules
//!   to be hardcoded using a terse syntax.
//! * Earlier versions of this crate were executed on-chain. This should still
//!   be possible, although not recommended.

pub mod macros;
pub mod units; pub use units::*;
pub mod validate; pub use validate::*;
pub mod vesting; pub use vesting::*;
pub mod history; pub use history::*;
#[cfg(test)] mod tests;

/// alias for the most basic return type that may contain an error
pub type UsuallyOk = StdResult<()>;

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

/// Vesting schedule; contains `Pool`s that must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
}

/// Vesting pool; contains `Account`s that must add up to `total`
/// if `partial == false`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    pub name:     String,
    pub total:    Uint128,
    pub partial:  bool,
    pub accounts: Vec<Account>,
}
impl Pool {
    fn accounts_total (&self) -> StdResult<u128> {
        let mut total = 0u128;
        for account in self.accounts.iter() {
            match account.validate() {
                Ok(_)  => { total += account.total.u128() },
                Err(e) => return Err(e)
            }
        }
        Ok(total)
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
    /// Funds that this account will release
    pub total: Uint128,
    /// If `> 0`, releases this much money the first time
    pub head: Uint128,
    /// Head can be portioned between multiple addresses
    pub head_allocations: Allocations,
    /// Size of regular portion - determines how many portions will be vested
    pub body_allocations: Allocations,
    /// Vested once after regular portions run out (TODO but not after `duration`?)
    pub tail_allocations: Allocations,

    /// How many seconds after contract launch to begin vesting
    pub start_at: Seconds,
    /// How many seconds to wait between portions
    pub interval: Seconds,
    /// If `> 0`, vesting stops after this much seconds
    /// regardless of how much is left of the `total`.
    pub duration: Seconds,
}

/// Each Portion can be distributed among multiple addresses.
pub type Allocations = Vec<Allocation>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    pub amount:  Uint128,
    pub address: HumanAddr,
}
impl Allocation {
    pub fn to_portion (&self, vested: Seconds, reason: &str) -> Portion {
        Portion {
            amount:  self.amount,
            address: self.address.clone(),
            vested,
            reason: reason.to_string()
        }
    }
}
pub fn allocations_to_portions (a: &Allocations, t: Seconds, r: &str) -> Portions {
    a.iter().map(|b|b.to_portion(t, r)).collect::<Vec<_>>()
}
pub fn sum_allocations (a: &Allocations) -> u128 {
    let mut sum = 0u128;
    for Allocation{amount,..} in a.iter() {
        sum+= amount.u128();
    }
    sum
}

/// Claimable portion
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Portion {
    pub vested:  Seconds,
    pub address: HumanAddr,
    pub amount:  Uint128,
    pub reason:  String
}
impl std::fmt::Display for Portion {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{} {} to {} at {}>", self.reason, self.amount, self.address, self.vested)
    }
}

/// list of `Portion`s
pub type Portions                 = Vec<Portion>;

/// list of `Portion`s with expected total (for caller to check)
pub type PortionsWithTotal        = (Portions, u128);

/// list of `Portion`s, or error
pub type UsuallyPortions          = StdResult<Portions>;

/// list of `Portion`s with total, or error
pub type UsuallyPortionsWithTotal = StdResult<PortionsWithTotal>;

/// list of `Portion`s with total, `None`, or error (used by `vest_head`/`vest_tail`)
pub type PerhapsPortionsWithTotal = StdResult<Option<PortionsWithTotal>>;
