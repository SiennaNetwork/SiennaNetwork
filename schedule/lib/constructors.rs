//! Opinionated helpers for instantiating schedule objects.

use crate::*;

pub fn schedule (total: u128, pools: Vec<Pool>) -> Schedule {
    Schedule { total: Uint128::from(total), pools }
}

pub fn pool (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: false }
}
pub fn pool_partial (name: &str, total: u128, channels: Vec<Channel>) -> Pool {
    Pool { name: name.to_string(), total: Uint128::from(total), channels, partial: true }
}
pub fn channel_immediate (
    amount: u128,
    address: &HumanAddr
) -> Channel {
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: None,
        allocations: vec![(0, vec![allocation(amount, address)])],
    }
}
pub fn channel_immediate_multi (
    _amount: u128,
    _allocations: &Vec<Allocation>
) -> Channel {
    panic!("immediate vesting with multiple recipients is not supported")
}
pub fn channel_periodic (
    amount:   u128,
    address:  &HumanAddr,
    interval: Seconds,
    start_at: Seconds,
    duration: Seconds,
    cliff:    u128
) -> StdResult<Channel> {
    let mut channel = Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(periodic_validated(amount, start_at, cliff, duration, interval)?),
        allocations: vec![]
    };
    let portion = channel.portion_size()?;
    channel.allocations.push((0, vec![allocation(portion, address)]));
    Ok(channel)
}
pub fn channel_periodic_multi (
    amount:      u128,
    allocations: &Vec<Allocation>,
    interval:    Seconds,
    start_at:    Seconds,
    duration:    Seconds,
    cliff:       u128
) -> Channel {
    if cliff > 0 { panic!("periodic vesting with cliff and multiple recipients is not supported") }
    Channel {
        name: String::new(),
        amount: Uint128::from(amount),
        periodic: Some(periodic_validated(amount, start_at, cliff, duration, interval).unwrap()),
        allocations: vec![(0, allocations.clone())]
    }
}
pub fn periodic (
    start_at: Seconds,
    cliff:    u128,
    duration: Seconds,
    interval: Seconds
) -> Periodic {
    Periodic {
        interval,
        start_at,
        duration,
        cliff:              Uint128::from(cliff),
        expected_portion:   Uint128::zero(),
        expected_remainder: Uint128::zero()
    }
}
pub fn periodic_validated (
    amount:   u128,
    start_at: Seconds,
    cliff:    u128,
    duration: Seconds,
    interval: Seconds
) -> StdResult<Periodic> {
    let mut p = Periodic {
        interval,
        start_at,
        duration,
        cliff:              Uint128::from(cliff),
        expected_portion:   Uint128::zero(),
        expected_remainder: Uint128::zero()
    };
    let portion = p.portion_size("", amount)?;
    let n_portions = p.portion_count("")?;
    p.expected_portion = Uint128::from(portion);

    let mut remainder = amount;
    remainder -= cliff;
    remainder -= portion * n_portions as u128;
    p.expected_remainder = Uint128::from(remainder);

    Ok(p)
}
pub fn allocation (amount: u128, addr: &HumanAddr) -> Allocation {
    Allocation { amount: Uint128::from(amount), addr: addr.clone() }
}
pub fn portion (amt: u128, addr: &HumanAddr, vested: Seconds, reason: &str) -> Portion {
    Portion {
        amount:  Uint128::from(amt),
        address: addr.clone(),
        vested:  vested,
        reason:  reason.to_string()
    }
}
