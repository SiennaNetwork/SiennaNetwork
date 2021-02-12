#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::Uint128;
use sienna_schedule::{
    Schedule, schedule, pool, pool_partial, Channel, periodic,
    channel_periodic, channel_periodic_multi, allocation
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
            schedule(200000u128,vec![pool_partial("Advisors", 200000u128,vec![
                Channel {
                    name:        "Invalid1".to_string(),
                    amount:      Uint128::from(11000u128),
                    periodic:    Some(periodic(15552000, 1000u128, 15552001, 86400)),
                    allocations: vec![]
                }])]),
            "channel Invalid1: duration (15552001s) does not divide evenly in intervals of 86400s"
        ), /* (DISABLED - currently implementing remainders 
            schedule(200000u128,vec![pool_partial("Advisors", 200000u128,vec![
                Channel {
                    name:        "Invalid2".to_string(),
                    amount:      Uint128::from(11000u128),
                    periodic:    Some(periodic(15552000, 1000u128, 15552000, 86400)),
                    allocations: vec![]
                }])]),
            "channel Invalid2: post-cliff amount 9900 does not divide evenly in 179 portions"
        )*/].iter() {
            test_tx!(deps, ALICE, 0, 0;
                Configure { schedule: schedule.clone() } => tx_err!(error));
        }
    }
    when "the admin sets a minimal valid configuration" {
        let s0 = schedule(150, vec![
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
            ]),
            pool_partial("P3", 50, vec![])
        ])
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s0.clone() } => tx_ok!());
    } then "the configuration is updated" {
        test_tx!(deps, ALICE, 0, 0; AddChannel {
            pool_name:   "P3".to_string(),
            name:        "FOO".to_string(),
            amount:      Uint128::from(20u128),
            allocations: vec![],
            periodic:    None
        } => tx_ok!());
        test_tx!(deps, ALICE, 0, 0; AddChannel {
            pool_name:   "P3".to_string(),
            name:        "BAR".to_string(),
            amount:      Uint128::from(20u128),
            allocations: vec![],
            periodic:    None
        } => tx_ok!());
        test_tx!(deps, ALICE, 0, 0; AddChannel {
            pool_name:   "P3".to_string(),
            name:        "BAZ".to_string(),
            amount:      Uint128::from(40u128),
            allocations: vec![],
            periodic:    None
        } => tx_err!("pool P3: tried to add channel with size 40, which is more than the remaining 10 of this pool's total 50"));
    }
    when "the admin adds a new account to a partial pool" {
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s0.clone() } => tx_ok!());
    } then "the configuration is updated" {
    }
    when "someone else tries to set a valid configuration" {
        test_tx!(deps, MALLORY, 0, 0; Configure { schedule: schedule(0, vec![]) } => tx_err_auth!());
    } then "the configuration remains unchanged" {
        test_q!(deps, GetSchedule; Schedule { schedule: Some(s0.clone()) });
    }
    when "the admin sets the planned production configuration" {
        let s: Schedule = serde_json::from_str(include_str!("../../config.json")).unwrap();
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s.clone() } => tx_ok!());
    } then "the configuration is updated" {
        test_q!(deps, GetSchedule; Schedule { schedule: Some(s.clone()) });
    }
    when "the contract launches" {
        test_tx!(deps, ALICE, 0, 0; Launch {} => tx_ok_launch!(s.total));
    } then "the configuration can't be changed anymore" {
        test_tx!(deps, ALICE,   0, 0; Configure { schedule: s0.clone() } => tx_err!(sienna_mgmt::UNDERWAY));
        test_tx!(deps, BOB,     0, 0; Configure { schedule: s0.clone() } => tx_err_auth!());
        test_tx!(deps, MALLORY, 0, 0; Configure { schedule: s0.clone() } => tx_err_auth!());
    }

);
