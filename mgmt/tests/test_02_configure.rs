#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
//use mgmt::{DAY, MONTH, ONE_SIENNA, err_allocation, Stream, Vesting};

use cosmwasm_std::{StdError, HumanAddr, Uint128, HandleResponse};

use secret_toolkit::snip20;

use sienna_mgmt::msg::Handle;
use sienna_schedule::{Schedule, Pool, Account, Vesting, Allocation};

kukumba!(

    #[configure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "anyone tries to set an invalid configuration" {
        let s1 = Schedule::new(100u128, vec![]);
    } then "that fails" {
        test_tx!(deps, ALICE, 0, 0;
            Handle::Configure { schedule: s1.clone() } =>
            tx_err!("schedule's pools add up to 0, expected 100"));
    }
    when "the admin tries to set a valid configuration" {
        let s1 = Schedule::new(100, vec![
            Pool::new(10, vec![
                Account::new(10, Vesting::Immediate {}, vec![
                    Allocation::new(10, BOB.clone())
                ])
            ]),
            Pool::new(90, vec![
                Account::new(45, Vesting::Immediate {}, vec![
                    Allocation::new(45, BOB.clone())
                ]),
                Account::new(45, Vesting::Immediate {}, vec![
                    Allocation::new( 5, BOB.clone()),
                    Allocation::new(10, BOB.clone()),
                    Allocation::new(30, BOB.clone())
                ])
            ])
        ])
        test_tx!(deps, ALICE, 0, 0;
            Handle::Configure { schedule: s1.clone() } => tx_ok!());
    } then "the configuration is updated" {
        let pools = s1.pools.clone();
        test_q!(deps, Schedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }
    when "someone else tries to set a valid configuration" {
        test_tx!(deps, MALLORY, 0, 0;
            Handle::Configure { schedule: Schedule::new(0, vec![]) } =>
                tx_err_auth!());
    } then "the configuration remains unchanged" {
        let pools = s1.pools.clone();
        test_q!(deps, Schedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }

);
