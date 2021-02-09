#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
//use mgmt::{DAY, MONTH, ONE_SIENNA, err_allocation, Stream, Vesting};

use cosmwasm_std::{StdError, HumanAddr, Uint128, HandleResponse};

use secret_toolkit::snip20;

use sienna_mgmt::msg::Handle;
use sienna_schedule::{Schedule, Pool, Account, Allocation};

kukumba!(

    #[configure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "anyone tries to set an invalid configuration" {
        let s1 = Schedule { total: Uint128::from(100u128), pools: vec![] }
    } then "that fails" {
        test_tx!(deps, ALICE, 0, 0;
            Handle::Configure { schedule: s1.clone() } => tx_ok!());
    }
    when "the admin tries to set a valid configuration" {
        let s1 = Schedule {
            total: Uint128::from(100u128),
            pools: vec![
                Pool {
                    total: Uint128::from(10u128),
                    accounts: vec![]
                },
                Pool {
                    total: Uint128::from(90u128),
                    accounts: vec![]
                }
            ]
        }
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
        let s2 = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps, MALLORY, 0, 0;
            Handle::Configure { schedule: s2 } => tx_err_auth!());
    } then "the configuration remains unchanged" {
        let pools = s1.pools.clone();
        test_q!(deps, Schedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }

);
