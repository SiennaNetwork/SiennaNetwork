#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_schedule::{
    Schedule,
    schedule, pool, pool_partial,
    channel_periodic, channel_periodic_multi,
    allocation
};

kukumba!(

    #[configure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "anyone but the admin tries to set a configuration"
    then "that fails" {
        for sender in [&BOB, &MALLORY].iter() {
            test_tx!(deps, sender.clone(), 0, 0;
                Configure { schedule: schedule(0, vec![]) }
                => tx_err_auth!());
        }
    }
    when "the admin tries to set a configuration that doesn't add up"
    then "that fails" {
        for schedule in [
            schedule(100u128, vec![])
        ].iter() {
            test_tx!(deps, ALICE, 0, 0;
                Configure { schedule: schedule.clone() }
                => tx_err!("schedule: pools add up to 0, expected 100")
            );
        }
    }
    when "the admin tries to set a configuration that doesn't divide evenly"
    then "that fails" {
        for (schedule, error) in [(
            schedule(100u128,
                vec![pool_partial("Advisors", 200000u128,
                    vec![channel_periodic(11000u128, &BOB, 86400, 15552000, 15552001, 1000).unwrap()])]),
            "channel : duration 15552001 does not divide evenly in intervals of 86400"
        ), (
            schedule(100u128,
                vec![pool_partial("Advisors", 200000u128,
                    vec![channel_periodic(11000u128, &BOB, 86400, 15552000, 15552000, 1000).unwrap()])]),
            "channel : post-cliff amount 10000 does not divide evenly in 180 portions"
        )].iter() {
            test_tx!(deps, ALICE, 0, 0;
                Configure { schedule: schedule.clone() } => tx_err!(error));
        }
    }
    when "the sets a valid configuration" {
        let s1 = schedule(100, vec![
            pool("P1", 10, vec![
                channel_periodic_multi(10, &vec![
                    allocation(10, &BOB)
                ], 1, 0, 1, 0)
            ]),
            pool("P2", 90, vec![
                channel_periodic_multi(45, &vec![
                    allocation(45, &BOB)
                ], 1, 0, 1, 0),
                channel_periodic_multi(45, &vec![
                    allocation( 5, &BOB),
                    allocation(10, &BOB),
                    allocation(30, &BOB)
                ], 1, 0, 1, 0)
            ])
        ])
        test_tx!(deps, ALICE, 0, 0;
            Configure { schedule: s1.clone() } => tx_ok!());
    } then "the configuration is updated" {
        let pools = s1.pools.clone();
        test_q!(deps, GetSchedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }
    when "someone else tries to set a valid configuration" {
        test_tx!(deps, MALLORY, 0, 0;
            Configure { schedule: schedule(0, vec![]) } =>
                tx_err_auth!());
    } then "the configuration remains unchanged" {
        let pools = s1.pools.clone();
        test_q!(deps, GetSchedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }
    when "the contract launches" {
        test_tx!(deps, ALICE, 0, 0;
            Launch {} => tx_ok_launch!(s1.total));
    } then "the configuration can't be changed anymore" {
        test_tx!(deps, ALICE, 0, 0;
            Configure { schedule: s1.clone() } => tx_err!(sienna_mgmt::UNDERWAY));
        test_tx!(deps, BOB, 0, 0;
            Configure { schedule: s1.clone() } => tx_err_auth!());
        test_tx!(deps, MALLORY, 0, 0;
            Configure { schedule: s1.clone() } => tx_err_auth!());
    }

);
