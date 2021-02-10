#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128, HandleResponse};
use sienna_mgmt::msg::Handle;
use sienna_schedule::{
    Schedule,
    schedule, pool, pool_partial,
    channel_periodic, channel_immediate_multi,
    allocation_addr
};

kukumba!(

    #[reconfigure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "the admin tries to remove a pool"
    then "it is not possible"{
        todo!();
    }
    when "the admin tries to change a pool"
    and  "the changes would reduce someone's balance"
    then "it is not possible" {
        todo!();
    }
    when "the admin tries to change a pool"
    and  "the changes don't reduce anyone's balance"
    then "it is possible" {
        todo!();
    }
    when "the admin tries to change allocations"
    then "it is not possible"{
        todo!();
    }

);

