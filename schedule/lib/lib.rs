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
//!     * has one or more `Channel`s, each of which
//!         * has one or more `AllocationSet`s
//!         * can be either
//!             * __immediate__ (`periodic: None`)
//!                 * which means the funds are released immediately
//!                 * in which case the associated `AllocationSet`s must
//!                   not contain `cliff` or `remainder` allocations.
//!             * or __periodic__ (`periodic: Some(Periodic{..})`)
//!                 * which means that it consists of
//!                     * an optional `cliff`
//!                     * one or more `regular` portions
//!                     * a `remainder`
//!                 * and that their `AllocationSet`s must contain
//!                   `cliff`, `regular` and `remainder` allocations
//!                   that add up to the correct amount
//!
//! `serde_json_wasm` (used internally by CosmWasm) does not support advanced
//! Rust `enum`s; were it to support them:
//! * `Channel`, `Periodic`, and `AllocationSet` could be merged into
//!   a single enum with two variants.
//! * the extra validation logic that prevents invalid combinations between the
//!   three could be removed.
//!
//! # How to use
//!
//! Normally, a `Schedule` is deserialized from user input; however, the
//! `schedule!` macro exists to allow `Schedule`s to be defined in terse
//! Rust code. Below is such an example, demonstrating most features of
//! this crate; use `.all()` to get the resulting list of transcations.
//!
//! ```rust
//! let ALICE = HumanAddr::from("Alice");
//! let BOB   = HumanAddr::from("Bob");
//! let CANDY = HumanAddr::from("Candy");
//! let S = schedule!(300 => (
//!     P0(100) = (
//!         C00(50) = (
//!             ALICE => 50
//!         )
//!         C01(50) = (
//!             BOB   => 25
//!             CANDY => 25
//!         )
//!     )
//!     P1(200) = (
//!         C10(100) = (
//!             cliff(20 at 5) = (
//!                 ALICE => 12
//!                 BOB   =>  8
//!             )
//!             regular(30 every 30) = (
//!                 ALICE => 15
//!                 BOB   => 15
//!             ),
//!             remainder(20) = (
//!                 CANDY => 20
//!             )
//!         )
//!     )
//! ));
//! assert_eq!(S.all(), portions!(
//!     [  0  ALICE  50  "C00: immediate"]
//!     [  0  BOB    25  "C01: immediate"]
//!     [  0  CANDY  25  "C01: immediate"]
//!
//!     [  5  ALICE  12  "C10: cliff"    ]
//!     [  5  BOB     8  "C10: cliff"    ]
//!     [ 35  ALICE  15  "C10: vesting"  ]
//!     [ 35  BOB    15  "C10: vesting"  ]
//!     [ 65  ALICE  15  "C10: vesting"  ]
//!     [ 65  BOB    15  "C10: vesting"  ]
//!     [ 65  CANDY  20  "C10: remainder"]
//! ))
//! ```

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

/// create `Schedule` w/ (`Pool`s w/ (`Channel`s w/ `Periodic`s & (`AllocationSet`s w/ `Allocation`s)))
macro_rules! schedule {
    ($total:expr => ($(
        $pool:ident ( $pool_total:expr ) = ($(
            $channel:ident ( $channel_total:expr ) = ($(
                $addr:expr => $amount:expr
            )+)
        )+)
    )+)) => { Schedule { total: $total, pools: vec![$($pool),+] } };
}

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
    pub start_at:           Seconds,
    pub cliff:              Uint128,
    pub duration:           Seconds,
    pub interval:           Seconds,
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

/// list of `Allocation`s
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

/// list of `Portion`s
pub type Portions                 = Vec<Portion>;

/// list of `Portion`s with expected total (for caller to check)
pub type PortionsWithTotal        = (Portions, u128);

/// list of `Portion`s, or error
pub type UsuallyPortions          = StdResult<Portions>;

/// list of `Portion`s with total, or error
pub type UsuallyPortionsWithTotal = StdResult<PortionsWithTotal>;

/// list of `Portion`s with total, `None`, or error (used by `vest_cliff`/`vest_remainder`)
pub type PerhapsPortionsWithTotal = StdResult<Option<PortionsWithTotal>>;
