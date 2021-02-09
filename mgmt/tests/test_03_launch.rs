#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
use mgmt::msg::Handle;

use cosmwasm_std::{StdError, HumanAddr, Uint128};

use secret_toolkit::snip20;

kukumba!(

    #[launch]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }

    when "a stranger tries to start the vesting"
    then "they fail" {
        test_tx!(deps, MALLORY, 2, 2;
            Handle::Launch {} => tx_err_auth!());
        test_q!(deps, Status;
            Status { launched: None, errors: 1 });
    }

    when "the admin starts the vesting"
    then "the contract mints the tokens"
    and  "it stores the current time as its launch date" {
        test_tx!(deps, ALICE, 3, 3;
            Handle::Launch {} => tx_ok!());
        test_q!(deps, Status;
            Status { launched: Some(3), errors: 1 });
    }

    given "the contract is already launched"
    when "the admin tries to start the vesting again"
    then "the contract says it's already launched"
    and "it does not mint tokens"
    and "it does not update its launch date" {
        test_tx!(deps, ALICE, 4, 4;
            Handle::Launch {} => tx_err!("already underway"));
        test_q!(deps, Status;
            Status { launched: Some(3), errors: 2 });
    }

);
