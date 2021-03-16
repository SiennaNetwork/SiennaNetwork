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
///   * contains 3 sets of `Allocation`s:
///     * `head` for splitting the cliff.
///     * `body` for splitting the regular portions.
///     * `tail` for splitting the remainders.
///   * The above are added for completeness' sake; the currently planned schedule does not
///     require splitting `head/`tail`s, and only needs one instance of splitting `body` -
///     but it's easier and more future-proof to implement splitting as the general case
///     rather than a special case (which would rightfully belong in a separate contract otherwise)
///   * Generates `Portion`s from `Allocation`s.
/// * `Allocation`: pair of address and amount.
///   * `TODO`: establish constraints about allocation totals
/// * `Portion`: an `Allocation` with a `vested` date and a `reason`.
///   * `TODO`: `reason`s are few; convert to enum

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use snafu::GenerateBacktrace;

pub mod macros;
pub mod validate; pub use validate::*;
pub mod vesting; pub use vesting::*;
pub use cosmwasm_std::{Uint128, HumanAddr, StdResult, StdError};
#[cfg(test)] mod tests;

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

/// contains `Account`s; if `partial == false`, they must add up to `total`.
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
            account.validate()?;
            total += account.amount.u128();
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
    /// 1 if immediate, or `duration/interval` if periodic.
    /// Returns error if `duration` is not a multiple of `interval`.
    pub fn portion_count (&self) -> u128 {
        if self.interval == 0 {
            0
        } else {
            (self.amount.u128() - self.cliff.u128()) / self.interval as u128
        }
    }
    /// Full `amount` if immediate, or `(amount-cliff)/portion_count` if periodic.
    /// Returns error if amount can't be divided evenly in that number of portions.
    pub fn portion_size (&self) -> u128 {
        if self.interval == 0 {
            0
        } else {
            self.amount.u128() / self.portion_count()
        }
    }
}
