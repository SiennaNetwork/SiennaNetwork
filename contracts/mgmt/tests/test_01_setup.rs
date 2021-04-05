#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate sienna_mgmt;
extern crate sienna_schedule; use sienna_schedule::{Schedule, Pool, Account};
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env};

kukumba! {

    #[ok_initialize_become_admin]
    given "no instance" { harness!(deps; ALICE); }
    when "someone deploys the instance" {
        use sienna_mgmt::{init, msg::Init};
        let s = sienna_schedule::Schedule::new(&[Pool::full("",&[])]);
        let _ = init(&mut deps, mock_env(0, 0, &ALICE), Init {
            schedule: s.clone(),
            token:    (cosmwasm_std::HumanAddr::from("token"), String::new()),
        }).unwrap(); }
    then "they become admin" {}
    and "the instance is not launched" { q!(deps; Status   == Status   { launched: None }); }
    and "the given schedule is set"    { q!(deps; Schedule == Schedule { schedule: s    }); }

    #[ok_configure_authorized_only]
    given "an instance before launch" { harness!(deps; ALICE, BOB, MALLORY); }
    when "the admin sets a minimal valid configuration" {
        let original_schedule = sienna_schedule::Schedule::new(&[Pool::full("original",&[])]);
        tx!(deps; ALICE, 0, 0; Configure { schedule: original_schedule.clone() } == ok!()); }
    then "the configuration is updated" {
        q!(deps; Schedule == Schedule { schedule: original_schedule }); }
    when "anyone but the admin tries to set a configuration"
    then "that fails" {
        for sender in [&BOB, &MALLORY].iter() {
            let sender = sender.clone();
            let bad_schedule = Schedule::new(&[Pool::full("malicious",&[])]);
            tx!(deps; sender, 0, 0; Configure { schedule: bad_schedule } == err!(auth));
            q!(deps; Schedule == Schedule { schedule: original_schedule }); } }
    when "the admin sets the real configuration" {
        let src = include_str!("../../../settings/schedule.json")
        let s: Schedule = serde_json::from_str(src).unwrap();
        tx!(deps; ALICE, 0, 0; Configure { schedule: s.clone() } == ok!()); }
    then "the configuration is updated" {
        q!(deps; Schedule == Schedule { schedule: s.clone() }); }

    #[no_reconfigure_after_launch]
    given "a launched instance" {
        harness!(deps; ALICE, BOB, MALLORY);
        tx!(deps; ALICE, 0, 0; Launch {} == ok!(launched: cosmwasm_std::Uint128::zero())); }
    then "the total configuration can't be changed anymore by anyone" {
        let s = sienna_schedule::Schedule::new(&[Pool::full("",&[])]);
        let UNDERWAY = MGMTError!(UNDERWAY);
        tx!(deps; ALICE,   0, 0; Configure { schedule: s.clone() } == err!(UNDERWAY));
        tx!(deps; BOB,     0, 0; Configure { schedule: s.clone() } == err!(auth));
        tx!(deps; MALLORY, 0, 0; Configure { schedule: s.clone() } == err!(auth)); }

    #[no_unauthorized_launch]
    given "the instance is not yet launched" {
        harness!(deps; ALICE, MALLORY); }
    when "a stranger tries to start the vesting"
    then "that fails" {
        tx!(deps; MALLORY, 2, 2; Launch {} == err!(auth));
        q!(deps; Status == Status { launched: None }); }

    #[ok_launch]
    given "the instance is not yet launched" {
        harness!(deps; ALICE, MALLORY); }
    when "the instance is configured"
    and  "the admin starts the vesting"
    then "the instance mints the tokens"
    and  "the current time is remembered as the launch date" {
        let s = sienna_schedule::Schedule::new(&[]);
        tx!(deps; ALICE, 3, 3; Configure { schedule: s.clone() } == ok!());
        tx!(deps; ALICE, 4, 4; Launch {} == ok!(launched: s.total));
        q!(deps; Status == Status { launched: Some(4) }); }

    #[no_relaunch]
    given "the instance was launched" {
        harness!(deps; ALICE);
        let s = sienna_schedule::Schedule::new(&[]);
        tx!(deps; ALICE, 3, 3; Configure { schedule: s.clone() } == ok!());
        tx!(deps; ALICE, 4, 4; Launch {} == ok!(launched: s.total)); }
    given "the instance is already launched"
    when "the admin tries to start the vesting again"
    then "the instance says it's already launched"
    and "it does not mint tokens"
    and "it does not update its launch date" {
        let UNDERWAY = MGMTError!(UNDERWAY);
        tx!(deps; ALICE, 5, 5; Launch {} == err!(UNDERWAY));
        q!(deps; Status == Status { launched: Some(4) }); }

}
