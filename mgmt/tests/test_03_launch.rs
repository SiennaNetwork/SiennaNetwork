#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
use mgmt::{DAY, MONTH, ONE_SIENNA};

use cosmwasm_std::{StdError, HumanAddr, Uint128};

use secret_toolkit::snip20;

kukumba!(

    #[launch]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }

    when "a stranger tries to start the vesting"
    then "they should fail" {
        let time = 2;
        assert_tx!(deps
            => from [MALLORY] at [block 4, T=4]
            => mgmt::msg::Handle::Launch {}
            => Err(StdError::Unauthorized { backtrace: None }));
        assert_query!(deps; Status; Status { launched: None, errors: 1 });
    }

    when "the admin tries to start the vesting"
    then "the contract should remember that moment" {
        let time = 3;
        let _ = tx(&mut deps, mock_env(1, time, &ALICE),
            mgmt::msg::Handle::Launch {});
        assert_query!(deps; Status; Status { launched: Some(time), errors: 1 });
    }

    given "the contract is already launched"
    when "the admin tries to start the vesting"
    then "the contract should say it's already launched"
    and "it should not update its launch date" {
        let next_time = 4;
        let _ = tx(&mut deps,
            mock_env(3, next_time, &ALICE),
            mgmt::msg::Handle::Launch {});
        assert_query!(deps; Status; Status { launched: Some(time), errors: 2 });
    }

);
