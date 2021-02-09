#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
//use mgmt::{DAY, MONTH, ONE_SIENNA, err_allocation, Stream, Vesting};

use cosmwasm_std::{StdError, HumanAddr, Uint128, HandleResponse};

use secret_toolkit::snip20;

use sienna_mgmt::msg::Handle;
use sienna_schedule::Schedule;

kukumba!(

    #[configure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }

    when "the admin tries to set the configuration" {
        let s1 = Schedule { total: Uint128::from(100u128), pools: vec![] }
        test_tx!(deps, ALICE, 0, 0;
            Handle::Configure { schedule: s1.clone() } => tx_ok!());
    }

    then "the configuration should be updated" {
        let pools = s1.pools.clone();
        test_q!(deps, Schedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }

    when "someone else tries to set the configuration" {
        let s2 = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps, MALLORY, 0, 0;
            Handle::Configure { schedule: s2 } => tx_err_auth!());
    }

    then "the configuration should remain unchanged" {
        let pools = s1.pools.clone();
        test_q!(deps, Schedule;
            Schedule {
                schedule: Some(Schedule { total: s1.total, pools })
            });
    }

);
