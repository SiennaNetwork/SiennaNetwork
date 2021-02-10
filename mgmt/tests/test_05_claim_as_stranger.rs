#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128};
use secret_toolkit::snip20::handle::{mint_msg, transfer_msg};
use sienna_mgmt::{PRELAUNCH, NOTHING, msg::Handle};
use sienna_schedule::{Schedule,Pool,Account,Allocation};

kukumba!(

    #[claim_as_stranger]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps, MALLORY, 1, 1;
            Handle::Claim {} => tx_err!(PRELAUNCH));
    }

    given "the contract is launched" {
        let s = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps, ALICE, 0, 0;
            Handle::Configure { schedule: s.clone() } => tx_ok!());
        test_tx!(deps, ALICE, 2, 2;
            Handle::Launch {} => tx_ok!(mint_msg(
                HumanAddr::from("mgmt"),
                Uint128::from(s.total),
                None, 256, String::new(), HumanAddr::from("token")
            ).unwrap()));
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps, MALLORY, 4, 4;
            Handle::Claim {} => tx_err!(NOTHING));
    }

);
