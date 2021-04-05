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

kukumba! {

    #[no_claim_as_stranger]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
        let PRELAUNCH = MGMTError!(PRELAUNCH);
        let NOTHING   = MGMTError!(NOTHING);
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        tx!(deps; MALLORY, 1, 1; Claim {} == err!(PRELAUNCH));
    }

    given "the contract is launched" {
        let s = Schedule { total: Uint128::from(0u128), pools: vec![] }
        tx!(deps; ALICE, 0, 0; Configure { schedule: s.clone() } == ok!());
        tx!(deps; ALICE, 2, 2; Launch {} == ok!(launched: s.total));
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        tx!(deps; MALLORY, 4, 4; Claim {} == err!(NOTHING));
    }

    #[ok_claim_as_user]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let s: Schedule = serde_json::from_str(include_str!("../../../settings/schedule.json")).unwrap();
        tx!(deps; ADMIN, 0, 0; Configure { schedule: s.clone() } == ok!());

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
            tx!(deps; *user, 1, 1; Claim {} == err!(PRELAUNCH));
        }
    }
    when "the contract is launched"
    then "tokens should be minted and minting should be disabled" {
        let t_launch = 2;
        tx!(deps; ADMIN, 2, t_launch; Launch {} == ok!(launched: s.total));
    }
    and "the appropriate amounts will be unlocked at the appropriate times" {
        let zero = Uint128::zero();
        for P in s.pools.iter() {
            for A in P.accounts.iter() {
                let address       = A.address.clone();
                let portion_count = A.portion_count();
                let portion_size  = A.portion_size();
                let remainder     = A.remainder();
                println!("\naccount: {} {} {} {}",
                    &A.name, A.start_at, A.interval, A.duration);
                println!("amounts: {} = {} + {} * {} + {}",
                    A.amount, A.cliff, portion_count, portion_size, remainder);
                assert_eq!(
                    A.cliff.u128() + portion_count as u128 * portion_size + remainder,
                    A.amount.u128(),
                    "(cliff + portions + remainder) should equal account total");
                if A.start_at > 0 { //funds are not unlocked before `start_at`
                    q!(deps;
                        Progress { address: address.clone(), time: A.start_at - 1 } ==
                        Progress { unlocked: zero, claimed: zero });
                }
                if A.cliff > zero { // cliff
                    q!(deps;
                        Progress { address: address.clone(), time: A.start_at } ==
                        Progress { unlocked: A.cliff, claimed: zero });
                } else { // first portion
                    q!(deps;
                        Progress { address: address.clone(), time: A.start_at } ==
                        Progress { unlocked: Uint128::from(A.portion_size()), claimed: zero });
                }
                q!(deps; // entire amount is unlocked by the end
                    Progress { address: address.clone(), time: A.start_at + A.duration } ==
                    Progress { unlocked: A.amount, claimed: zero });
            }
        }
    }
    and "by the end of the contract everyone will have unlocked exactly their assigned amount" {
        for P in s.pools.iter() {
            for A in P.accounts.iter() {
                tx!(deps; A.address, A.end() / 5, A.end();
                    Claim {} == ok!(claimed: A.address, A.amount));
            }
        }
    }

}
