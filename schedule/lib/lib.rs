//! The main thing this create does is generate a list of `Portions`;
//! a smart contract can take that list and use it as a blueprint for the
//! funds that are unlocked for users to claim.
//!
//! # Data model
//!
//! The `Schedule`:
//! * has a `total`
//! * has one or more `Pool`s which must add up to `total`, each of which
//!     * has a `total`
//!     * has one or more `Account`s, each of which
//!         * has one or more `AllocationSet`s
//!         * can be either
//!             * __immediate__ (`periodic: None`)
//!                 * which means the funds are released immediately
//!                 * in which case the associated `AllocationSet`s must
//!                   not contain `head` or `tail` allocations.
//!             * or __periodic__ (`periodic: Some(Periodic{..})`)
//!                 * which means that it consists of
//!                     * an optional `head`
//!                     * one or more `body` portions
//!                     * a `tail`
//!                 * and that their `AllocationSet`s must contain
//!                   `head`, `body` and `tail` allocations
//!                   that add up to the correct amount
//!
//! `serde_json_wasm` (used internally by CosmWasm) does not support advanced
//! Rust `enum`s; were it to support them:
//! * `Account`, `Periodic`, and `AllocationSet` could be merged into
//!   a single enum with two variants.
//! * the extra validation logic that prevents invalid combinations between the
//!   three could be removed.
//!
//! # How to use
//!
//! * Normally, a `Schedule` is built from a spreadsheet or deserialized from
//!   a JSON file.
//! * However, the `schedule!` macro, documented in `macros`, exists to allow
//!   `Schedule`s to be defined in reasonably terse Rust code.

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
