use serde::{Serialize, Deserialize};
use crate::types::*;

/// Description of release modes
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseMode { Daily, Monthly, Immediate, Configurable }
pub type ReleaseMonths = u16;
pub type CliffMonths   = u16;
pub type CliffPercent  = u16;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    pub amount:         Amount,
    pub addr:           cosmwasm_std::CanonicalAddr,
    pub cliff_months:   CliffMonths,
    pub cliff_percent:  CliffPercent,
    pub release_mode:   ReleaseMode,
    pub release_months: ReleaseMonths,
}

/// This is needed to import the schedule from JSON during compilation.
const SCHEDULE_SRC: &str = include_str!("../../schedule/schedule.yml");
lazy_static! {
    static ref SCHEDULES: Vec<Schedule> =
        serde_yaml::from_str(&SCHEDULE_SRC).unwrap();
}

const DAY:   Time = 24*60*60;
const MONTH: Time = 30*DAY;

/// Distil the value in question from the schedule.

pub fn at (
    a: &cosmwasm_std::CanonicalAddr,
    l: Time,
    t: Time,
) -> Amount {
    for s in SCHEDULES.iter() {
        if s.addr != *a { continue }
        let cliff_seconds = s.cliff_months as u64 * MONTH;
        return if t > l + cliff_seconds {
            match s.release_mode {
                ReleaseMode::Immediate    => s.amount,
                ReleaseMode::Daily        => s.amount,
                ReleaseMode::Monthly      => s.amount,
                ReleaseMode::Configurable => s.amount
            }
        } else {
            0
        }
    }
    0
}
