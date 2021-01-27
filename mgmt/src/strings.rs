use crate::types::Amount;

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

pub fn warn_div_cliff () {
    println!("WARNING: division with remainder for cliff amount")
}

pub fn warn_div_vesting () {
    println!("WARNING: division with remainder for vesting amount")
}
