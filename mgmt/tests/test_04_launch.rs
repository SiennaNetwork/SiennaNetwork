#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128};
use sienna_schedule::Schedule;

kukumba!(

    #[launch]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }
    when "a stranger tries to start the vesting" 
    then "that fails" {
        test_tx!(deps, MALLORY, 2, 2;
            Launch {} => tx_err_auth!());
        test_q!(deps, Status;
            Status { launched: None, errors: 1 });
    }
    when "the contract is not configured"
    and  "the admin tries to start the vesting"
    then "that fails" {
        test_tx!(deps, ALICE, 3, 3;
            Launch {} => tx_err!(sienna_mgmt::NO_SCHEDULE));
        test_q!(deps, Status;
            Status { launched: None, errors: 2 });
    }
    when "the contract is configured"
    and  "the admin starts the vesting"
    then "the contract mints the tokens"
    and  "the current time is remembered as the launch date" {
        let s = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps, ALICE, 0, 0;
            Configure { schedule: s.clone() } => tx_ok!());
        test_tx!(deps, ALICE, 4, 4;
            Launch {} => tx_ok_launch!(s.total));
        test_q!(deps, Status;
            Status { launched: Some(4), errors: 2 });
    }
    given "the contract is already launched"
    when "the admin tries to start the vesting again"
    then "the contract says it's already launched"
    and "it does not mint tokens"
    and "it does not update its launch date" {
        test_tx!(deps, ALICE, 5, 5;
            Launch {} => tx_err!(sienna_mgmt::UNDERWAY));
        test_q!(deps, Status;
            Status { launched: Some(4), errors: 3 });
    }

);
