#![allow(unused_macros)]
#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env};

kukumba!(

    #[transfer_ownership]

    given "a contract instance" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "a stranger tries to set a new admin"
    then "just the hit counter goes up" {
        test_tx!(deps; MALLORY, 1, 1; SetOwner { new_admin: MALLORY.clone() } == err!(auth));
    }
    when "the admin tries to set a new admin"
    then "the admin is updated" {
        test_tx!(deps; ALICE, 2, 2; SetOwner { new_admin: BOB.clone() } == ok!());
    }
    when "the former admin tries to set a new admin"
    then "just the hit counter goes up" {
        test_tx!(deps; ALICE, 3, 3; SetOwner { new_admin: ALICE.clone() } == err!(auth));
    }
    when "the new admin tries to set the admin"
    then "the admin is updated" {
        test_tx!(deps; BOB, 4, 4; SetOwner { new_admin: ALICE.clone() } == ok!());
    }
    when "someone else tries to disown the contract"
    and  "just the hit counter goes up" {
        test_tx!(deps; MALLORY, 5, 5; Disown {} == err!(auth));
    }
    when "the admin disowns the contract"
    then "there is no admin"
    and  "nobody can control the contract" {
        test_tx!(deps; ALICE, 6, 6; Disown {} == ok!());
        test_tx!(deps; ALICE, 2, 2; SetOwner { new_admin: ALICE.clone() } == err!(auth));
    }

);

