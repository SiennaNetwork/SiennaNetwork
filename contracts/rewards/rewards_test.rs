#![allow(unused_macros)]
#![allow(non_snake_case)]

use fadroma::{
    scrt::{HumanAddr, StdError},
    scrt_contract_harness::Harness
};
use crate::{
    rewards_harness::*,
    msg,
    rewards_config::load_viewing_key
};

const REWARD: u128 = 100;
const STAKE:  u128 = 100;

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
const DAY: u128 = crate::DAY as u128;

const NO_REWARDS: &str = "You've already received as much as your share of the reward pool allows. Keep your liquidity tokens locked and wait for more rewards to be vested, and/or lock more liquidity tokens to grow your share of the reward pool.";

kukumba_harnessed! {

    type Error = StdError;

    let Test: RewardsHarness<RewardsMockQuerier>;

    init {
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

    close {
        given "an instance:" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            Test.at(1)
                .init_configured(&admin)?
                .fund(REWARD)
                .set_vk(&alice, "")?
                .user(&alice,     0,   0,       0,   0,   0,   0)? }

        when  "pool is closed by admin after some activity" {
            Test.at(1)
                .lock(&alice, STAKE)?
                .user(&alice,     0, STAKE,       0,   0,   0,   0)?;

            Test.after(DAY)
                .user(&alice, 1*DAY, STAKE, STAKE*DAY, REWARD,      0, REWARD)?
                .claim(&alice, REWARD)?
                .user(&alice, 1*DAY, STAKE, STAKE*DAY, REWARD, REWARD,      0)?;

            Test.after(DAY)
                .retrieve(&alice, REWARD)?
                .fund(REWARD)
                .user(&alice, 2*DAY,   0, 200*DAY, 200, 100, 100)?;

            Test.after(DAY)
                .user(&alice, 2*DAY,   0, 200*DAY,  88, 100,   0)?
                .fund(REWARD)
                .user(&alice, 2*DAY,   0, 200*DAY, 133, 100,  33)?
                .lock(&alice, 50)?;

            Test.after(DAY)
                .user(&alice, 3*DAY,  50, 250*DAY, 168, 100,  68)?;

            Test.after(DAY)
                .user(&alice, 4*DAY,  50, 300*DAY, 192, 100,  92)?
                .close(&admin)?
                .user(&alice, 4*DAY,  50, 300*DAY, 192, 100,  92)?; }

        then  "time stops" {
            Test.after(DAY)
                .user(&alice, 4*DAY,  50, 300*DAY, 192, 100,  92)?;
            Test.after(DAY)
                .user(&alice, 4*DAY,  50, 300*DAY, 192, 100,  92)?; }

        given "a closed pool"
        when  "user with locked tokens performs any action"
        then  "user gets all locked tokens back instead"
        and   "user can't do anything else" {
            assert_eq!(Test.after(DAY).tx_lock(&alice, 1000)?,
                      (vec![Snip20::transfer(&alice.as_str(), "50") ], 0, 0));
            assert_eq!(Test.after(DAY).tx_lock(&alice, 1000)?,
                      (vec![], 0, 0)); }

        when  "user claims rewards"
        then  "user can earn no more rewards" {
            Test.after(DAY)
                .user(&alice, 4*DAY,   0, 300*DAY, 192, 100,  92)?;
            Test.claim(&alice, 92)?
                .user(&alice, 4*DAY,   0,       0,   0,   0,   0)?;
            Test.fund(REWARD)
                .user(&alice, 4*DAY,   0,       0,   0,   0,   0)? } }

    init_then_set_lp_token {
        given "no instance"
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

    one_user {
        given "an instance:" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            Test.at(1).init_configured(&admin)?
                      .set_vk(&alice, "")?
                      .fund(REWARD)
                      .user(&alice,    0,   0,   0, 0, 0, 0)? }

        when "alice first locks lp tokens,"
        then "alice's age and lifetime share starts incrementing;" {
            Test.at( 1).user(&alice,   0,   0,   0,   0, 0, 0)?.lock(&alice, 100)?;
            Test.at( 1).user(&alice,   0, 100,   0,   0, 0, 0)?;
            Test.at( 2).user(&alice,   1, 100, 100, 100, 0, 0)?;
            Test.at( 3).user(&alice,   2, 100, 200, 100, 0, 0)? }

        when "alice retrieves half of the tokens,"
        then "alice's age keeps incrementing;" {
            Test.at( 4).user(&alice,   3, 100, 300, 100, 0, 0)?.retrieve(&alice, 50)?
                       .user(&alice,   3,  50, 300, 100, 0, 0)? }

        when "alice retrieves all of the tokens,"
        then "alice's age start decrementing;" {
            Test.at( 5).user(&alice,   4,  50, 350, 100, 0, 0)?;
            Test.at( 6).user(&alice,   5,  50, 400, 100, 0, 0)?.retrieve(&alice, 50)?;
            Test.at( 6).user(&alice,   5,   0, 400, 100, 0, 0)?;
            Test.at( 7).user(&alice,   5,   0, 400,  69, 0, 0)?;
            Test.at( 8).user(&alice,   5,   0, 400,  50, 0, 0)? }

        when "alice locks tokens again,"
        then "alice's age starts incrementing again;" {
            Test.at( 9).user(&alice,   5,   0, 400,  38, 0, 0)?.lock(&alice,   1)?
                       .user(&alice,   5,   1, 400,  38, 0, 0)?
                .at(10).user(&alice,   6,   1, 401,  44, 0, 0)?
                .at(11).user(&alice,   7,   1, 402,  49, 0, 0)? }

        when "alice's age reaches the configured threshold,"
        then "alice is eligible to claim rewards" {
            Test.at(DAY+4).user(&alice, DAY, 1, 17675, 98, 0, 98)? } }

    lock_and_retrieve {
        given "an instance" {
            let admin   = HumanAddr::from("admin");
            let alice   = HumanAddr::from("alice");
            let bob     = HumanAddr::from("bob");
            let mallory = HumanAddr::from("mallory");
            Test.at(1).init_configured(&admin)? }

        when  "someone requests to lock tokens"
        then  "the instance transfers them to itself"
        and   "the liquidity provider starts accruing a reward" {
            Test.at(1).lock(&alice, 100)?
                .at(2).pool(100, 100, 1)? }

        when  "a provider requests to retrieve tokens"
        then  "the instance transfers them to the provider"
        and   "the reward now increases at a reduced rate" {
            Test.at(3).pool(100, 200, 1)?
                .at(4).retrieve(&alice, 50)? }

        when  "a provider requests to retrieve all their tokens"
        then  "the instance transfers them to the provider"
        and   "their reward stops increasing" {
            Test.at(5).pool(50, 350, 4)?.retrieve(&alice, 50)?
                .at(6).pool(0, 350, 5)? }

        when  "someone else requests to lock tokens"
        then  "the previous provider's share of the rewards begins to diminish" {
            Test.at(7).pool(0, 350, 5)?
                .at(7).lock(&bob, 500)?
                .at(8).pool(500, 850, 7)? }

        when  "a provider tries to retrieve too many tokens"
        then  "they get an error" {
            Test.at(9).pool(500, 1350, 7)?.retrieve_too_much(&bob, 1000u128, "not enough locked (500 < 1000)")?
                .at(10).pool(500, 1850, 7)? }

        when  "a stranger tries to retrieve any tokens"
        then  "they get an error" {
            Test.at(10).retrieve_too_much(&mallory, 100u128, "not enough locked (0 < 100)")?
                .at(11).pool(500, 2350, 7)? } }

    claim {
        given "an instance" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(1).init_configured(&admin)? }

        when  "strangers try to claim rewards"
        then  "they get an error" {
            Test.at(1).claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                      .claim_must_wait(&bob,   "lock tokens for 17280 more blocks to be eligible")? }

        when  "users provide liquidity"
        and   "they wait for rewards to accumulate" {
            Test.at(1)
                .lock(&alice, 100)?.claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                .lock(&bob,   100)?.claim_must_wait(&bob, "lock tokens for 17280 more blocks to be eligible")?
                .at(2).claim_must_wait(&alice, "lock tokens for 17279 more blocks to be eligible")?
                .at(3).claim_must_wait(&bob,   "lock tokens for 17278 more blocks to be eligible")?
                .at(4).claim_must_wait(&alice, "lock tokens for 17277 more blocks to be eligible")?
                .at(5).claim_must_wait(&bob,   "lock tokens for 17276 more blocks to be eligible")? }

        and   "a provider claims rewards"
        then  "that provider receives reward tokens" {
            Test.fund(REWARD)
                .at(1 + DAY).claim(&alice, 50)? }

        when  "a provider claims rewards twice within a period"
        then  "rewards are sent only the first time" {
            Test.at(1 + DAY).claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                .at(2 + DAY).claim_must_wait(&alice, "lock tokens for 17279 more blocks to be eligible")?
                .at(3 + DAY).claim_must_wait(&alice, "lock tokens for 17278 more blocks to be eligible")? }

        when  "a provider claims their rewards less often"
        then  "they receive equivalent rewards as long as the liquidity locked hasn't changed" {
            Test.fund(REWARD)
                .at(3 + DAY * 2).claim(&alice, 50)?.claim(&bob, 100)? } }

    two_parallel_users {
        given "an instance:" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(1).init_configured(&admin)?
                      .fund(REWARD)
                      .set_vk(&alice, "")?
                      .set_vk(&bob,   "")?
                      .user(&alice, 0, 0, 0, 0, 0, 0)?
                      .user(&bob,   0, 0, 0, 0, 0, 0)? }

        when "alice and bob first lock lp tokens simultaneously,"
        then "their ages and earnings start incrementing simultaneously;" {
            Test.at(1).user(&alice, 0,   0,   0, 0,  0, 0)?.lock(&alice, 100)?;
            Test.at(1).user(&bob,   0,   0,   0, 0,  0, 0)?.lock(&bob,   100)?;
            Test.at(1).user(&alice, 0, 100,   0, 0,  0, 0)?;
            Test.at(1).user(&bob,   0, 100,   0, 0,  0, 0)?;
            Test.at(2).user(&alice, 1, 100, 100, 50, 0, 0)?;
            Test.at(2).user(&bob,   1, 100, 100, 50, 0, 0)?;
            Test.at(3).user(&alice, 2, 100, 200, 50, 0, 0)?;
            Test.at(3).user(&bob,   2, 100, 200, 50, 0, 0)?; }

        when "alice and bob's ages reach the configured threshold,"
        then "each is eligible to claim half of the available rewards" {
            Test.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 50, 0, 50)?
                          .user(&bob,   DAY, 100, DAY * 100, 50, 0, 50)? } }

    two_sequential_users {
        given "an instance" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(1).init_configured(&admin)?
                      .set_vk(&alice, "")?
                      .set_vk(&bob,   "")?
                      .fund(REWARD) }

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
                .at(         DAY+2+1).user(&alice, DAY,   0, DAY * 100, 97, 0, 97)?
                .at(     DAY+2+DAY/2).user(&alice, DAY,   0, DAY * 100, 43, 0, 43)?
                .at(DAY+2+DAY/2+1000).user(&alice, DAY,   0, DAY * 100, 40, 0, 40)? }

        when "bob reaches the age threshold"
        then "each is eligible to claim some rewards" {
            Test.at(         2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 49, 0, 49)?.retrieve(&bob, 100)?
                                     .user(&bob,   DAY,   0, DAY * 100, 49, 0, 49)?
                                     .user(&alice, DAY,   0, DAY * 100, 24, 0, 24)? } }

    two_sequential_users_and_claim {
        given "an instance" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(1).init_configured(&admin)?
                      .set_vk(&alice, "")?
                      .set_vk(&bob,   "")? }

        when "alice locks lp tokens,"
        and  "alice retrieves them after reaching the threshold;"
        then "alice is eligible to claim the whole pool" {
            Test.fund(REWARD)
                .at(    1).user(&alice, 0, 0, 0, 0, 0, 0)?.lock(&alice, 100)?
                .at(DAY+1).user(&alice, DAY, 100, DAY * 100, 100, 0, 100)?.retrieve(&alice, 100)?
                          .user(&alice, DAY,   0, DAY * 100, 100, 0, 100)? }

        when "bob locks the same amount of tokens" {
            Test.at(DAY+2).user(&bob,    0,   0, 0, 0, 0, 0)?.lock(&bob, 100)?
                          .user(&bob,    0, 100, 0, 0, 0, 0)? }

        then "alice's rewards start decreasing proportionally" {
            Test.at(DAY+2+1).user(&alice, DAY, 0, DAY * 100, 97, 0, 97)? }

        when "alice claims some time after maturing"
        then "alice's state is reset because of selective_memory" {
            Test.at(     DAY+2+DAY/2).user(&alice, DAY, 0, DAY * 100, 43, 0, 43 )?.claim(&alice, 43)?
                .at(1000+DAY+2+DAY/2).user(&alice, DAY, 0, 0, 0, 0, 0) }

        when "bob reaches the age threshold"
        then "bob is eligible to claim a comparable amount of rewards" {
            Test.at(2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 49,  0, 49)?.retrieve(&bob, 100)?
                            .user(&bob,   DAY,   0, DAY * 100, 49,  0, 49)?
                            .user(&alice, DAY,   0, 0, 0, 0, 0)? } }

    single_sided_staking {
        given "an instance where the lp token and the reward token are the same" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            Test.at(1).init_partial(&admin)?
                      .set_token(&admin, "reward_token_address", "reward_token_hash")?
                      .set_vk(&alice, "")?; }

        when "alice locks tokens"
        and "alice becomes eligible for rewards" {
            Test.fund(REWARD)
                .at(    1).user(&alice,   0,     0,           0,      0, 0,      0)?
                .lock(&alice, STAKE)?.fund(STAKE) // because it gets added to the same balance
                .at(DAY+1).user(&alice, DAY, STAKE, DAY * STAKE, REWARD, 0, REWARD)?; }

        when "alice claims rewards"
        then "they are calculated from the reward balance" {
            Test.claim(&alice, REWARD)?;
        }

        when "alice retrieves tokens"
        then "alice gets back the original amount" {
            Test.retrieve(&alice, 100)?;
        }
    }

    global_ratio_zero {

        given "an instance with 0/1 ratio" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            Test.at(1)
                .init_configured(&admin)?
                .set_ratio(&admin, 0u128, 1u128)?
                .fund(REWARD)
                .set_vk(&alice, "")?
                .user(&alice,     0,   0,       0,   0,   0,   0)? }

        when "user becomes eligible for rewards"
        then "rewards are zero" {
            Test.at(DAY+1)
                .user(&alice,     0,   0,       0,   0,   0,   0)? }

        when "ratio is set to 1/1"
        then "rewards can be claimed" {
            Test.at(DAY+2)
                .set_ratio(&admin, 1u128, 1u128)?
                .user(&alice,     0,   0,       0,   0,   0,   0)? } }

    claim_ratio_zero {
        given "an instance" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");
            let bob   = HumanAddr::from("bob");
            Test.at(1).init_configured(&admin)? }

        when  "strangers try to claim rewards"
        then  "they get an error" {
            Test.at(1).claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                      .claim_must_wait(&bob,   "lock tokens for 17280 more blocks to be eligible")? }

        when  "users provide liquidity"
        and   "they wait for rewards to accumulate" {
            Test.at(1)
                .lock(&alice, 100)?.claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                .lock(&bob,   100)?.claim_must_wait(&bob, "lock tokens for 17280 more blocks to be eligible")?
                .at(2).claim_must_wait(&alice, "lock tokens for 17279 more blocks to be eligible")?
                .at(3).claim_must_wait(&bob,   "lock tokens for 17278 more blocks to be eligible")?
                .at(4).claim_must_wait(&alice, "lock tokens for 17277 more blocks to be eligible")?
                .at(5).claim_must_wait(&bob,   "lock tokens for 17276 more blocks to be eligible")? }

        and   "a provider claims rewards"
        then  "that provider receives reward tokens" {
            Test.fund(REWARD)
                .set_ratio(&admin, 0u128, 1u128)?
                .at(1 + DAY).claim_must_wait(&alice, NO_REWARDS)? }

        when  "a provider claims rewards twice within a period"
        then  "rewards are sent only the first time" {
            Test.at(1 + DAY).claim_must_wait(&alice, NO_REWARDS)?
                .at(2 + DAY).claim_must_wait(&alice, NO_REWARDS)?
                .at(3 + DAY).claim_must_wait(&alice, NO_REWARDS)? }

        when  "a provider claims their rewards less often"
        then  "they receive equivalent rewards as long as the liquidity locked hasn't changed" {
            Test.fund(REWARD)
                .set_ratio(&admin, 1u128, 1u128)?
                .at(3 + DAY * 2).claim(&alice, 100)?.claim(&bob, 100)? } }

    release_snip20 {
        given "an instance" {
            let admin = HumanAddr::from("admin");
            let alice = HumanAddr::from("alice");

            let key = "key";

            let msg = msg::Handle::ReleaseSnip20 {
                snip20: Test.reward_token(),
                key: key.into(),
                recipient: None
            };

            Test.at(1).init_configured(&admin)? }

        when "non admin-tries to call release"
        then "gets rejected" {
            assert!(Test.tx(20, &alice, msg.clone()).is_err());
        }

        when "calling with reward token info"
        then "the viewing key changes" {
            assert!(Test.tx(20, &admin, msg).is_ok());
            let deps = Test.deps();
            let vk = load_viewing_key(&deps.storage)?;

            assert_eq!(vk.0, String::from(key));
        }
    }

}
