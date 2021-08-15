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

        given "a pool"
        when  "the age threshold is first queried"
        then  "the pool returns the default value" {
            let addr = CanonicalAddr::default();
            let mut s = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            assert!(pool.threshold().is_err()); }

        when  "the age threshold is updated"
        then  "the pool returns the new value" {
            pool.configure_threshold(&crate::DAY)?
                .configure_cooldown(&0)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?;
            assert_eq!(Pool::new(&mut s).threshold()?,
                       crate::DAY); }

        when  "a user locks LP tokens" {
            let mut user = Pool::new(&mut s).at(10).user(addr.clone());
            user.lock_tokens(100u128.into())?;
            assert_eq!(user.locked()?,
                       100u128.into()); }

        then  "their age starts incrementing" {
            user.pool.set_time(10);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), ( 0, 0,  0));
            user.pool.set_time(50);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (40, 0, 40));
            user.pool.set_time(100);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (90, 0, 90)); }

        when  "a user's balance is updated"
        then  "their current age is committed"
        and   "their age keeps incrementing" {
            user.pool.set_time(110);
            user.retrieve_tokens(50u128.into())?;
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (100, 100, 0));
            user.pool.set_time(150);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (140, 100, 40));
            user.pool.set_time(200);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (190, 100, 90)); }

        when  "a user unlocks all their LP tokens"
        then  "their current age is committed"
        and   "their age stops incrementing until they lock again" {
            user.pool.set_time(210);
            user.retrieve_tokens(50u128.into())?;
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (200, 200, 0));
            user.pool.set_time(250);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (200, 200, 0));
            user.pool.set_time(300);
            assert_eq!((user.present()?,
                        user.last_present()?,
                        user.elapsed_present()?), (200, 200, 0)); }

        when  "a user claims before reaching the age threshold"
        then  "the claim is refused" {
            assert_eq!(user.claimable()?, Amount::zero());
            assert!(user.claim_reward().is_err()); }

        when  "a user claims after reaching the age threshold"
        then  "the claim is fulfilled" {
            user.pool.set_time(310);
            user.lock_tokens(50u128.into())?;
            user.pool.set_time(crate::DAY * 2);
            user.pool.set_balance(100u128.into());
            #[cfg(feature="pool_liquidity_ratio")] user.pool.reset_liquidity_ratio()?;
            #[cfg(feature="user_liquidity_ratio")] user.reset_liquidity_ratio()?;
            assert_eq!(user.claim_reward()?, Amount::from(100u128)); } }

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

        given "a pool and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(0);
            pool.configure_threshold(&crate::DAY)?
                .configure_cooldown(&0)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?;
            let mut user = pool.user(addr.clone()); }

        when "LP tokens have never been locked"
        then "the pool liquidity ratio is unknown" {
            assert!(user.pool.liquidity_ratio().is_err()); }

        when "LP tokens are locked"
        then "the pool liquidity ratio is 1" {
            user.pool.set_time(10000);
            assert!(user.pool.liquidity_ratio().is_err());
            user.lock_tokens(100u128.into())?;
            assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into()); }

        when "some LP tokens are unlocked"
        then "the pool liquidity ratio remains 1" {
            user.pool.set_time(20000);
            assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
            user.retrieve_tokens(50u128.into())?;
            assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into()); }

        when "all LP tokens are unlocked"
        then "the pool liquidity ratio begins to decrease toward 0" {
            user.pool.set_time(30000);
            assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
            user.retrieve_tokens(50u128.into())?;
            assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
            user.pool.set_time(50000);
            assert_eq!(user.pool.liquidity_ratio()?,  50000000u128.into()); }

        when "some LP tokens are locked again"
        then "the pool liquidity ratio begins to increase toward 1" {
            user.lock_tokens(50u128.into())?;
            user.pool.set_time(90000);
            assert_eq!(user.pool.liquidity_ratio()?,  75000000u128.into()); }

        when "a user is eligible to claim rewards"
        then "the rewards are diminished by the pool liquidity ratio" {
            user.pool.set_balance(100u128.into());
            user.retrieve_tokens(50u128.into())?;
            assert_eq!(user.claim_reward()?, 75u128.into()); } }

    feature_user_liquidity_ratio {

        given "a pool and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(0);
            pool.configure_threshold(&crate::DAY)?
                .configure_cooldown(&0)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?;
            let mut user = pool.user(addr.clone()); }

        when "LP tokens have never been locked by this user"
        then "the user's liquidity ratio is 1" {
            assert_eq!(user.liquidity_ratio()?, 100000000u128.into()); }

        when "LP tokens are locked by this user"
        then "the user's liquidity ratio remains 1" {
            user.pool.set_time(10000);
            user.lock_tokens(100u128.into())?;
            assert_eq!(user.liquidity_ratio()?, 100000000u128.into()); }

        when "some LP tokens are unlocked by this user"
        then "the user's liquidity ratio remains 1" {
            user.pool.set_time(20000);
            user.retrieve_tokens(50u128.into())?;
            assert_eq!(user.liquidity_ratio()?, 100000000u128.into()); }

        when "all LP tokens are unlocked by this user"
        then "the user's liquidity ratio begins to decrease toward 0" {
            user.pool.set_time(30000);
            user.retrieve_tokens(50u128.into())?;
            assert_eq!(user.liquidity_ratio()?, 100000000u128.into());

            user.pool.set_time(40000);
            assert_eq!(user.liquidity_ratio()?,  66666666u128.into()); }

        when "LP tokens are locked again by this user"
        then "the user's liquidity ratio begins to increase toward 1" {
            user.pool.set_time(50000);
            user.lock_tokens(50u128.into())?;
            assert_eq!(user.liquidity_ratio()?,  50000000u128.into());

            user.pool.set_time(90000);
            assert_eq!(user.liquidity_ratio()?,  75000000u128.into()); }

        when "the user is eligible to claim rewards"
        then "the rewards are diminished by the user's liquidity ratio" {
            user.retrieve_tokens(50u128.into())?;
            user.pool.set_balance(100u128.into());

            // mock away the pool liquidity ratio if applied:
            #[cfg(feature="pool_liquidity_ratio")]
            user.pool.reset_liquidity_ratio()?;

            assert_eq!(user.claim_reward()?, 75u128.into()); } }

    feature_selective_memory {
        given "a pool and a user" { unimplemented!() }

        when  "user locks tokens"
        and   "user first claims rewards and then then unlocks all tokens"
        then  "user's lifetime is preserved"
        and   "user can continue accumulating lifetime later" { unimplemented!() }

        when  "a user locks tokens"
        and   "user first unlocks all tokens and then claims rewards"
        then  "user's lifetime is erased"
        and   "former stakers don't get a continual pension" { unimplemented!() }

        given "a user is crowded out"
        when  "user unlocks all tokens"
        then  "user's lifetime is preserved"
        and   "crowded out users can't reset their negative claimable" { unimplemented!() } }

    feature_pool_closes {
        given "a pool" { unimplemented!() }
        when  "it is closed by the admin" { unimplemented!() }
        then  "every tallied variable is committed to storage" { unimplemented!() }
        and   "time stops" { unimplemented!() }
        when  "an eligible user claims their rewards" { unimplemented!() }
        then  "they do not accrue any more rewards" { unimplemented!() } }

}
