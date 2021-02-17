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
        $(pub fn $name<T> ($(&$self,)? $($arg : $type),*) -> StdResult<T> {
            Error!(format!($format $(, $var)*))
        })+
    }
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
pub fn pool (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: false }
}
pub fn pool_partial (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: true }
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
pub fn channel_immediate (
    amount: u128,
    address: &HumanAddr
) -> Channel {
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: None,
        allocations: vec![(0, vec![allocation(amount, address)])],
    }
}
pub fn channel_immediate_multi (
    _amount: u128,
    _allocations: &Vec<Allocation>
) -> Channel {
    panic!("immediate vesting with multiple recipients is not supported")
}
pub fn channel_periodic (
    amount:   u128,
    address:  &HumanAddr,
    interval: Seconds,
    start_at: Seconds,
    duration: Seconds,
    cliff:    u128
) -> StdResult<Channel> {
    let mut channel = Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(periodic_validated(amount, start_at, cliff, duration, interval)?),
        allocations: vec![]
    };
    let portion = channel.portion_size()?;
    channel.allocations.push((0, vec![allocation(portion, address)]));
    Ok(channel)
}
pub fn channel_periodic_multi (
    amount:      u128,
    allocations: &Vec<Allocation>,
    interval:    Seconds,
    start_at:    Seconds,
    duration:    Seconds,
    cliff:       u128
) -> Channel {
    if cliff > 0 { panic!("periodic vesting with cliff and multiple recipients is not supported") }
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(periodic_validated(amount, start_at, cliff, duration, interval).unwrap()),
        allocations: vec![(0, allocations.clone())]
    }
}

/// Configuration of periodic vesting ladder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Periodic {
    pub interval:  Seconds,
    pub start_at:  Seconds,
    pub duration:  Seconds,
    pub cliff:              Uint128,
    pub expected_portion:   Uint128,
    pub expected_remainder: Uint128
}
pub fn periodic (
    start_at: Seconds,
    cliff:    u128,
    duration: Seconds,
    interval: Seconds
) -> Periodic {
    Periodic {
        interval, start_at, duration, cliff: Uint128::from(cliff),
        expected_portion:   Uint128::zero(),
        expected_remainder: Uint128::zero()
    }
}
pub fn periodic_validated (
    amount:   u128,
    start_at: Seconds,
    cliff:    u128,
    duration: Seconds,
    interval: Seconds
) -> StdResult<Periodic> {
    let mut p = Periodic {
        interval, start_at, duration, cliff: Uint128::from(cliff),
        expected_portion:   Uint128::zero(),
        expected_remainder: Uint128::zero()
    };
    let portion = p.portion_size("", amount)?;
    let n_portions = p.portion_count("")?;
    p.expected_portion = Uint128::from(portion);

    let mut remainder = amount;
    remainder -= cliff;
    remainder -= portion * n_portions as u128;
    p.expected_remainder = Uint128::from(remainder);

    Ok(p)
}

/// Allocation of vesting to multiple addresses.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllocationSet {
    t:         Seconds,
    cliff:     Vec<Allocation>,
    regular:   Vec<Allocation>,
    remainder: Vec<Allocation>,
}
pub fn allocation_set (
    t:         Seconds,
    cliff:     Vec<Allocation>,
    regular:   Vec<Allocation>,
    remainder: Vec<Allocation>
) -> AllocationSet {
    AllocationSet { t, cliff, regular, remainder }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    amount: Uint128,
    addr:   HumanAddr,
}
pub fn allocation (amount: u128, addr: &HumanAddr) -> Allocation {
    Allocation { amount: Uint128::from(amount), addr: addr.clone() }
}

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
pub fn portion (amt: u128, addr: &HumanAddr, vested: Seconds, reason: &str) -> Portion {
    Portion {
        amount:  Uint128::from(amt),
        address: addr.clone(),
        vested:  vested,
        reason:  reason.to_string()
    }
}

/// Log of executed claims
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct History {
    pub history: Vec<ClaimedPortion>
}
impl History {
    pub fn new () -> Self { Self { history: vec![] } }
    /// Takes list of portions, returns the ones which aren't marked as claimed
    pub fn unclaimed (&mut self, claimable: Vec<Portion>) -> Vec<Portion> {
        // TODO sort by timestamp and validate that there is no overlap
        //      between claimed/unclaimed because that would signal an error
        let claimed_portions: Vec<Portion> =
            self.history.iter().map(|claimed| claimed.portion.clone()).collect();
        claimable.into_iter()
            .filter(|portion| !claimed_portions.contains(portion)).collect()
    }
    /// Marks a portion as claimed
    pub fn claim (&mut self, claimed: Seconds, portions: Vec<Portion>) {
        for portion in portions.iter() {
            self.history.push(ClaimedPortion {claimed, portion: portion.clone()} )
        }
    }
}

/// History entry
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimedPortion {
    portion: Portion,
    claimed: Seconds
}
