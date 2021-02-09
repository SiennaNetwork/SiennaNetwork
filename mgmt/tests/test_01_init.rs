#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::HumanAddr;

kukumba!(

    #[init]

    given "the contract is not yet deployed" {
        harness!(deps; ALICE);
    }

    when "someone deploys the contract" {
        use sienna_mgmt::{init, msg::Init};
        let _ = init(
            &mut deps,
            mock_env(0, 0, &ALICE),
            Init {
                token_addr: HumanAddr::from("mgmt"),
                token_hash: String::new(),
                schedule:   None
            }
        ).unwrap();
    }

    then "they become admin"
    and  "they should be able to query its state"
    and  "it should say it's not launched yet" {
        assert_query!(deps; Status; Status { launched: None, errors: 0 });
    }

);

