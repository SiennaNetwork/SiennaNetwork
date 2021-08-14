#![allow(unused_macros)]
#![allow(non_snake_case)]

use fadroma::scrt::{cosmwasm_std::{HumanAddr, StdError, MemoryStorage, CanonicalAddr as Addr}};
use crate::{rewards_harness::*, rewards_algo::{User, Pool}};

const PORTION: u128 = 100;

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
const DAY: u128 = crate::DAY as u128;

kukumba_harnessed! {

    type Error = StdError;

    let Test: RewardsHarness<RewardsMockQuerier>;

    feature_age_threshold {

        given "a pool" {
            let mut s = MemoryStorage::new(); }

        when  "the age threshold is first queried"
        then  "the pool returns the default value" {
            assert!(Pool::new(&mut s).threshold().is_err()); }

        when  "the age threshold is updated" {
            Pool::new(&mut s)
                .configure_threshold(&crate::DAY)?
                .configure_cooldown(&0)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?; }

        then  "the pool returns the new value" {
            assert_eq!(Pool::new(&mut s).threshold()?,
                       crate::DAY); }

        when  "a user locks LP tokens" {
            let mut user = Pool::new(&mut s).at(10).user(Addr::default());
            user.lock_tokens(100u128.into())?;
            assert_eq!(
                user.locked()?,
                100u128.into()); }

        then  "their age starts incrementing" {
            let mut user = Pool::new(&mut s).at( 10).user(Addr::default());
            assert_eq!(user.present()?,           0);
            assert_eq!(user.last_present()?,      0);
            assert_eq!(user.elapsed_present()?,   0);
            let mut user = Pool::new(&mut s).at( 50).user(Addr::default());
            assert_eq!(user.present()?,          40);
            assert_eq!(user.last_present()?,      0);
            assert_eq!(user.elapsed_present()?,  40);
            let mut user = Pool::new(&mut s).at(100).user(Addr::default());
            assert_eq!(user.present()?,          90);
            assert_eq!(user.last_present()?,      0);
            assert_eq!(user.elapsed_present()?,  90); }

        when  "a user's balance is updated"
        then  "their current age is committed"
        and   "their age keeps incrementing" {
            Pool::new(&mut s)
                .at(110)
                .user(Addr::default())
                .retrieve_tokens(50u128.into())?;
            let mut user = Pool::new(&mut s).at(110).user(Addr::default());
            assert_eq!(user.present()?,         100);
            assert_eq!(user.last_present()?,    100);
            assert_eq!(user.elapsed_present()?,   0);
            let mut user = Pool::new(&mut s).at(150).user(Addr::default());
            assert_eq!(user.present()?,         140);
            assert_eq!(user.last_present()?,    100);
            assert_eq!(user.elapsed_present()?,  40);
            let mut user = Pool::new(&mut s).at(200).user(Addr::default());
            assert_eq!(user.present()?,         190);
            assert_eq!(user.last_present()?,    100);
            assert_eq!(user.elapsed_present()?,  90); }

        when  "a user unlocks all their LP tokens"
        then  "their current age is committed"
        and   "their age stops incrementing until they lock again" {
            Pool::new(&mut s)
                .at(210)
                .user(Addr::default())
                .retrieve_tokens(50u128.into())?;
            let mut user = Pool::new(&mut s).at(210).user(Addr::default());
            assert_eq!(user.present()?,         200);
            assert_eq!(user.last_present()?,    200);
            assert_eq!(user.elapsed_present()?,   0);
            let mut user = Pool::new(&mut s).at(250).user(Addr::default());
            assert_eq!(user.present()?,         200);
            assert_eq!(user.last_present()?,    200);
            assert_eq!(user.elapsed_present()?,   0);
            let mut user = Pool::new(&mut s).at(300).user(Addr::default());
            assert_eq!(user.present()?,         200);
            assert_eq!(user.last_present()?,    200);
            assert_eq!(user.elapsed_present()?,   0); }

        when  "a user claims before reaching the age threshold"
        then  "the claim is refused" {
            assert_eq!(user.claimable()?, Amount::zero());
            assert!(user.claim_reward().is_err()); }

        when  "a user claims after reaching the age threshold"
        then  "the claim is fulfilled" {
            Pool::new(&mut s)
                .at(310)
                .user(Addr::default())
                .lock_tokens(50u128.into())?;
            assert_eq!(
                Pool::new(&mut s).at(crate::DAY * 2)
                                 .with_balance(100u128.into())
                                 .user(Addr::default())
                                 .claim_reward()?, Amount::from(100u128)); }

        //user.claimable
        //user.update
        //user.claim_reward
    }

    feature_claim_cooldown {
        given "nothing" { unimplemented!() }
        //pool.cooldown
        //pool.configure_cooldown
        //user.elapsed
        //user.cooldown
        //user.last_cooldown
        //user.reset_cooldown
        //user.update
        //user.claim_reward
    }

    feature_global_ratio {
        given "nothing" { unimplemented!() }
        //pool.ratio
        //pool.configure_ratio
        //pool.earned
    }

    feature_pool_liquidity_ratio {
        given "nothing" { unimplemented!() }
        //pool.liquid
        //pool.last_liquid
        //pool.liquidity_ratio
        //pool.existed
        //pool.populated
        //pool.created
        //pool.update_locked
        //pool.configure_created
        //pool.configure_populated
        //user.earned
    }

    feature_user_liquidity_ratio {
        given "nothing" { unimplemented!() }
        //user.existed
        //user.last_existed
        //user.present
        //user.last_present
        //user.lifetime
        //user.share
        //user.update
    }

    feature_selective_memory {
        given "nothing" { unimplemented!() }
        //user.claim_reward
    }

    feature_pool_closes {
        given "nothing" { unimplemented!() }
        //pool.elapsed
        //pool.closed
        //pool.close
        //user.elapsed
    }

}
