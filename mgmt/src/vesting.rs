use cosmwasm_std::{HumanAddr, CanonicalAddr};

use crate::schedule::SCHEDULE;
use crate::constants::{
    DAY, MONTH,
    warn_cliff_remainder, warn_vesting_remainder
};
use crate::types::{
    Seconds,
    Address, Amount, Percentage,
    Allocation, FulfilledClaims,
    Stream, Vesting
};

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
    for Stream {addr, amount, vesting} in SCHEDULE.predefined.iter() {
        match vesting {
            Vesting::Immediate {} => {
                if addr == recipient {
                    return immediate(now, launched, amount.u128());
                }
            },
            Vesting::Monthly { start_at, duration, cliff } => {
                if addr == recipient {
                    return periodic(
                        amount.u128(), MONTH, launched, now,
                        *start_at, *duration, *cliff
                    )
                }
            },
            Vesting::Daily { start_at, duration, cliff } => {
                if addr == recipient {
                    return periodic(
                        amount.u128(), DAY, launched, now,
                        *start_at, *duration, *cliff
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

/// Immediate vesting: if the contract has launched,
/// the recipient can claim the entire allocated amount
fn immediate (now: Seconds, launched: Seconds, amount: Amount) -> Amount {
    return if now >= launched {
        amount
    } else {
        0
    }
}

/// Periodic vesting: calculate how much the user can claim at the given time.
fn periodic (
    amount: Amount, interval: Seconds,
    launched: Seconds, now: Seconds,
    start_at: Seconds, duration: Seconds, cliff: Percentage,
) -> Amount {
    let t_start = launched + start_at;
    if now >= t_start {
        let t_end = t_start + duration;
        let c: u128 = cliff.into();
        if c * amount % 100 > 0 { warn_cliff_remainder() }
        let cliff_amount = (c * amount / 100) as u128;
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
