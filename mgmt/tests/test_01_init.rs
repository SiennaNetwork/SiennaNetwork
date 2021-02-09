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
    and  "if someone queries its state"
    and  "it says the contract is not launched" {
        test_q!(deps, Status; Status {
            launched: None,
            errors: 0
        });
    }

);

