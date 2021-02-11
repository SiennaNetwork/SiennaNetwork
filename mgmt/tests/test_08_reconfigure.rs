#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_schedule::{
    Schedule,
    schedule, pool, pool_partial,
    channel_periodic, channel_immediate_multi,
    allocation
};

kukumba!(

    // TODO merge this into test_02_configure.rs
    // allow full reconfiguration only before vesting has started
    // (or before anyone has claimed from that pool?)

    #[reconfigure]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let s: Schedule = serde_json::from_str(include_str!("../../config_msg.json")).unwrap();
        test_tx!(deps, ADMIN, 0, 0;
            Configure { schedule: s.clone() } => tx_ok!());
    }

    //when "the admin tries to change the number, names, or ordering of pools"
    //then "it is not possible"{
        //todo!();
    //}
    //when "the admin tries to change a pool"
    //and  "the changes would reduce someone's balance"
    //then "it is not possible" {
        //todo!();
    //}
    //when "the admin tries to change a pool"
    //and  "the changes don't reduce anyone's balance"
    //then "it is possible" {
        //todo!();
    //}
    //when "the admin tries to change allocations"
    //then "it is not possible"{
        //todo!();
    //}

);

