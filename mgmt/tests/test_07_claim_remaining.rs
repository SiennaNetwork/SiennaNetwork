#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128};
use secret_toolkit::snip20::handle::{mint_msg, set_minters_msg, transfer_msg};
use sienna_mgmt::{PRELAUNCH, NOTHING, msg::Handle};
use sienna_schedule::Schedule;

kukumba!(

    #[claim_remaining_pool_tokens]

);
