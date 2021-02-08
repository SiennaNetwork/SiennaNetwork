pub use sienna_schedule::{DAY, MONTH, ONE_SIENNA};
use sienna_schedule::Amount;

/// Default value for Secret Network block size
/// (according to Reuven on Discord)
pub const BLOCK_SIZE: usize = 256;

lazy_static! {
    pub static ref BROKEN:    &'static str = "broken";
    pub static ref NOTHING:   &'static str = "nothing for you";
    pub static ref UNDERWAY:  &'static str = "already underway";
    pub static ref PRELAUNCH: &'static str = "not launched yet";
}

pub fn err_allocation (total: Amount, max: Amount) -> String {
    format!("allocations added up to {} which is over the maximum of {}",
        total, max)
}
