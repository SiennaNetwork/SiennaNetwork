//! # Schedule
//!
//! This crate describes a vesting `Schedule`, split into `Pools` that
//! consist of `Accounts`, which may be immediate or `Periodic`, and their
//! output may be split between multiple addresses in accordance with given
//! `AllocationSet`s.
//!
//! The main thing a `Schedule` does is generate a list of `Portions`;
//! a smart contract can take that list and use it as a blueprint for the
//! funds that are unlocked for users to claim:
//!
//! ```rust
//! let amount = Uint128::from(100);
//! let address = HumanAddr::from("someone");
//! assert_eq!(
//!     Schedule {
//!         total: amount,
//!         pools: vec![
//!             Pool {
//!                 name: "P1",
//!                 total: amount,
//!                 partial: false,
//!                 channels: vec![
//!                     Channel {
//!                         name: "C1",
//!                         amount,
//!                         allocations: vec![
//!                             AllocationSet {
//!                                 t: 0,
//!                                 cliff: vec![],
//!                                 regular: vec![
//!                                     Allocation {
//!                                         amount,
//!                                         address
//!                                     }
//!                                 ]
//!                             }
//!                         ]
//!                     }
//!                 ]
//!             }
//!         ]
//!     }.all(),
//!     vec![
//!         Portion {
//!             amount: Uint128::from(100),
//!             address: HumanAddr
//!         }
//!     ]
//! )
//! ```
//!
//! ## 4-tier validation
//!
//! 1. The schema, representing the vesting schedule in terms of the structs
//!    defined by this crate. This is deserialized from a static input;
//!    any deviations from the schema cause the input to be rejected.
//!
//! 2. The `validate` module, which checks that sums don't exceed totals.
//!
//! 3. The runtime assertions in the `vesting`, which prevent
//!    invalid configurations from generating output.
//!
//! 4. For a running contract, valid outputs are further filtered by the
//!    `reconfig` module, which rejects configurations that change already
//!    vested/claimed portions.

/// error result constructor
macro_rules! Error {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr { msg: $msg.to_string(), backtrace: None })
    };
}

/// define error conditions with corresponding parameterized messages
macro_rules! define_errors {
    ($(
        $name:ident ($(&$self:ident,)? $($arg:ident : $type:ty),*) ->
        ($format:literal $(, $var:expr)*)
    )+) => {
        $(
            #[doc=$format]
            pub fn $name<T> ($(&$self,)? $($arg : $type),*) -> StdResult<T> {
                Error!(format!($format $(, $var)*))
            }
        )+
    }
}
/// alias for the most basic return type that may contain an error
pub type UsuallyOk = StdResult<()>;

pub mod units; pub use units::*;
pub mod validate; pub use validate::*;
pub mod vesting; pub use vesting::*;
pub mod reconfig; pub use reconfig::*;
#[cfg(test)] mod tests;

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

/// Vesting schedule; contains `Pool`s that must add up to `total`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:   Uint128,
    pub pools:   Vec<Pool>,
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
impl Pool {
    fn channels_total (&self) -> StdResult<u128> {
        let mut total = 0u128;
        for channel in self.channels.iter() {
            match channel.validate() {
                Ok(_)  => { total += channel.amount.u128() },
                Err(e) => return Err(e)
            }
        }
        Ok(total)
    }
}

/// Portions generator: can be immediate or `Periodic`; contains `Allocation`s (maybe partial).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    pub name:   String,
    pub amount: Uint128,

    /// Each portion can be split between multiple addresses.
    /// The full history of reallocations is stored here.
    pub allocations: Vec<AllocationSet>,

    /// This is an `Option` instead of `Channel` being an `Enum` because
    /// `serde_json_wasm` doesn't support non-C-style enums.
    ///
    /// `None` -> immediate vesting at launch:
    /// the recipient can claim the entire allocated amount
    /// once (after the contract has been launched).
    ///
    /// `Some(Periodic{..})` -> Periodic vesting:
    /// amount is unlocked in portions
    /// and claims transfer only the portions unlocked so far
    pub periodic: Option<Periodic>
}

/// Configuration of periodic vesting ladder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Periodic {
    pub interval:           Seconds,
    pub start_at:           Seconds,
    pub duration:           Seconds,
    pub cliff:              Uint128,
    pub expected_portion:   Uint128,
    pub expected_remainder: Uint128
}

/// Each Portion can be distributed among multiple addresses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllocationSet {
    t:         Seconds,
    cliff:     Allocations,
    regular:   Allocations,
    remainder: Allocations,
}
impl AllocationSet {
    fn portions (a: &Allocations, t: Seconds, r: &str) -> Portions {
        a.iter().map(|b|b.to_portion(t, r)).collect::<Vec<_>>()
    }
    fn sum (a: &Allocations) -> u128 {
        let mut sum = 0u128;
        for Allocation{amount,..} in a.iter() {
            sum+= amount.u128();
        }
        sum
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    amount:  Uint128,
    address: HumanAddr,
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
pub type Allocations = Vec<Allocation>;

/// Claimable portion
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Portion {
    pub amount:  Uint128,
    pub address: HumanAddr,
    pub vested:  Seconds,
    pub reason:  String
}
impl std::fmt::Display for Portion {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{} {} to {} at {}>", self.reason, self.amount, self.address, self.vested)
    }
}
pub type Portions                 = Vec<Portion>;
pub type PortionsWithTotal        = (Portions, u128);
pub type UsuallyPortions          = StdResult<Portions>;
pub type UsuallyPortionsWithTotal = StdResult<PortionsWithTotal>;
pub type PerhapsPortionsWithTotal = StdResult<Option<PortionsWithTotal>>;
