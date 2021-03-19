#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate sienna_mgmt;
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env};
use cosmwasm_std::{HumanAddr, Uint128};
use sienna_schedule::{Schedule};

/// Add 18 zeroes and make serializable
macro_rules! SIENNA {
    ($x:expr) => { Uint128::from($x as u128 * sienna_schedule::ONE_SIENNA) }
}

kukumba!(

    #[claim_as_stranger]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
        let PRELAUNCH = MGMTError!(PRELAUNCH);
        let NOTHING   = MGMTError!(NOTHING);
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps; MALLORY, 1, 1; Claim {} == err!(PRELAUNCH));
    }

    given "the contract is launched" {
        let s = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps; ALICE, 0, 0; Configure { schedule: s.clone() } == ok!());
        test_tx!(deps; ALICE, 2, 2; Launch {} == ok!(launched: s.total));
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps; MALLORY, 4, 4; Claim {} == err!(NOTHING));
    }

    #[claim_as_user]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let s: Schedule = serde_json::from_str(include_str!("../../../settings/schedule.json")).unwrap();
        test_tx!(deps; ADMIN, 0, 0; Configure { schedule: s.clone() } == ok!());

        let founder_1 = HumanAddr::from("secret1TODO20xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let founder_2 = HumanAddr::from("secret1TODO21xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let founder_3 = HumanAddr::from("secret1TODO22xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let PRELAUNCH = MGMTError!(PRELAUNCH);
        let NOTHING   = MGMTError!(NOTHING);
    }
    when "the contract is not yet launched"
    and  "anyone tries to claim funds"
    then "they are denied" {
        for user in [&founder_1, &founder_2, &founder_3].iter() {
            test_tx!(deps; *user, 1, 1; Claim {} == err!(PRELAUNCH));
        }
    }
    when "the contract is launched"
    then "tokens should be minted and minting should be disabled" {
        let t_launch = 2;
        test_tx!(deps; ADMIN, 2, t_launch; Launch {} == ok!(launched: s.total));
    }
    and "the appropriate amounts will be unlocked at the appropriate times" {
        for pool in s.pools.iter() {
            for account in pool.accounts.iter() {
                println!("\n{:?}\n{} + {} * {} + {} = {}",
                    &account,
                    account.cliff,
                    account.portion_count(),
                    account.portion_size(),
                    account.remainder(),
                    account.amount
                );
                let address = account.address.clone();

                // test that funds are not unlocked before `start_at`
                if account.start_at > 0 {
                    test_q!(deps; Progress {
                        address: address.clone(),
                        time:    account.start_at - 1
                    } == Progress {
                        address:  address.clone(),
                        unlocked: Uint128::zero(),
                        claimed:  Uint128::zero()
                    });
                }

                // test cliff or first portion
                if account.cliff > Uint128::zero() {
                    test_q!(deps; Progress {
                        address: address.clone(),
                        time:    account.start_at
                    } == Progress {
                        address:  address.clone(),
                        unlocked: account.cliff,
                        claimed:  Uint128::zero()
                    });
                } else {
                    test_q!(deps; Progress {
                        address: address.clone(),
                        time:    account.start_at
                    } == Progress {
                        address:  address.clone(),
                        unlocked: Uint128::from(account.portion_size()),
                        claimed:  Uint128::zero()
                    });
                }

                // test that entire amount is vested by the end
                test_q!(deps; Progress {
                    address: address.clone(),
                    time:    account.start_at + account.duration
                } == Progress {
                    address:  address.clone(),
                    unlocked: account.amount,
                    claimed:  Uint128::zero()
                });
            }
        }
    }
    then "they are denied" {
        let t_cliff = 15552000;
        test_tx!(deps; founder_1, 3, t_launch + 1; Claim {} == err!(NOTHING));
        test_tx!(deps; founder_1, 4, t_launch + t_cliff - 1; Claim {} == err!(NOTHING));
    }
    when "Founder1 claims funds right after the cliff"
    then "they receive 80000 SIENNA" {
        test_tx!(deps; founder_1, 5, t_launch + t_cliff; Claim {} ==
            ok!(claimed: founder_1, SIENNA!(80000u128)));
    }
    when "Founder1 tries to claim funds before the next vesting"
    then "they are denied" {
        test_tx!(deps; founder_1, 6, t_launch + t_cliff + 3600; Claim {} == err!(NOTHING));
    }
    when "Founder1 claims funds again after 1 day"
    then "they receive 1 vesting's worth of 1500 SIENNA" {
        test_tx!(deps; founder_1, 7, t_launch + t_cliff + 86400; Claim {} ==
            ok!(claimed: founder_1, SIENNA!(1500u128)));
    }
    when "Founder1 claims funds again after 2 more days"
    then "they receive 2 vestings' worth of 3000 SIENNA" {
        test_tx!(deps; founder_1, 8, t_launch + t_cliff + 86400 + 86400 * 2; Claim {} ==
            ok!(claimed: founder_1, SIENNA!(3000u128)));
    }

    when "Founder2 tries to claim funds before the cliff"
    then "they are denied" {
        test_tx!(deps; founder_2, 9, t_launch + t_cliff - 1000; Claim {} == err!(NOTHING));
    }
    when "Founder2 claims funds for the 1st time 10 days after the cliff"
    then "they receive cliff 80000 + 10 vestings' worth of 15000 = 95000 SIENNA" {
        test_tx!(deps; founder_2, 10, t_launch + t_cliff + 10 * 86400; Claim {} ==
            ok!(claimed: founder_2, SIENNA!(95000u128)));
    }
    when "Founder 3 claims funds 500 days after the cliff"
    then "they receive the full amount of 731000 SIENNA" {
        test_tx!(deps; founder_3, 11, t_launch + t_cliff + 500 * 86400; Claim {} ==
            ok!(claimed: founder_3, SIENNA!(731000u128)));
    }

);
