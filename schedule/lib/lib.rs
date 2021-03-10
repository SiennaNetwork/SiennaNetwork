/// # SIENNA/Hack.bg Schedule v2.0
///
/// ## Concept model
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
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use snafu::GenerateBacktrace;
pub mod macros;
pub mod units; pub use units::*;
pub mod validate; pub use validate::*;
pub mod vesting; pub use vesting::*;
pub mod history; pub use history::*;
#[cfg(test)] mod tests;
/// alias for the most basic return type that may contain an error
pub type UsuallyOk = StdResult<()>;
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
        write!(f, "T={:.>10} \"{}\" {:.>18} to {}", self.vested, self.reason, self.amount.u128(), self.address)
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
