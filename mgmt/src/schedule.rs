use crate::types::*;
use crate::strings::{warn_div_cliff, warn_div_vesting};
use cosmwasm_std::HumanAddr;

/// This is needed to import the schedule from JSON during compilation.
const SRC: &str = include_str!("schedule.yml");
lazy_static! {
    static ref SCHEDULE: Schedule = serde_yaml::from_str(&SRC).unwrap();
}

const DAY:   Seconds = 24*60*60;
const MONTH: Seconds = 30*DAY;

/// Distil the value in question from the schedule.

pub fn claimable (
    recipient: &HumanAddr,
    launched:  Seconds,
    now:       Seconds,
) -> Amount {
    for s in SCHEDULE.preconfigured.iter() {
        match s {
            Stream::Immediate { amount, addr } => {
                if addr == recipient {
                    return immediate(*amount, launched, now)
                }
            },
            Stream::Monthly {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                if addr == recipient {
                    return periodic(
                        *amount, MONTH, launched, now,
                        *release_months, *cliff_months, *cliff_percent,
                    )
                }
            },
            Stream::Daily {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                if addr == recipient {
                    return periodic(
                        *amount, DAY, launched, now,
                        *release_months, *cliff_months, *cliff_percent,
                    )
                }
            },
        }
    }
    0
}

fn immediate (
    amount: Amount,
    launched: Seconds, now: Seconds,
) -> Amount {
    if now >= launched {
        amount
    } else {
        0
    }
}

fn periodic (
    amount: Amount, interval: Seconds,
    launched: Seconds, now: Seconds,
    release_months: Months, cliff_months: Months, cliff_percent: Percentage,
) -> Amount {
    let t_start = launched + cliff_months * MONTH;
    if now >= t_start {
        let t_end = t_start + release_months as u64 * MONTH;
        if now > t_end {
            0
        } else {
            let c = cliff_percent as u64;
            if c * amount % 100 > 0 { warn_div_cliff() }
            let cliff_amount  = c * amount / 100;
            let (t_elapsed, t_total) = (
                (  now - t_start) / interval,
                (t_end - t_start) / interval
            );
            if amount % t_total > 0 { warn_div_vesting() }
            cliff_amount + amount * t_elapsed / t_total
        }
    } else {
        0
    }
}

pub fn claimed (
    a:      &Address,
    claims: &FulfilledClaims,
    t:      Seconds
) -> Amount {
    for (addr, time, amount) in claims.iter().rev() {
       if addr != a { continue }
       if *time > t { continue }
       return *amount
    }
    0
}
