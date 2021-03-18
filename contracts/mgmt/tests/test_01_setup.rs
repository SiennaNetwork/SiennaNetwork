#![cfg(test)]
#![allow(non_snake_case)]
#[macro_use] extern crate sienna_mgmt;
extern crate sienna_schedule; use sienna_schedule::{Schedule, Pool, Account};
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

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
                schedule:   sienna_schedule::Schedule::new(&[]),
                token_addr: cosmwasm_std::HumanAddr::from("token"),
                token_hash: String::new(),
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

    #[configure]

    given "a contract" {
        harness!(deps; ALICE, BOB, MALLORY);
    }
    when "anyone but the admin tries to set a configuration"
    then "that fails" {
        for sender in [&BOB, &MALLORY].iter() {
            test_tx!(deps, sender.clone(), 0, 0;
                Configure { schedule: Schedule::new(&[]) }
                => tx_err_auth!());
        }
    }
    when "the admin sets a minimal valid configuration" {
        let s0 = Schedule::new(&[])
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s0.clone() } => tx_ok!());
    } then "the configuration is updated" {
    }
    when "someone else tries to set a valid configuration" {
        let sX = Schedule::new(&[Pool::full("P0", &[Account::immediate("Mallory", &MALLORY.clone(), 1)])]);
        test_tx!(deps, MALLORY, 0, 0; Configure { schedule: sX.clone() } => tx_err_auth!());
    } then "the configuration remains unchanged" {
        test_q!(deps, GetSchedule; Schedule { schedule: s0 });
    }
    when "the admin sets the planned production configuration" {
        let s: Schedule = serde_json::from_str(include_str!("../../../settings/schedule.json")).unwrap();
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s.clone() } => tx_ok!());
    } then "the configuration is updated" {
        test_q!(deps, GetSchedule; Schedule { schedule: s.clone() });
    }
    when "the contract launches" {
        test_tx!(deps, ALICE, 0, 0; Launch {} => tx_ok_launch!(s.total));
    } then "the configuration can't be changed anymore" {
        test_tx!(deps, ALICE,   0, 0; Configure { schedule: s.clone() } => tx_err!(UNDERWAY));
        test_tx!(deps, BOB,     0, 0; Configure { schedule: s.clone() } => tx_err_auth!());
        test_tx!(deps, MALLORY, 0, 0; Configure { schedule: s.clone() } => tx_err_auth!());
    }

    #[launch]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }
    when "a stranger tries to start the vesting"
    then "that fails" {
        test_tx!(deps, MALLORY, 2, 2; Launch {} => tx_err_auth!());
        test_q!(deps, Status; Status { launched: None, errors: 1 });
    }
    when "the contract is configured"
    and  "the admin starts the vesting"
    then "the contract mints the tokens"
    and  "the current time is remembered as the launch date" {
        let s = sienna_schedule::Schedule::new(&[]);
        test_tx!(deps, ALICE, 3, 3; Configure { schedule: s.clone() } => tx_ok!());
        test_tx!(deps, ALICE, 4, 4; Launch {} => tx_ok_launch!(s.total));
        test_q!(deps, Status; Status { launched: Some(4), errors: 1 });
    }
    given "the contract is already launched"
    when "the admin tries to start the vesting again"
    then "the contract says it's already launched"
    and "it does not mint tokens"
    and "it does not update its launch date" {
        test_tx!(deps, ALICE, 5, 5; Launch {} => tx_err!(UNDERWAY));
        test_q!(deps, Status; Status { launched: Some(4), errors: 2 });
    }


);

