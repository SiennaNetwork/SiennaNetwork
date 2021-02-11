#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128};
use sienna_mgmt::{PRELAUNCH, NOTHING};
use sienna_schedule::Schedule;

kukumba!(

    #[claim_as_user]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let s: Schedule = serde_json::from_str(include_str!("../../config_msg.json")).unwrap();
        test_tx!(deps, ADMIN, 0, 0;
            Configure { schedule: s.clone() } => tx_ok!());

        let founder_1 = HumanAddr::from("secret1TODO20xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let founder_2 = HumanAddr::from("secret1TODO21xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let founder_3 = HumanAddr::from("secret1TODO22xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    }
    when "the contract is not yet launched"
    and  "anyone tries to claim funds"
    then "they are denied" {
        for user in [&founder_1, &founder_2, &founder_3].iter() {
            test_tx!(deps, *user, 1, 1; Claim {} => tx_err!(PRELAUNCH));
        }
    }
    when "the contract is launched"
    then "tokens should be minted and minting should be disabled" {
        let t_launch = 2;
        test_tx!(deps, ADMIN, 2, t_launch;
            Launch {} => tx_ok_launch!(s.total));
    }

    when "Founder1 tries to claim funds before the cliff"
    then "they are denied" {
        let t_cliff = 15552000;
        test_tx!(deps, founder_1, 3, t_launch + 1;
            Claim {} => tx_err!(NOTHING));
        test_tx!(deps, founder_1, 4, t_launch + t_cliff - 1;
            Claim {} => tx_err!(NOTHING));
    }
    when "Founder1 claims funds right after the cliff"
    then "they receive 80000 SIENNA" {
        test_tx!(deps, founder_1, 5, t_launch + t_cliff;
            Claim {} => tx_ok_claim!(founder_1, SIENNA!(80000u128)));
    }
    when "Founder1 tries to claim funds before the next vesting"
    then "they are denied" {
        test_tx!(deps, founder_1, 6, t_launch + t_cliff + 3600;
            Claim {} => tx_err!(NOTHING));
    }
    when "Founder1 claims funds again after 1 day"
    then "they receive 1 vesting's worth of 1500 SIENNA" {
        test_tx!(deps, founder_1, 7, t_launch + t_cliff + 86400;
            Claim {} => tx_ok_claim!(founder_1, SIENNA!(1500u128)));
    }
    when "Founder1 claims funds again after 2 more days"
    then "they receive 2 vestings' worth of 3000 SIENNA" {
        test_tx!(deps, founder_1, 8, t_launch + t_cliff + 86400 + 86400 * 2;
            Claim {} => tx_ok_claim!(founder_1, SIENNA!(3000u128)));
    }

    when "Founder2 tries to claim funds before the cliff"
    then "they are denied" {
        test_tx!(deps, founder_2, 9, t_launch + t_cliff - 1000;
            Claim {} => tx_err!(NOTHING));
    }
    when "Founder2 claims funds for the 1st time 10 days after the cliff"
    then "they receive cliff 80000 + 10 vestings' worth of 15000 = 95000 SIENNA" {
        test_tx!(deps, founder_2, 10, t_launch + t_cliff + 10 * 86400;
            Claim {} => tx_ok_claim!(founder_2, SIENNA!(95000u128)));
    }
    when "Founder 3 claims funds 500 days after the cliff"
    then "they receive the full amount of 731000 SIENNA" {
        test_tx!(deps, founder_3, 11, t_launch + t_cliff + 500 * 86400;
            Claim {} => tx_ok_claim!(founder_3, SIENNA!(731000u128)));
    }

);
