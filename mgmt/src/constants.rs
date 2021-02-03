use crate::types::{Seconds, Amount};

/// Time
pub const DAY:   Seconds = 24*60*60;
pub const MONTH: Seconds = 30*DAY;

/// Money
pub const ONE_SIENNA: u128 = 1000000000000000000u128;

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

pub fn warn_cliff_remainder () {
    //println!("WARNING: division with remainder for cliff amount")
}

pub fn warn_vesting_remainder () {
    //println!("WARNING: division with remainder for vesting amount")
}
