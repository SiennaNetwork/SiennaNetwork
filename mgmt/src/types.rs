/// Basic quantities.
pub type Seconds    = u64;
pub type Months     = u64;
pub type Amount     = u64;
pub type Percentage = u64;
pub type Address    = cosmwasm_std::CanonicalAddr;

/// Creator of contract.
pub type Admin = Address;

/// The token contract that will be controlled.
pub type Token = Option<Address>;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// TODO: Public hit counter. ;)
pub type ErrorCount = u64;

/// Log of executed claims
pub type FulfilledClaims = Vec<(Address, Seconds, Amount)>;

/// Configurable recipients.
pub type Allocation = Vec<(Address, Amount)>;

/// Description of release modes
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseMode { Daily, Monthly, Immediate, Configurable }
pub type ReleaseMonths = u64;
pub type CliffMonths   = u64;
pub type CliffPercent  = u64;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub total:         Amount,
    pub configurable:  Amount,
    pub preconfigured: Vec<Stream>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "release_mode")]
pub enum Stream {
    Immediate {
        amount: Amount,
        addr:   cosmwasm_std::HumanAddr
    },
    Daily {
        amount:         Amount,
        addr:           cosmwasm_std::HumanAddr,
        release_months: Months,
        cliff_months:   Months,
        cliff_percent:  Percentage
    },
    Monthly {
        amount:         Amount,
        addr:           cosmwasm_std::HumanAddr,
        release_months: Months,
        cliff_months:   Months,
        cliff_percent:  Percentage
    },
}
