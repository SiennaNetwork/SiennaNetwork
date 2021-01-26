/// Basic quantities.
pub type Time    = u64;
pub type Amount  = u64;
pub type Address = cosmwasm_std::CanonicalAddr;

/// Creator of contract.
/// TODO make configurable
pub type Admin = Address;

/// The token contract that will be controlled.
/// TODO see how this can be generated for testing
pub type Token = Option<Address>;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Time>;

/// TODO: Public hit counter. ;)
pub type ErrorCount = u64;

/// Log of executed claims
pub type FulfilledClaims = Vec<(Address, Time, Amount)>;

/// Configurable recipients.
pub type ConfiguredRecipients = Vec<(Address, Amount)>;

/// Description of release modes
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseMode { Daily, Monthly, Immediate, Configurable }
pub type ReleaseMonths = u16;
pub type CliffMonths   = u16;
pub type CliffPercent  = u16;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub amount:         Amount,
    pub addr:           cosmwasm_std::HumanAddr,
    pub release_mode:   ReleaseMode,
    pub release_months: Option<ReleaseMonths>,
    pub cliff_months:   Option<CliffMonths>,
    pub cliff_percent:  Option<CliffPercent>,
}
