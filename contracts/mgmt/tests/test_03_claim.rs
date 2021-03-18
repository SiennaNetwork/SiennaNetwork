#![allow(non_snake_case)]
#[macro_use] extern crate sienna_mgmt;
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};
use cosmwasm_std::{HumanAddr, Uint128};
use sienna_schedule::{Schedule};

kukumba!(

    #[claim_as_stranger]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps, MALLORY, 1, 1; Claim {} => tx_err!(PRELAUNCH));
    }

    given "the contract is launched" {
        let s = Schedule { total: Uint128::from(0u128), pools: vec![] }
        test_tx!(deps, ALICE, 0, 0; Configure { schedule: s.clone() } => tx_ok!());
        test_tx!(deps, ALICE, 2, 2; Launch {} => tx_ok_launch!(s.total));
    }
    when "a stranger tries to claim funds"
    then "they are denied" {
        test_tx!(deps, MALLORY, 4, 4; Claim {} => tx_err!(NOTHING));
    }

    #[claim_as_user]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let s: Schedule = serde_json::from_str(include_str!("../../../settings/schedule.json")).unwrap();
        test_tx!(deps, ADMIN, 0, 0; Configure { schedule: s.clone() } => tx_ok!());

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
        test_tx!(deps, ADMIN, 2, t_launch; Launch {} => tx_ok_launch!(s.total));
    }

    when "Founder1 tries to claim funds before the cliff"
    then "they are denied" {
        let t_cliff = 15552000;
        test_tx!(deps, founder_1, 3, t_launch + 1; Claim {} => tx_err!(NOTHING));
        test_tx!(deps, founder_1, 4, t_launch + t_cliff - 1; Claim {} => tx_err!(NOTHING));
    }
    when "Founder1 claims funds right after the cliff"
    then "they receive 80000 SIENNA" {
        test_tx!(deps, founder_1, 5, t_launch + t_cliff; Claim {} =>
            tx_ok_claim!(founder_1, SIENNA!(80000u128)));
    }
    when "Founder1 tries to claim funds before the next vesting"
    then "they are denied" {
        test_tx!(deps, founder_1, 6, t_launch + t_cliff + 3600; Claim {} => tx_err!(NOTHING));
    }
    when "Founder1 claims funds again after 1 day"
    then "they receive 1 vesting's worth of 1500 SIENNA" {
        test_tx!(deps, founder_1, 7, t_launch + t_cliff + 86400; Claim {} =>
            tx_ok_claim!(founder_1, SIENNA!(1500u128)));
    }
    when "Founder1 claims funds again after 2 more days"
    then "they receive 2 vestings' worth of 3000 SIENNA" {
        test_tx!(deps, founder_1, 8, t_launch + t_cliff + 86400 + 86400 * 2; Claim {} =>
            tx_ok_claim!(founder_1, SIENNA!(3000u128)));
    }

    when "Founder2 tries to claim funds before the cliff"
    then "they are denied" {
        test_tx!(deps, founder_2, 9, t_launch + t_cliff - 1000; Claim {} => tx_err!(NOTHING));
    }
    when "Founder2 claims funds for the 1st time 10 days after the cliff"
    then "they receive cliff 80000 + 10 vestings' worth of 15000 = 95000 SIENNA" {
        test_tx!(deps, founder_2, 10, t_launch + t_cliff + 10 * 86400; Claim {} =>
            tx_ok_claim!(founder_2, SIENNA!(95000u128)));
    }
    when "Founder 3 claims funds 500 days after the cliff"
    then "they receive the full amount of 731000 SIENNA" {
        test_tx!(deps, founder_3, 11, t_launch + t_cliff + 500 * 86400; Claim {} =>
            tx_ok_claim!(founder_3, SIENNA!(731000u128)));
    }

    #[claim_minting_pool]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let s: Schedule = serde_json::from_str(include_str!("../../../settings/schedule.json")).unwrap();
        test_tx!(deps, ADMIN, 0, 0; Configure { schedule: s.clone() } => tx_ok!());

        let lpf  = HumanAddr::from("secret1TODO50xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let rem1 = HumanAddr::from("secret1TODO51xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let rem2 = HumanAddr::from("secret1TODO52xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let rem3 = HumanAddr::from("secret1TODO53xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    }

    when "the liquidity provision fund is claimed before launch"
    then "it is not transferred" {
        test_tx!(deps, lpf, 1, 1; Claim {} => tx_err!(PRELAUNCH));
    }
    when "the remaining pool tokens are claimed before launch"
    then "they are not transferred" {
        for rem in [&rem1, &rem2, &rem3].iter() {
            test_tx!(deps, *rem, 1, 1; Claim {} => tx_err!(PRELAUNCH));
        }
    }

    when "the contract is launched" {
        let t_launch = 2*86400u64;
        test_tx!(deps, ADMIN, 2, t_launch; Launch {} => tx_ok_launch!(s.total));
    }
    and "the liquidity provision fund is claimed"
    then "it is received in full" {
        test_tx!(deps, lpf, 3, t_launch + 1; Claim {} => tx_ok_claim!(lpf, SIENNA!(300000u128)));
    }
    and "the liquidity provision fund is claimed again"
    then "nothing more is transfered" {
        test_tx!(deps, lpf, 3, t_launch + 1; Claim {} => tx_err!(NOTHING));
    }

    when "the remaining pool tokens are claimed"
    then "the corresponding portion of them is transferred" {
        println!("{:#?}", &s)
        test_tx!(deps, rem1, 3, t_launch + 1; Claim {} => tx_ok_claim!(rem1, SIENNA!(1250u128)));
        test_tx!(deps, rem2, 4, t_launch + 86400; Claim {} => tx_ok_claim!(rem2, SIENNA!(2*750u128)));
    }

    //when "the remaining pool tokens are claimed late"
    //then "the corresponding portion of them is transferred" {
        //test_tx!(deps, rem3, 6, t_launch + 86400 + 3*86400;
            //Claim {} => tx_ok_claim!(rem1, SIENNA!(4*1500u128)));
    //}

    //when "more remaining pool tokens are claimed"
    //then "the corresponding portion of them is transferred" {
        //test_tx!(deps, rem1, 6, t_launch + 86400 + 4*86400;
            //Claim {} => tx_ok_claim!(rem1, SIENNA!(2500u128)));
        //test_tx!(deps, rem2, 7, t_launch + 86400 + 4*86400 + 1;
            //Claim {} => tx_ok_claim!(rem2, SIENNA!(1500u128)));
        //test_tx!(deps, rem3, 8, t_launch + 86400 + 4*86400 + 2;
            //Claim {} => tx_ok_claim!(rem3, SIENNA!(500u128)));
    //}

    //when "the allocations of remaining pool tokens are changed"
    //then "subsequent claims use the new allocations" {
        //test_tx!(deps, ADMIN, 9, t_launch + 4*86400 + 3;
            //Reallocate {
                //pool_name: "Minting Pool".to_string(),
                //channel_name: "PoolRem".to_string(),
                //allocations: vec![
                    //allocation( 900, &rem1),
                    //allocation(1000, &rem2),
                    //allocation(1100, &rem3)
                //] } => tx_ok!());
        //test_tx!(deps, rem3, 6, t_launch + 86400 + 5*86400;
            //Claim {} => tx_ok_claim!(rem3, SIENNA!(1100u128)));
        //test_tx!(deps, rem2, 6, t_launch + 86400 + 5*86400 + 10;
            //Claim {} => tx_ok_claim!(rem2, SIENNA!(1000u128)));
        //test_tx!(deps, rem1, 6, t_launch + 86400 + 5*86400 + 20;
            //Claim {} => tx_ok_claim!(rem1, SIENNA!( 900u128)));
    //}



);
