use crate::types::*;
use crate::strings::{warn_cliff_remainder, warn_vesting_remainder};
use cosmwasm_std::{HumanAddr, CanonicalAddr};

use crate::schedule::{SCHEDULE, DAY, MONTH};

/// Imports the schedule from JSON during compilation.
//const SRC: &str = include_str!("schedule.yml");
//lazy_static! {
    //pub static ref SCHEDULE: Schedule = serde_yaml::from_str(&SRC).unwrap();
//}

/// Determine how much an account has claimed
/// based on the history of fulfilled claims.
pub fn claimed (
    a:      &Address,
    claims: &FulfilledClaims,
    t:      Seconds
) -> Amount {
    for (addr, time, amount) in claims.iter().rev() {
       if addr != a { continue }
       if *time > t { continue }
       return amount.u128()
    }
    0
}

/// Determine how much one can claim
/// based on the predefined schedule
/// and the configurable allocation.
pub fn claimable (
    recipient:       &HumanAddr,
    recipient_canon: &CanonicalAddr,
    recipients:      &Allocation,
    launched:        Seconds,
    now:             Seconds,
) -> Amount {
    // preconfigured claimants:
    for s in SCHEDULE.predefined.iter() {
        match s {
            Stream::Immediate { amount, addr } => {
                if addr == recipient {
                    return if now >= launched {
                        amount.u128()
                    } else {
                        0
                    }
                }
            },
            Stream::Monthly {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                if addr == recipient {
                    return periodic(
                        amount.u128(), MONTH, launched, now,
                        *release_months, *cliff_months, *cliff_percent,
                    )
                }
            },
            Stream::Daily {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                if addr == recipient {
                    return periodic(
                        amount.u128(), DAY, launched, now,
                        *release_months, *cliff_months, *cliff_percent,
                    )
                }
            },
        }
    }
    // configurable claimants:
    for (addr, amount) in recipients {
        if addr == recipient_canon {
            let days_since_launch = (now - launched) / DAY;
            return amount.u128() * (days_since_launch + 1) as u128;
        }
    }
    // default case:
    0
}

/// Calculate how much the user
/// can claim at the given time.
fn periodic (
    amount: Amount, interval: Seconds,
    launched: Seconds, now: Seconds,
    release_months: Months, cliff_months: Months, cliff_percent: Percentage,
) -> Amount {
    let t_start = launched + cliff_months * MONTH;
    if now >= t_start {
        let t_end = t_start + release_months * MONTH;
        let c: u128 = cliff_percent.into();
        if c * amount % 100 > 0 { warn_cliff_remainder() }
        let cliff_amount  = (c * amount / 100) as u128;
        let (t_elapsed, t_total) = (
            (  now - t_start) / interval,
            (t_end - t_start) / interval
        );
        if amount % (t_total as u128) > 0 { warn_vesting_remainder() }
        Amount::min(
            amount,
            cliff_amount + amount * (t_elapsed / t_total) as u128
        )
    } else {
        0
    }
}
