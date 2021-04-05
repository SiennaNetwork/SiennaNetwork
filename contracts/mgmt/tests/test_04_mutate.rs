#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate sienna_mgmt;
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env};
use cosmwasm_std::{HumanAddr, Uint128};
use sienna_schedule::{Schedule, Pool, Account};

kukumba! {

    #[no_unauthorized_mutate_before_launch]
    given "an instance" { harness!(deps; ADMIN, STRANGER); }
    and "a schedule with a partial pool" {
        let original_schedule = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: original_schedule.clone() } == ok!()); }
    when "a stranger tries to add an account to an existing pool"
    then "that fails" {
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        tx!(deps; STRANGER, 1, 1; AddAccount { pool_name: "pool".to_string(), account: a }
            == err!(auth));
        q!(deps; Schedule == Schedule { schedule: original_schedule }); }

    #[no_add_user_to_missing_pool]
    given "an instance" { harness!(deps; ADMIN); }
    and "a schedule with a partial pool" {
        let original_schedule = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: original_schedule.clone() } == ok!()); }
    when "the admin tries to add an account to a missing pool"
    then "that fails" {
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        tx!(deps; ADMIN, 1, 1; AddAccount { pool_name: "missing".to_string(), account: a }
            == err!(auth)); 
        q!(deps; Schedule == Schedule { schedule: original_schedule }); }

    #[ok_add_user_to_pool_before_launch]
    given "an instance" { harness!(deps; ADMIN); }
    and "a schedule with a partial pool" {
        let original_schedule = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: original_schedule.clone() } == ok!()); }
    when "the admin tries to add an account to a pool"
    then "the schedule is updated"
    and "the correct amounts claimable can be queried for the new account" {
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        let mut updated_schedule = original_schedule.clone();
        updated_schedule.add_account("pool".to_string(), a.clone()).unwrap();
        tx!(deps; ADMIN, 1, 1; AddAccount { pool_name: "pool".to_string(), account: a.clone() }
            == ok!());
        q!(deps; Schedule == Schedule { schedule: updated_schedule });
        q!(deps; Progress { address: a.address, time: 0 } == Progress { unlocked: cosmwasm_std::Uint128::zero() }); }

    #[no_add_user_to_full_pool_before_launch]
    given "an instance" { harness!(deps; ADMIN); }
    and "a schedule with a partial pool" {
        let s = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: s.clone() } == ok!()); }
    when "the admin tries to add an account to a pool"
    and "the account's amount is more than what's left in the pool"
    then "that fails" {
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        let ADD_ACCOUNT = MGMTError!(ADD_ACCOUNT);
        tx!(deps; ADMIN, 1, 1; AddAccount { pool_name: "pool".to_string(), account: a }
            == err!(ADD_ACCOUNT)); }

    #[no_unauthorized_mutate_after_launch]
    given "a launched instance" { harness!(deps; ADMIN, STRANGER); }
    and "a schedule with a partial pool" {
        let s = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: s.clone() } == ok!()); }
    when "someone tries to add an account to an existing pool" {}
    then "that fails" {
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        tx!(deps; STRANGER, 1, 1; AddAccount { pool_name: "bad".to_string(), account: a } == err!(auth)); }

    #[ok_add_user_to_pool_after_launch]
    given "a launched instance" { harness!(deps; ADMIN, STRANGER); }
    and "a schedule with a partial pool" {
        let original_schedule = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: original_schedule.clone() } == ok!()); }
    when "the admin tries to add an account to a missing pool" {}
    then "the schedule is updated" {}
    and "the correct amounts claimable can be queried for the new account" {
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        let mut updated_schedule = original_schedule.clone();
        updated_schedule.add_account("pool".to_string(), a.clone()).unwrap();
        tx!(deps; ADMIN, 1, 1; AddAccount { pool_name: "pool".to_string(), account: a.clone() }
            == ok!());
        q!(deps; Schedule == Schedule { schedule: updated_schedule });
        q!(deps; Progress { address: a.address, time: 0 } == Progress { unlocked: cosmwasm_std::Uint128::zero() }); }

    #[no_add_user_to_full_pool_after_launch]
    given "a launched instance" { harness!(deps; ADMIN, STRANGER); }
    and "a schedule with a partial pool" {
        let s = Schedule::new(&[Pool::partial("pool", 1000, &[])]);
        tx!(deps; ADMIN, 0, 0; Configure { schedule: s.clone() } == ok!()); }
    when "the admin tries to add an account to a pool"
    and "the account's amount is more than what's left in the pool" {}
    then "that fails" {
        let ADD_ACCOUNT = MGMTError!(ADD_ACCOUNT);
        let a = Account::immediate("account", &HumanAddr::from("account"), 500);
        tx!(deps; ADMIN, 1, 1; AddAccount { pool_name: "pool".to_string(), account: a } == err!(ADD_ACCOUNT)); }

}
