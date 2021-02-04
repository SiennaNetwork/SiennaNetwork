use cosmwasm_std::HumanAddr;

use crate::schedule::SCHEDULE;
use crate::constants::{
    DAY, MONTH,
    warn_cliff_remainder, warn_vesting_remainder
};
use crate::types::{
    Seconds, Amount, Percentage,
    Allocation, FulfilledClaims,
    Stream, Vesting, Interval
};

/// Imports the schedule from JSON during compilation.
//const SRC: &str = include_str!("schedule.yml");
//lazy_static! {
    //pub static ref SCHEDULE: Schedule = serde_yaml::from_str(&SRC).unwrap();
//}

/// Determine how much an account has claimed
/// based on the history of fulfilled claims.
pub fn claimed (
    a:      &HumanAddr,
    vested: &FulfilledClaims,
    t:      Seconds
) -> Amount {
    let mut sum = 0;
    for (addr, time, amount) in vested.iter().rev() {
       if addr != a { continue }
       if *time > t { continue }
       sum += amount.u128();
    }
    sum
}

#[test]
fn test_claimed () {
    use cosmwasm_std::Uint128;
    let alice = HumanAddr::from("alice");
    let bobby = HumanAddr::from("bob");
    let log = vec![ (alice.clone(), 100, 100u128.into())
                  , (bobby.clone(), 100, 200u128.into())
                  , (alice.clone(), 200, 300u128.into()) ];
    assert_eq!(claimed(&alice, &log,   0),   0);
    assert_eq!(claimed(&alice, &log,   1),   0);
    assert_eq!(claimed(&alice, &log, 100), 100);
    assert_eq!(claimed(&alice, &log, 101), 100);
    assert_eq!(claimed(&alice, &log, 200), 400);
    assert_eq!(claimed(&alice, &log, 999), 400);
    assert_eq!(claimed(&bobby, &log, 999), 200);
    assert_eq!(claimed(&bobby, &log,  99),   0);
}

/// Determine how much one can claim
/// based on the predefined schedule
/// and the configurable allocation.
pub fn claimable (
    recipient:  &HumanAddr,
    recipients: &Allocation,
    launched:   Seconds,
    now:        Seconds,
) -> Amount {

    // Nothing can be vested before the launch date
    if now < launched { return 0 }

    // Preconfigured claimants:
    for Stream {addr, amount, vesting} in SCHEDULE.predefined.iter() {
        if addr != recipient { continue }

        return match vesting {
            // Immediate vesting: if the contract has launched,
            // the recipient can claim the entire allocated amount
            Vesting::Immediate {} => (*amount).u128(),

            // Periodic vesting: need to calculate the maximum amount
            // that the user can claim at the given time.
            Vesting::Periodic { interval, start_at, duration, cliff } => {
                let interval = match interval {
                    Interval::Daily   => DAY,
                    Interval::Monthly => MONTH
                };
                // Can't vest before the cliff
                let start = launched + start_at;
                if now < start { return 0 }
                periodic(
                    amount.u128(), interval, now - start,
                    *start_at, *duration, *cliff
                )
            },
        }
    }

    // Configurable daily vesting:
    for (addr, amount) in recipients {
        if addr != recipient { continue }

        let days_since_launch = (now - launched) / DAY;
        return (*amount).u128() * (days_since_launch + 1) as u128;
    }

    // Default case:
    0
}

fn periodic (
    amount: Amount, interval: Seconds, since_start: Seconds,
    start_at: Seconds, duration: Seconds, cliff: Percentage,
) -> Amount {

    // mutable for clarity:
    let mut vest = 0;

    // start with the cliff amount
    let cliff = cliff as u128;
    if cliff * amount % 100 > 0 { warn_cliff_remainder() }
    let cliff_amount = (cliff * amount / 100) as u128;
    vest += cliff_amount;

    // then for every `interval` since `t_start`
    // add an equal portion of the remaining amount

    // then, from the remaining amount and the number of vestings
    // determine the size of the portion
    let post_cliff_amount = amount - cliff_amount;
    let n_total: u128 = (duration / interval).into();
    if post_cliff_amount % n_total > 0 { warn_vesting_remainder() }
    let portion = post_cliff_amount / n_total;

    // then determine how many vesting periods have elapsed,
    // up to the maximum; `duration - interval` and `1 + n_elapsed`
    // are used to ensure vesting happens at the begginning of an interval
    let t_elapsed = Seconds::min(since_start, duration - interval);
    let n_elapsed = t_elapsed / interval;
    let mut n_elapsed: u128 = (1 + n_elapsed).into();
    //if t_elapsed % interval > interval / 2 { n_elapsed += 1; }

    // then add that amount to the cliff amount
    vest += portion * n_elapsed;

    //println!("periodic {}/{}={} -> {}", n_elapsed, n_total, n_elapsed/n_total, vest);
    vest
}
