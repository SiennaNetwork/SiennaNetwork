#![allow(non_snake_case)]
#[macro_use] extern crate sienna_mgmt;
#[macro_use] extern crate kukumba;
#[macro_use] extern crate sienna_schedule; use sienna_schedule::{Schedule,Pool,Account,Vesting};
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::Uint128;

kukumba!(

    #[configure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "anyone but the admin tries to set a configuration"
    then "that fails" {
        for sender in [&BOB, &MALLORY].iter() {
            test_tx!(deps, sender.clone(), 0, 0;
                Configure { schedule: Schedule::new(&[]) }
                => tx_err_auth!());
        }
    }
    when "the admin sets a minimal valid configuration" {
        let s0 = Schedule::new(&[])
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s0.clone() } => tx_ok!());
    } then "the configuration is updated" {
    }
    when "someone else tries to set a valid configuration" {
        let sX = Schedule::new(&[Pool::full("P0", &[Account::immediate("Mallory", &MALLORY.clone(), 1)])]);
        test_tx!(deps, MALLORY, 0, 0; Configure { schedule: sX.clone() } => tx_err_auth!());
    } then "the configuration remains unchanged" {
        test_q!(deps, GetSchedule; Schedule { schedule: s0, total: Uint128::zero() });
    }
    when "the admin sets the planned production configuration" {
        let s: Schedule = serde_json::from_str(include_str!("../../../settings/config.json")).unwrap();
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s.clone() } => tx_ok!());
    } then "the configuration is updated" {
        test_q!(deps, GetSchedule; Schedule { schedule: s.clone(), total: Uint128::zero() });
    }
    when "the contract launches" {
        test_tx!(deps, ALICE, 0, 0; Launch {} => tx_ok_launch!(s.total));
    } then "the configuration can't be changed anymore" {
        test_tx!(deps, ALICE,   0, 0; Configure { schedule: s.clone() } => tx_err!(UNDERWAY));
        test_tx!(deps, BOB,     0, 0; Configure { schedule: s.clone() } => tx_err_auth!());
        test_tx!(deps, MALLORY, 0, 0; Configure { schedule: s.clone() } => tx_err_auth!());
    }

);
