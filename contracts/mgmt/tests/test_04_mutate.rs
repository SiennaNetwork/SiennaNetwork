#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate sienna_mgmt;
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env};
use cosmwasm_std::{HumanAddr, Uint128};
use sienna_schedule::{Schedule};

kukumba! {

    #[no_unauthorized_mutate_before_launch]

    #[ok_add_user_to_pool_before_launch]

    #[no_add_user_to_full_pool_before_launch]

    #[no_unauthorized_mutate_after_launch]

    #[ok_add_user_to_pool_after_launch]

    #[no_add_user_to_full_pool_after_launch]

}
