use crate::units::*;
use super::*;

#[test]
fn valid_schedule_with_main_features () {
    let alice = HumanAddr::from("Alice");
    let bob = HumanAddr::from("Bob");
    let s = schedule(110, vec![
        pool("P1", 50, vec![
            channel_immediate(29, &alice),
            channel_immediate(1, &bob),
            channel_periodic(20, &alice, 1, 0, 1, 0).unwrap()
        ]),
        pool("P2", 60, vec![
            channel_periodic(10, &alice, 1, 0, 2, 2).unwrap(),
            channel_periodic_multi(50, &vec![
                allocation(28, &alice),
                allocation( 3, &bob),
                allocation(19, &alice)
            ], 1, 0, 1, 0)])]);
    valid!(s);
    claim!(s, alice, 0,
        portion(29u128, &alice, 0u64, ": immediate"),
        portion(20u128, &alice, 0u64, ": vesting"),
        portion( 2u128, &alice, 0u64, ": cliff"),
        portion(28u128, &alice, 0u64, ": vesting"),
        portion(19u128, &alice, 0u64, ": vesting"));
    claim!(s, bob, 0,
        portion(1u128, &bob, 0u64, ": immediate"),
        portion(3u128, &bob, 0u64, ": vesting"));
}

#[test]
fn test_channel_immediate () {
    let alice = HumanAddr::from("Alice");
    claim!(channel_immediate(100, &alice), &alice, 0,
        portion(100u128, &alice, 0u64, ": immediate"));
}

#[test]
fn test_channel_periodic_no_cliff () {
    let total    = 300;
    let interval = DAY;
    let start_at = 100;
    let duration = 3*DAY;
    let cliff    = 0;
    let alice    = HumanAddr::from("Alice");
    let bob      = HumanAddr::from("Bob");

    let c = schedule(total,vec![pool("P1",total,vec![channel_periodic_multi(
        total, &vec![
            allocation(40, &alice),
            allocation(60, &bob)
        ], interval, start_at, duration, cliff)])]);

    claim!(c, alice, start_at - 1);

    claim!(c, alice, start_at,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"));

    claim!(c, alice, start_at + 1,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"));

    claim!(c, alice, start_at + interval,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 1*interval, ": vesting"));

    claim!(c, alice, start_at + interval + interval / 2,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 1*interval, ": vesting"));

    claim!(c, alice, start_at + 2*interval,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 1*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 2*interval, ": vesting"));

    claim!(c, alice, start_at + 2*interval + interval / 2,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 1*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 2*interval, ": vesting"));

    claim!(c, alice, start_at + 3*interval,
        portion( 40u128, &alice, start_at + 0*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 1*interval, ": vesting"),
        portion( 40u128, &alice, start_at + 2*interval, ": vesting"));
}

#[test]
fn test_channel_periodic_with_cliff_only () {
    // duration = interval -> [cliff]
    // but that should be an immediate channel instead, so it fails
    let alice    = HumanAddr::from("Alice");
    let total    = 100;
    let interval = DAY;
    let start_at = 1;
    let duration = interval;
    let cliff    = 1u128;
    assert_eq!(channel_periodic(total, &alice, interval, start_at, duration, cliff),
        Error!("channel : periodic vesting must contain at least 1 non-cliff portion"));
}

#[test]
fn test_channel_periodic_with_cliff_and_1_vesting () {
    // duration = 2*interval -> [cliff, vesting]
    let alice    = HumanAddr::from("Alice");
    let total    = 100;
    let interval = DAY;
    let start_at = 1;
    let duration = 2*interval;
    let cliff    = 1u128;

    let c = channel_periodic(total, &alice, interval, start_at, duration, cliff).unwrap();
    assert_eq!(c.portion_count(), Ok(1));
    assert_eq!(c.portion_size(),  Ok(99u128));

    claim!(c, alice, start_at,
        portion(cliff, &alice, start_at,           ": cliff"));

    claim!(c, alice, start_at + 1,
        portion(cliff, &alice, start_at,           ": cliff"));

    claim!(c, alice, start_at + interval,
        portion(cliff,  &alice, start_at,          ": cliff")
               ,portion(99u128, &alice, start_at+interval, ": vesting"));

    claim!(c, alice, start_at + 10*interval,
        portion(cliff,  &alice, start_at,          ": cliff")
               ,portion(99u128, &alice, start_at+interval, ": vesting"));
}

#[test]
fn test_channel_periodic_with_cliff_and_2_vestings () {
    // duration = 3*interval -> [cliff, vesting, vesting]
    let alice    = HumanAddr::from("Alice");
    let total    = 201;
    let interval = DAY;
    let start_at = 1;
    let duration = 3*interval;
    let cliff    = 1u128;

    // if cliff > 0 then the first portion is the cliff
    // and the remaining amount is divided by `portion_count`

    let c = channel_periodic(total, &alice, interval, start_at, duration, cliff).unwrap();
    assert_eq!(c.portion_count(), Ok(2));
    assert_eq!(c.portion_size(),  Ok(100u128));

    claim!(c, alice, start_at,
        portion(cliff,   &alice, start_at,            ": cliff"));

    claim!(c, alice, start_at + 1,
        portion(cliff,   &alice, start_at,            ": cliff"));

    claim!(c, alice, start_at + interval,
        portion(cliff,   &alice, start_at,            ": cliff"),
        portion(100u128, &alice, start_at+interval,   ": vesting"));

    claim!(c, alice, start_at + 2*interval,
        portion(cliff,   &alice, start_at,            ": cliff"),
        portion(100u128, &alice, start_at+interval,   ": vesting"),
        portion(100u128, &alice, start_at+2*interval, ": vesting"));

    claim!(c, alice, start_at + 10*interval,
        portion(cliff,   &alice, start_at,            ": cliff"),
        portion(100u128, &alice, start_at+interval,   ": vesting"),
        portion(100u128, &alice, start_at+2*interval, ": vesting"));
}

#[test]
fn test_reallocation () {
    // TODO: time of reallocation should be time of last claimed portion
    let alice   = HumanAddr::from("Alice");
    let bob     = HumanAddr::from("Bob");
    let charlie = HumanAddr::from("Charlie");

    let interval = DAY;
    let start_at = 0;
    let duration = 7 * DAY;
    let cliff    = 0;

    let mut s = channel_periodic_multi(
        700u128, &vec![allocation(100u128, &alice)],
        interval, start_at, duration, cliff);

    claim!(s, alice, 0*DAY,
        portion(100u128, &alice, 0 * DAY, ": vesting"));
    claim!(s, alice, 1*DAY,
        portion(100u128, &alice, 0 * DAY, ": vesting"),
        portion(100u128, &alice, 1 * DAY, ": vesting"));
    claim!(s, alice, 2*DAY,
        portion(100u128, &alice, 0 * DAY, ": vesting"),
        portion(100u128, &alice, 1 * DAY, ": vesting"),
        portion(100u128, &alice, 2 * DAY, ": vesting"));
    claim!(s, alice, 3*DAY,
        portion(100u128, &alice, 0 * DAY, ": vesting"),
        portion(100u128, &alice, 1 * DAY, ": vesting"),
        portion(100u128, &alice, 2 * DAY, ": vesting"),
        portion(100u128, &alice, 3 * DAY, ": vesting"));
    claim!(s, bob, 0*DAY);
    claim!(s, bob, 1*DAY);
    claim!(s, bob, 2*DAY);
    claim!(s, bob, 3*DAY);
    claim!(s, charlie, 10*DAY);
    s.reallocate(4*DAY, vec![allocation(50u128, &alice)
                            ,allocation(50u128, &bob)]).unwrap();
    claim!(s, alice, 4*DAY,
        portion(100u128, &alice, 0 * DAY, ": vesting"),
        portion(100u128, &alice, 1 * DAY, ": vesting"),
        portion(100u128, &alice, 2 * DAY, ": vesting"),
        portion(100u128, &alice, 3 * DAY, ": vesting"),
        portion(50u128, &alice, 4 * DAY, ": vesting"));
    claim!(s, alice, 7*DAY,
        portion(100u128, &alice, 0 * DAY, ": vesting"),
        portion(100u128, &alice, 1 * DAY, ": vesting"),
        portion(100u128, &alice, 2 * DAY, ": vesting"),
        portion(100u128, &alice, 3 * DAY, ": vesting"),
        portion(50u128, &alice, 4 * DAY, ": vesting"),
        portion(50u128, &alice, 5 * DAY, ": vesting"),
        portion(50u128, &alice, 6 * DAY, ": vesting"));
    claim!(s, bob, 5*DAY,
        portion(50u128, &bob, 4 * DAY, ": vesting"),
        portion(50u128, &bob, 5 * DAY, ": vesting"));
    claim!(s, bob, 10*DAY,
        portion(50u128, &bob, 4 * DAY, ": vesting"),
        portion(50u128, &bob, 5 * DAY, ": vesting"),
        portion(50u128, &bob, 6 * DAY, ": vesting"));
    claim!(s, charlie, 10*DAY);
}

#[test]
fn test_channel_with_cliff_multiple_vestings_partial_reallocation_and_remainder () {
}
