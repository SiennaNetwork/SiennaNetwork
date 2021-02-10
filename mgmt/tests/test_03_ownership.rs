#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128, HandleResponse};
use sienna_mgmt::msg::Handle;
use sienna_schedule::Schedule;

kukumba!(

    #[transfer_ownership]

    given "a contract instance" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "a stranger tries to set a new admin"
    then "just the hit counter goes up" {
        test_tx!(deps, MALLORY, 1, 1;
            Handle::TransferOwnership { new_admin: MALLORY.clone() } =>
            tx_err_auth!());
    }
    when "the admin tries to set a new admin"
    then "the admin is updated" {
        test_tx!(deps, ALICE, 2, 2;
            Handle::TransferOwnership { new_admin: BOB.clone() } =>
            tx_ok!());
    }
    when "the former admin tries to set a new admin"
    then "just the hit counter goes up" {
        test_tx!(deps, ALICE, 3, 3;
            Handle::TransferOwnership { new_admin: ALICE.clone() } =>
            tx_err_auth!());
    }
    when "the new admin tries to set the admin"
    then "the admin is updated" {
        test_tx!(deps, BOB, 4, 4;
            Handle::TransferOwnership { new_admin: ALICE.clone() } =>
            tx_ok!());
    }
    when "someone else tries to disown the contract"
    and  "just the hit counter goes up" {
        test_tx!(deps, MALLORY, 5, 5;
            Handle::Disown {} =>
            tx_err_auth!());
    }
    when "the admin disowns the contract"
    then "there is no admin"
    and  "nobody can control the contract" {
        test_tx!(deps, ALICE, 6, 6;
            Handle::Disown {} =>
            tx_ok!());
        test_tx!(deps, ALICE, 2, 2;
            Handle::TransferOwnership { new_admin: ALICE.clone() } =>
            tx_err_auth!());
    }

);

