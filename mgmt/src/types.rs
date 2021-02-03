use cosmwasm_std::{Uint128, CanonicalAddr, HumanAddr};

/// Time
pub const DAY:   Seconds = 24*60*60;
pub const MONTH: Seconds = 30*DAY;

/// Money
pub const ONE_SIENNA: u128 = 1000000000000000000u128;

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
    pub predefined:     Vec<Stream>,
}

/// A predefined stream of transactions
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Stream {
    pub amount:  Uint128,
    pub addr:    HumanAddr,
    pub vesting: Vesting
}

/// The vesting schedule of an indiviudal stream
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Vesting {
    /// Release everything immediately
    Immediate {},

    /// After releasing `cliff_percent` at `cliff`,
    /// release every 24 hours for `duration` seconds
    Daily {
        start_at: Seconds,
        duration: Seconds,
        cliff:    Percentage
    },

    /// After releasing `cliff_percent` at `cliff`,
    /// release every 720 hours for `duration` seconds
    Monthly {
        start_at: Seconds,
        duration: Seconds,
        cliff:    Percentage
    }
}
