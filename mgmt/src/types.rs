pub use sienna_schedule::{
    Seconds, Days, Months, Percentage, Amount,
    Schedule, Pool, Account, Allocation, Vesting, Interval,
    FulfilledClaims,
};

/// A contract's code hash
pub type CodeHash = String;

/// Whether the vesting process has begun and when.
pub type Launched = Option<Seconds>;

/// Public counter of invalid operations.
pub type ErrorCount = u64;
