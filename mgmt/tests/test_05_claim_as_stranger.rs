#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{HumanAddr, Uint128};
use sienna_mgmt::{PRELAUNCH, NOTHING};
use sienna_schedule::Schedule;

kukumba!(

    #[claim_as_stranger]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps, MALLORY, 1, 1;
            Claim {} => tx_err!(PRELAUNCH));
    }

    given "the contract is launched" {
        let s = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps, ALICE, 0, 0;
            Configure { schedule: s.clone() } => tx_ok!());
        test_tx!(deps, ALICE, 2, 2;
            Launch {} => tx_ok_launch!(s.total));
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps, MALLORY, 4, 4;
            Claim {} => tx_err!(NOTHING));
    }

);
