#![allow(unused_macros)]
#![allow(non_snake_case)]

use fadroma::scrt::{
    cosmwasm_std::{StdError, MemoryStorage, CanonicalAddr},
    storage::Writable
};

use crate::{rewards_harness::*, rewards_algo::Pool};

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
            let addr = CanonicalAddr::default();
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
            let mut user = Pool::new(&mut s).at(10).user(addr.clone());
            user.lock_tokens(100u128.into())?;
            assert_eq!(
                user.locked()?,
                100u128.into()); }

        then  "their age starts incrementing" {
            let mut user = Pool::new(&mut s).at( 10).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), ( 0, 0,  0));
            let mut user = Pool::new(&mut s).at( 50).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (40, 0, 40));
            let mut user = Pool::new(&mut s).at(100).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (90, 0, 90)); }

        when  "a user's balance is updated"
        then  "their current age is committed"
        and   "their age keeps incrementing" {
            Pool::new(&mut s).at(110)
                .user(addr.clone())
                .retrieve_tokens(50u128.into())?;
            let mut user = Pool::new(&mut s).at(110).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (100, 100, 0));
            let mut user = Pool::new(&mut s).at(150).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (140, 100, 40));
            let mut user = Pool::new(&mut s).at(200).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (190, 100, 90)); }

        when  "a user unlocks all their LP tokens"
        then  "their current age is committed"
        and   "their age stops incrementing until they lock again" {
            Pool::new(&mut s).at(210)
                .user(addr.clone())
                .retrieve_tokens(50u128.into())?;
            let mut user = Pool::new(&mut s).at(210).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (200, 200, 0));
            let mut user = Pool::new(&mut s).at(250).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (200, 200, 0));
            let mut user = Pool::new(&mut s).at(300).user(addr.clone());
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (200, 200, 0)); }

        when  "a user claims before reaching the age threshold"
        then  "the claim is refused" {
            assert_eq!(user.claimable()?, Amount::zero());
            assert!(user.claim_reward().is_err()); }

        when  "a user claims after reaching the age threshold"
        then  "the claim is fulfilled" {
            Pool::new(&mut s).at(310)
                .user(addr.clone())
                .lock_tokens(50u128.into())?;
            assert_eq!(
                Pool::new(&mut s).at(crate::DAY * 2)
                                 .with_balance(100u128.into())
                                 .user(addr.clone())
                                 .claim_reward()?, Amount::from(100u128)); } }

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

        given "a pool" {
            let addr = CanonicalAddr::default();
            let mut s = MemoryStorage::new(); }

        when "LP tokens have never been locked"
        then "the pool liquidity ratio is unknown" {
            let mut pool = Pool::new(&mut s).at(0);
            pool.configure_threshold(&crate::DAY)?
                .configure_cooldown(&0)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?;
            assert!(pool.liquidity_ratio()
                .is_err()); }

        when "LP tokens are locked"
        then "the pool liquidity ratio is 1" {
            assert!(Pool::new(&mut s).at(10000).liquidity_ratio()
                .is_err());
            Pool::new(&mut s).at(10000).user(addr.clone())
                .lock_tokens(100u128.into())?;
            assert_eq!(Pool::new(&mut s).at(10000)
                .liquidity_ratio()?, 100000000u128.into()); }

        when "some LP tokens are unlocked"
        then "the pool liquidity ratio remains 1" {
            assert_eq!(Pool::new(&mut s).at(20000)
                .liquidity_ratio()?, 100000000u128.into());
            Pool::new(&mut s).at(20000).user(addr.clone())
                .retrieve_tokens(50u128.into())?;
            assert_eq!(Pool::new(&mut s).at(20000)
                .liquidity_ratio()?, 100000000u128.into()); }

        when "all LP tokens are unlocked"
        then "the pool liquidity ratio begins to decrease toward 0" {
            assert_eq!(Pool::new(&mut s).at(30000)
                .liquidity_ratio()?, 100000000u128.into());
            Pool::new(&mut s).at(30000).user(addr.clone())
                .retrieve_tokens(50u128.into())?;
            assert_eq!(Pool::new(&mut s).at(30000)
                .liquidity_ratio()?, 100000000u128.into());
            assert_eq!(Pool::new(&mut s).at(50000)
                .liquidity_ratio()?,   50000000u128.into()); }

        when "some LP tokens are locked again"
        then "the pool liquidity ratio begins to increase toward 1" {
            Pool::new(&mut s).at(50000).user(addr.clone())
                .lock_tokens(50u128.into())?;
            assert_eq!(Pool::new(&mut s).at(90000)
                .liquidity_ratio()?, 75000000u128.into()); }

        when "a user is eligible to claim rewards"
        then "the rewards are diminished by the pool liquidity ratio" {
            let mut pool = Pool::new(&mut s).at(90000);
            println!("pool.liquidity_ratio{}", pool.liquidity_ratio()?);
            println!("pool.lifetime {}", pool.lifetime()?);
            let mut user = Pool::new(&mut s).at(90000).user(addr.clone());
            println!("user.lifetime {}", user.lifetime()?);
            println!("{} {} {} {}",
                user.existed()?, user.last_existed()?,
                user.present()?, user.last_present()?);
            user.retrieve_tokens(50u128.into())?;
            println!("{} {} {} {}",
                user.existed()?, user.last_existed()?,
                user.present()?, user.last_present()?);
            // mock away the user liquidity ratio:
            let never_left = user.last_existed()?;
            let address = user.address.as_slice();
            let mut user = Pool::new(&mut s).at(90000).with_balance(100u128.into())
                .user(addr.clone());
            user.save_ns(crate::rewards_algo::USER_PRESENT, address, never_left)?;
            println!("{} {} {} {}",
                user.existed()?, user.last_existed()?,
                user.present()?, user.last_present()?);
            println!("user.lifetime {}", user.lifetime()?);
            assert_eq!(user.claim_reward()?, 75u128.into()); } }

    feature_user_liquidity_ratio {
        given "nothing" { unimplemented!() }

        given "a pool" {
            let mut s = MemoryStorage::new(); }

        when "LP tokens have never been locked by this user"
        then "the user's liquidity ratio is 1" {}

        when "LP tokens are locked by this user"
        then "the user's liquidity ratio is still 1" {}

        when "LP tokens are unlocked by this user"
        then "the user's liquidity ratio begins to decrease toward 0" {}

        when "LP tokens are locked again by this user"
        then "the user's liquidity ratio begins to increase toward 1" {}

        when "the user is eligible to claim rewards"
        then "the rewards are diminished by their liquidity ratio" {}
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
