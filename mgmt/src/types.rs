use cosmwasm_std::{Uint128, CanonicalAddr, HumanAddr};

/// Basic quantities.
pub type Seconds    = u64;
pub type Days       = u64;
pub type Months     = u64;
pub type Percentage = u64;
pub type Amount     = u128;
pub type Address    = CanonicalAddr;

/// Creator of contract.
pub type Admin = Address;

/// The token contract that will be controlled.
pub type TokenAddress = HumanAddr;

/// A contract's code hash
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// TODO: Public hit counter. ;)
pub type ErrorCount = u64;

/// Log of executed claims
pub type FulfilledClaims = Vec<(Address, Seconds, Uint128)>;

/// Configurable recipients.
/// TODO what happens if this is empty?
pub type Allocation = Vec<(Address, Uint128)>;

/// Schedule: predefined vesting streams + how much is configurable on the fly.
/// TODO validate that predefined + configurable == total, maybe in Init?
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    /// Total amount to be minted on contract deploy
    pub total:              Uint128,

    /// Total amount that can be reconfigured after launch
    pub configurable:       Uint128,

    /// Daily amount that can be reconfigured after launch
    pub configurable_daily: Uint128,

    /// Predefined vesting streams
    pub predefined:    Vec<Stream>,
}

/// A predefined vesting stream from `schedule.yml`
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "release_mode")]
pub enum Stream {
    Immediate {
        amount: Uint128,
        addr:   HumanAddr
    },
    Daily {
        amount:         Uint128,
        addr:           HumanAddr,
        release_months: Months,
        cliff_months:   Months,
        cliff_percent:  Percentage
    },
    Monthly {
        amount:         Uint128,
        addr:           HumanAddr,
        release_months: Months,
        cliff_months:   Months,
        cliff_percent:  Percentage
    },
}
