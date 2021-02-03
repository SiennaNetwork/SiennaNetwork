use crate::types::*;
use cosmwasm_std::{HumanAddr, Uint128};

pub const DAY:   Seconds = 24*60*60;
pub const MONTH: Seconds = 30*DAY;

lazy_static! {
    pub static ref SCHEDULE: Schedule = Schedule {
        total: Uint128::from(10000000000000000000000000u128),
        configurable: Uint128::from(300000000000000000000000u128),
        configurable_daily: Uint128::from(2500000000000000000000u128),
        predefined: vec! [
            Stream::Monthly {
                amount: Uint128::from(1500000000000000000000000u128),
                addr: HumanAddr::from("DevFund"),
                release_months: 20,
                cliff_months: 4,
                cliff_percent: 5
            },
            Stream::Daily {
                amount: Uint128::from(2000000000000000000000000u128),
                addr: HumanAddr::from("Investors"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 0
            },
            Stream::Daily {
                amount: Uint128::from(897000000000000000000000u128),
                addr: HumanAddr::from("Founder1"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 10
            },
            Stream::Daily {
                amount: Uint128::from(897000000000000000000000u128),
                addr: HumanAddr::from("Founder2"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 10
            },
            Stream::Daily {
                amount: Uint128::from(437000000000000000000000u128),
                addr: HumanAddr::from("Founder3"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 10
            },
            Stream::Daily {
                amount: Uint128::from(69000000000000000000000u128),
                addr: HumanAddr::from("Founder4"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 10
            },
            Stream::Daily {
                amount: Uint128::from(50000000000000000000000u128),
                addr: HumanAddr::from("Advisor1"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 0
            },
            Stream::Daily {
                amount: Uint128::from(50000000000000000000000u128),
                addr: HumanAddr::from("Advisor2"),
                release_months: 16,
                cliff_months: 6,
                cliff_percent: 0
            },
            Stream::Daily {
                amount: Uint128::from(10000000000000000000000u128),
                addr: HumanAddr::from("Advisor3"),
                release_months: 6,
                cliff_months: 6,
                cliff_percent: 0
            },
            Stream::Daily {
                amount: Uint128::from(5000000000000000000000u128),
                addr: HumanAddr::from("Advisor4"),
                release_months: 6,
                cliff_months: 6,
                cliff_percent: 0
            },
            Stream::Immediate {
                amount: Uint128::from(85000000000000000000000u128),
                addr: HumanAddr::from("AdvisorR")
            },
            Stream::Immediate {
                amount: Uint128::from(3700000000000000000000000u128),
                addr: HumanAddr::from("Remaining")
            }
        ]
    };
}
