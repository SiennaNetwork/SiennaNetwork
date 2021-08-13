#![allow(unused_macros)]
#![allow(non_snake_case)]

use fadroma::scrt::{cosmwasm_std::{HumanAddr, StdError}};
use crate::{rewards_harness::*};

const PORTION: u128 = 100;

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
const DAY: u128 = crate::DAY as u128;

kukumba_harnessed! {
    type Error = StdError;

    let Test: RewardsHarness<RewardsMockQuerier>;

    ok_pool_init {
        given "no instance"
        when  "admin inits with an asset token address"
        then  "the instance configures a viewing key for itself" {
            let admin = HumanAddr::from("admin");
            Test.at(1).init_configured(&admin)? }

        when  "admin locks funds"
        then  "the instance starts counting the liquidity that accumulates" {
            Test.at(1).pool(0, 0, 1)?
                .at(2).pool(0, 0, 2)?
                .at(3).pool(0, 0, 3)?
                .at(4).pool(0, 0, 4)?.lock(&admin, 1)?.pool(1, 0, 4)?
                .at(5).pool(1, 1, 4)?
                .at(6).pool(1, 2, 4)? } }

    ok_pool_init_then_set_lp_token {

        given  "no instance"
        when  "admin inits without providing an asset token address"
        then  "the instance is not ready" {
            let admin  = HumanAddr::from("admin");
            let badman = HumanAddr::from("badman");
            Test.at(1).init_partial(&admin)? }

        when  "badman tries to provide an asset token address"
        then  "an error is returned and nothing changes" {
            Test.at(2).set_token_fails(&badman, "bad_addr", "bad_hash")? }

        when  "admin provides an asset token address"
        then  "the instance configures a viewing key for itself"
        and   "it starts counting when someone locks funds" {
            Test.at(4).set_token(&admin, "lp_token_address", "lp_token_hash")?
                .at(7).pool(0, 0, 7)?.lock(&admin, 1)?.pool(1, 0, 7)?
                .at(8).pool(1, 1, 7)?
                .at(9).pool(1, 2, 7)? } }

    ok_one {

        given "an instance:" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            Test.at(0).init_configured(&admin)?
                      .set_vk(&alice, "")?
                      .fund(PORTION)
                      .user(&alice,    0,   0,   0, 0, 0, 0)? }

        when "alice first locks lp tokens,"
        then "alice's age and lifetime share starts incrementing;" {
            Test.at( 1).user(&alice,   0,   0,   0, 0, 0, 0)?.lock(&alice, 100)?
                       .user(&alice,   0, 100,   0, 0, 0, 0)?
                .at( 2).user(&alice,   1, 100, 100, 0, 0, 0)?
                .at( 3).user(&alice,   2, 100, 200, 0, 0, 0)? }

        when "alice retrieves half of the tokens,"
        then "alice's age keeps incrementing;" {
            Test.at( 4).user(&alice,   3, 100, 300, 0, 0, 0)?.retrieve(&alice, 50)?
                       .user(&alice,   3,  50, 300, 0, 0, 0)? }

        when "alice retrieves all of the tokens,"
        then "alice's age stops incrementing;" {
            Test.at( 5).user(&alice,   4,  50, 350, 0, 0, 0)?
                .at( 6).user(&alice,   5,  50, 400, 0, 0, 0)?.retrieve(&alice, 50)?
                       .user(&alice,   5,   0, 400, 0, 0, 0)?
                .at( 7).user(&alice,   5,   0, 400, 0, 0, 0)?
                .at( 8).user(&alice,   5,   0, 400, 0, 0, 0)? }

        when "alice locks tokens again,"
        then "alice's age resumes incrementing;" {
            Test.at( 9).user(&alice,   5,   0, 400, 0, 0, 0)?.lock(&alice,   1)?
                       .user(&alice,   5,   1, 400, 0, 0, 0)?
                .at(10).user(&alice,   6,   1, 401, 0, 0, 0)?
                .at(11).user(&alice,   7,   1, 402, 0, 0, 0)? }

        when "alice's age reaches the configured threshold,"
        then "alice is eligible to claim the whole pool" {
            Test.at(DAY+4).user(&alice, DAY, 1, 17675, 100, 0, 100)? } }

    ok_two_simultaneous {

        given "an instance:" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(0)
             .init_configured(&admin)?
             .fund(PORTION)
             .set_vk(&alice, "")?
             .set_vk(&bob,   "")?
             .user(&alice, 0, 0, 0, 0, 0, 0)?
             .user(&bob,   0, 0, 0, 0, 0, 0)? }

        when "alice and bob first lock lp tokens simultaneously,"
        then "their ages start incrementing simultaneously;" {
            Test.at(1).user(&alice, 0,   0,   0, 0, 0, 0)?.lock(&alice, 100)?
                      .user(&bob,   0,   0,   0, 0, 0, 0)?.lock(&bob,   100)?
                      .user(&alice, 0, 100,   0, 0, 0, 0)?
                      .user(&bob,   0, 100,   0, 0, 0, 0)?
                .at(2).user(&alice, 1, 100, 100, 0, 0, 0)?
                      .user(&bob,   1, 100, 100, 0, 0, 0)?
                .at(3).user(&alice, 2, 100, 200, 0, 0, 0)?
                      .user(&bob,   2, 100, 200, 0, 0, 0)? }

        when "alice and bob's ages reach the configured threshold,"
        then "each is eligible to claim half of the pool" {
            Test.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 50, 0, 50)?
                          .user(&bob,   DAY, 100, DAY * 100, 50, 0, 50)? } }

    ok_two_sequential {

        given "an instance" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(0).init_configured(&admin)?
                      .set_vk(&alice, "")?
                      .set_vk(&bob,   "")?
                      .fund(PORTION) }

        when "alice locks lp tokens,"
        and  "alice retrieves them after reaching the threshold;"
        then "alice is eligible to claim the whole pool" {
            Test.at(    1).user(&alice,   0,   0,         0,   0, 0,   0)?.lock(&alice, 100)?
                .at(DAY+1).user(&alice, DAY, 100, DAY * 100, 100, 0, 100)?.retrieve(&alice, 100)?
                          .user(&alice, DAY,   0, DAY * 100, 100, 0, 100)? }

        when "bob locks the same amount of tokens"
        then "alice's rewards start decreasing proportionally" {
            Test.at(           DAY+2).user(&bob,     0,   0,         0,  0, 0,  0)?.lock(&bob, 100)?
                                     .user(&bob,     0, 100,         0,  0, 0,  0)?
                .at(         DAY+2+1).user(&alice, DAY,   0, DAY * 100, 99, 0, 99)?
                .at(     DAY+2+DAY/2).user(&alice, DAY,   0, DAY * 100, 66, 0, 66)?
                .at(DAY+2+DAY/2+1000).user(&alice, DAY,   0, DAY * 100, 64, 0, 64)? }

        when "bob reaches the age threshold"
        then "each is eligible to claim half of the pool" {
            Test.at(         2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 50, 0, 50)?.retrieve(&bob, 100)?
                                     .user(&bob,   DAY,   0, DAY * 100, 50, 0, 50)?
                                     .user(&alice, DAY,   0, DAY * 100, 50, 0, 50)? } }

    //#[ok_two_sequential_with_claim]
    //given "an instance" {
        //let mut T = RewardsHarness::new();
        //let admin = HumanAddr::from("admin");
        //let alice = HumanAddr::from("alice");
        //let bob   = HumanAddr::from("bob");
        //let _ = T.init_configured(0, &admin)?;
        //let _ = T.tx_set_vk(0, &alice, "")?;
        //let _ = T.tx_set_vk(0, &bob,   "")?; }
    //when "alice locks lp tokens,"
    //and  "alice retrieves them after reaching the threshold;"
    //then "alice is eligible to claim the whole pool" {
        //T = T.fund(PORTION)
        //     .at(1).user(&alice)     -> { age:   0, locked:   0, lifetime:         0, earned:   0, claimed: 0, claimable:   0 } }
        //     .at(1 ; alice locks 100 -> [ Snip20::transfer_from("alice", "contract_addr", "100") ] }
        //     .at(DAY+1).user(&alice)     -> { age: DAY, locked: 100, lifetime: DAY * 100, earned: 100, claimed: 0, claimable: 100 } }
        //     .at(DAY+1 ; alice retrieves 100 -> [ Snip20::transfer("alice", "100") ] }
        //     .at(DAY+1).user(&alice)     -> { age: DAY, locked:   0, lifetime: DAY * 100, earned: 100, claimed: 0, claimable: 100 } } }
    //when "bob locks the same amount of tokens" {
        //     .at(DAY+2).user(&bob)     -> { age:   0, locked:   0, lifetime:         0, earned:   0, claimed: 0, claimable:   0 } }
        //     .at(DAY+2 ; bob locks 100 -> [ Snip20::transfer_from("bob", "contract_addr", "100") ] }
        //     .at(DAY+2).user(&bob)     -> { age:   0, locked: 100, lifetime:         0, earned:   0, claimed: 0, claimable:   0 } } }
    //then "alice's rewards start decreasing proportionally" {
        //     .at(DAY+2+1).user(&alice, age: DAY, locked:   0, lifetime: DAY * 100, earned:  99, claimed: 0, claimable:   99 } } }
    //when "alice claims some time after maturing" {
        //test!(T = DAY+2+DAY/2).user(&alice)      -> {
            //age: DAY, locked:   0, lifetime: DAY * 100, earned:  66, claimed: 0, claimable:   66 });
        //test!(T = DAY+2+DAY/2 ; alice claims
            //-> [ Snip20::transfer("alice", "66") ]);
        //test!(T = DAY+2+DAY/2+1000).user(&alice,
            //age: DAY, locked:   0, lifetime: DAY * 100, earned:  64, claimed: 66, claimable:   0 }); }
    //when "bob reaches the age threshold"
    //then "each is eligible to claim half of the pool" {
        //     .at(2*DAY+2).user(&bob, age: DAY, locked: 100, lifetime: DAY * 100, earned:  50, claimed:  0, claimable:  50 } }
        //test!(T = 2*DAY+2 ; bob retrieves 100
            //-> [ Snip20::transfer("bob", "100") ]);
        //     .at(2*DAY+2).user(&bob)   -> { age: DAY, locked:   0, lifetime: DAY * 100, earned:  50, claimed:  0, claimable:  50 } }
        //     .at(2*DAY+2).user(&alice, age: DAY, locked:   0, lifetime: DAY * 100, earned:  50, claimed: 66, claimable:   0 } } }

    //#[ok_lock_and_retrieve]
    //given "an instance" {
        //let mut T = RewardsHarness::new();
        //let admin   = HumanAddr::from("admin");
        //let alice   = HumanAddr::from("alice");
        //let bob     = HumanAddr::from("bob");
        //let mallory = HumanAddr::from("mallory");
        //assert_eq!(T.init_configured(0, &admin)?, (vec![ Snip20::set_viewing_key("") ], 0, 0)); }
    //when  "someone requests to lock tokens"
    //then  "the instance transfers them to itself"
    //and   "the liquidity provider starts accruing a reward" {
        //     .at(1; alice locks 100
            //-> [ Snip20::transfer_from("alice", "contract_addr", "100") ] }
        //     .at(2; pool -> { locked: 100, lifetime: 100, updated: 1 } } }
    //when  "a provider requests to retrieve tokens"
    //then  "the instance transfers them to the provider"
    //and   "the reward now increases at a reduced rate" {
        //     .at(3; pool -> { locked: 100, lifetime: 200, updated: 1 } }
        //     .at(4; alice retrieves 50
            //-> [ Snip20::transfer("alice", "50") ] } }
    //when  "a provider requests to retrieve all their tokens"
    //then  "the instance transfers them to the provider"
    //and   "their reward stops increasing" {
        //     .at(5; pool -> { locked:  50, lifetime: 350, updated: 4 } }
        //     .at(5; alice retrieves 50
            //-> [ Snip20::transfer("alice", "50") ] }
        //     .at(6; pool -> { locked:   0, lifetime: 350, updated: 5 } } }
    //when  "someone else requests to lock tokens"
    //then  "the previous provider's share of the rewards begins to diminish" {
        //     .at(7; pool -> { locked:   0, lifetime: 350, updated: 5 } }
        //     .at(7; bob locks 500
            //-> [ Snip20::transfer_from("bob", "contract_addr", "500") ] }
        //     .at(8; pool -> { locked: 500, lifetime: 850, updated: 7 } } }
    //when  "a provider tries to retrieve too many tokens"
    //then  "they get an error" {
        //     .at(9; pool -> { locked: 500, lifetime: 1350, updated: 7 } }
        //assert_error!(T.tx_retrieve(9, &bob, 1000u128), "not enough locked (500 < 1000)");
        //     .at(10; pool -> { locked: 500, lifetime: 1850, updated: 7 } } }
    //when  "a stranger tries to retrieve any tokens"
    //then  "they get an error" {
        //assert_error!(T.tx_retrieve(10, &mallory, 100u128), "not enough locked (0 < 100)");
        //     .at(11; pool -> { locked: 500, lifetime: 2350, updated: 7 } } }

    //#[ok_claim]
    //given "an instance" {
        //let mut T = RewardsHarness::new();
        //let admin   = HumanAddr::from("admin");
        //let alice   = HumanAddr::from("alice");
        //let bob     = HumanAddr::from("bob");
        //let _ = T.init_configured(0, &admin)?; }
    //when  "strangers try to claim rewards"
    //then  "they get an error" {
        //assert_error!(T.tx_claim(1, &alice), "lock tokens for 17280 more blocks to be eligible");
        //assert_error!(T.tx_claim(1, &bob),   "lock tokens for 17280 more blocks to be eligible"); }
    //when  "users provide liquidity"
    //and   "they wait for rewards to accumulate" {
        //assert_eq!(T.tx_lock(2, &alice, 100)?, (vec![Snip20::transfer_from("alice", "contract_addr", "100")], 0, 0));
        //assert_error!(T.tx_claim(2, &alice), "lock tokens for 17280 more blocks to be eligible");
        //assert_eq!(T.tx_lock(2, &bob, 100)?, (vec![Snip20::transfer_from("bob", "contract_addr", "100")], 0, 0));
        //assert_error!(T.tx_claim(2, &alice), "lock tokens for 17280 more blocks to be eligible");
        //assert_error!(T.tx_claim(3, &bob),   "lock tokens for 17279 more blocks to be eligible");
        //assert_error!(T.tx_claim(4, &alice), "lock tokens for 17278 more blocks to be eligible");
        //assert_error!(T.tx_claim(5, &bob),   "lock tokens for 17277 more blocks to be eligible"); }
    //and   "a provider claims rewards"
    //then  "that provider receives reward tokens" {
        //T = T.fund(PORTION)
        //assert_eq!(T.tx_claim(17282, &alice)?, (vec![Snip20::transfer("alice", "50")], 0, 0)); }
    //when  "a provider claims rewards twice"
    //then  "rewards are sent only once" {
        //assert_error!(T.tx_claim(17282, &alice), "already claimed"); }
    //when  "a provider claims their rewards less often"
    //then  "they receive equivalent rewards as long as the liquidity locked hasn't changed" {
        ////assert_eq!(T.tx_claim(4, &alice)?, (vec![Snip20::transfer("alice",  "5")], 0, 0));
        //T = T.fund(PORTION)
        //assert_eq!(T.tx_claim(3 + DAY * 2, &alice)?, (vec![Snip20::transfer("alice", "50")], 0, 0));
        //assert_eq!(T.tx_claim(3 + DAY * 2, &bob)?,   (vec![Snip20::transfer("bob", "100")], 0, 0));
        ////println!("{:#?}", T.tx_claim(10, &alice));
        ////println!("{:#?}", T.tx_claim(4, &bob)?);
        ////panic!()
    //}

    //#[rewards_parallel_or_sequential]
    //given "three users providing liquidity" {
        //let admin   = HumanAddr::from("admin");
        //let alice   = HumanAddr::from("alice");
        //let bob     = HumanAddr::from("bob");
        //let cyril   = HumanAddr::from("cyril"); }
    //when "they provide the liquidity simultaneously" {
        //let mut T = RewardsHarness::new().fund(PORTION);
        //let _ = T.init_configured(0, &admin)?;
        //let _ = T.tx_set_vk(0, &alice, "")?;
        //let _ = T.tx_set_vk(0, &bob,   "")?;
        //let _ = T.tx_set_vk(0, &cyril, "")?;

        //let _ = T.tx_lock(0, &alice, 100)?;
        //let _ = T.tx_lock(0, &bob,   100)?;
        //let _ = T.tx_lock(0, &cyril, 100)?;
        ////println!("{:#?}", T.q_pool_info(0));
        //assert_eq!(T.tx_claim(DAY, &alice)?, (vec![Snip20::transfer("alice", "33")], 0, 0));
        //assert_eq!(T.tx_claim(DAY, &bob  )?, (vec![Snip20::transfer("bob",   "33")], 0, 0));
        //assert_eq!(T.tx_claim(DAY, &cyril)?, (vec![Snip20::transfer("cyril", "33")], 0, 0));
        //println!("{:#?}", T.q_pool_info(DAY));
        //println!("{:#?}", T.q_user_info(DAY, &alice));
        //println!("{:#?}", T.q_user_info(DAY, &bob));
        //println!("{:#?}", T.q_user_info(DAY, &cyril)); }
    //then "it's the same as if they provided the liquidity sequentially, as long as nobody claims" {
        //let mut T = RewardsHarness::new().fund(PORTION);
        //let _ = T.init_configured(0, &admin)?;
        //let _ = T.tx_set_vk(0, &alice, "")?;
        //let _ = T.tx_set_vk(0, &bob,   "")?;
        //let _ = T.tx_set_vk(0, &cyril, "")?;

        //let _ = T.tx_lock(              2, &alice, 100)?;
        //let _ = T.tx_retrieve(DAY * 1 + 2, &alice, 100)?;
        //let _ = T.tx_lock(    DAY * 1 + 3, &bob,   100)?;
        //let _ = T.tx_retrieve(DAY * 2 + 3, &bob,   100)?;
        //let _ = T.tx_lock(    DAY * 2 + 4, &cyril, 100)?;
        //let _ = T.tx_retrieve(DAY * 3 + 4, &cyril, 100)?;
        //println!("{:#?}", T.q_pool_info(DAY * 4));
        //println!("{:#?}", T.q_user_info(DAY * 4, &alice));
        //println!("{:#?}", T.q_user_info(DAY * 4, &bob));
        //println!("{:#?}", T.q_user_info(DAY * 4, &cyril));
        //assert_eq!(T.tx_claim(DAY * 4, &alice)?, (vec![Snip20::transfer("alice", "33")], 0, 0));
        //assert_eq!(T.tx_claim(DAY * 4, &bob  )?, (vec![Snip20::transfer("bob",   "33")], 0, 0));
        //assert_eq!(T.tx_claim(DAY * 4, &cyril)?, (vec![Snip20::transfer("cyril", "33")], 0, 0)); }
    //when "one of the users claims when providing liquidity sequentially"
    //then "the remaining rewards are split between the late-comer and the late-claimer" {
        //let mut T = RewardsHarness::new();
        //let _ = T.init_configured(0, &admin)?; }

}
