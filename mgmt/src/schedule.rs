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
    amount:         Amount,
    addr:           cosmwasm_std::CanonicalAddr,
    cliff_months:   CliffMonths,
    cliff_percent:  CliffPercent,
    release_type:   ReleaseMode,
    release_months: ReleaseMonths,
}

/// This is needed to import the schedule from JSON during compilation.
const SCHEDULE_SRC: &str = include_str!("../../schedule/schedule.yml");
lazy_static::lazy_static! {
    static ref SCHEDULE: serde_yaml::Mapping =
        serde_yaml::from_str(&SCHEDULE_SRC).unwrap();
}

const DAY:   Time = 24*60*60;
const MONTH: Time = 30*DAY;

/// Distil the schedule into a single value.
pub fn slope_at (
    l: Time,
    t: Time,
    a: cosmwasm_std::CanonicalAddr
) -> Option<Amount> {
    match SCHEDULE.get(a) {
        None => None,
        Some(s) => if t > l + s.cliff_months * MONTH {
            Some(match s.release_type {
                ReleaseMode::Immediate    => s.amount,
                ReleaseMode::Daily        => s.amount,
                ReleaseMode::Monthly      => s.amount,
                ReleaseMode::Configurable => s.amount
            })
        } else {
            None
        }
    }
}
