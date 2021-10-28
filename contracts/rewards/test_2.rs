#![allow(unused_macros)]
#![allow(non_snake_case)]

use fadroma::*;
use crate::*;
use crate::test_harness::*;

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
const DAY: u128 = crate::DAY as u128;

const PORTION: u128 = 100;

kukumba_harnessed! {

    type Error = StdError;

    let Test: RewardsHarness<RewardsMockQuerier>;

    age_threshold {

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
            assert_eq!(Pool::new(&mut s).threshold()?, crate::DAY); }

        when  "a user locks LP tokens" {
            let mut user = Pool::new(&mut s).at(10).user(addr.clone());
            user.lock_tokens(100u128.into())?;
            assert_eq!(user.locked()?, 100u128.into()); }

        then  "their age starts incrementing" {
            user.pool.set_time(10);
            assert_eq!(( 0, 0,  0),
                (user.present()?, user.last_present()?, user.elapsed_present()?));
            user.pool.set_time(50);
            assert_eq!((40, 0, 40),
                (user.present()?, user.last_present()?, user.elapsed_present()?));
            user.pool.set_time(100);
            assert_eq!((90, 0, 90),
                (user.present()?, user.last_present()?, user.elapsed_present()?)); }

        when  "a user's balance is updated"
        then  "their current age is committed"
        and   "their age keeps incrementing" {
            user.pool.set_time(110);
            user.retrieve_tokens(50u128.into())?;
            assert_eq!((100, 100, 0),
                (user.present()?, user.last_present()?, user.elapsed_present()?));
            user.pool.set_time(150);
            assert_eq!((140, 100, 40),
                (user.present()?, user.last_present()?, user.elapsed_present()?));
            user.pool.set_time(200);
            assert_eq!((190, 100, 90),
                (user.present()?, user.last_present()?, user.elapsed_present()?)); }

        when  "a user unlocks all their LP tokens"
        then  "their current age is committed"
        and   "their age stops incrementing until they lock again" {
            user.pool.set_time(210);
            user.retrieve_tokens(50u128.into())?;
            assert_eq!((200, 200, 0),
                (user.present()?, user.last_present()?, user.elapsed_present()?));
            user.pool.set_time(250);
            assert_eq!((200, 200, 0),
                (user.present()?, user.last_present()?, user.elapsed_present()?));
            user.pool.set_time(300);
            assert_eq!((200, 200, 0),
                (user.present()?, user.last_present()?, user.elapsed_present()?)); }

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

    claim_cooldown {

        given "a pool and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(1)
                .set_balance(100u128.into())
                .configure_threshold(&0)?
                .configure_cooldown(&1000)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?; }

        when "user claims rewards"
        then "user must wait the cooldown amount before claiming again" {
            let mut user = pool.user(addr.clone());
            user.lock_tokens(100u128.into())?;
            user.pool.set_time(100000);
            assert_eq!(user.claim_reward()?, 100u128.into());
            assert_eq!(user.claim_reward(), Err(StdError::generic_err(
                "lock tokens for 1000 more blocks to be eligible")));
            user.pool.set_time(100500);
            assert_eq!(user.claim_reward(), Err(StdError::generic_err(
                "lock tokens for 500 more blocks to be eligible")));
            user.pool.set_time(101000);
            assert_eq!(user.claim_reward()?, 100u128.into());
            assert_eq!(user.claim_reward(), Err(StdError::generic_err(
                "lock tokens for 1000 more blocks to be eligible")));} }

    global_ratio {

        given "a pool and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(0).set_balance(100u128.into())
                .configure_threshold(&0)?
                .configure_cooldown(&0)?
                .configure_ratio(&(1u128.into(), 2u128.into()))?; }

        when "user becomes eligible for rewards"
        then "rewards are diminished by the global rewards ratio" {
            let mut user = pool.user(addr.clone());
            user.lock_tokens(100u128.into())?;
            user.pool.set_time(100000);
            assert_eq!(user.claimable()?, 50u128.into()); } }

    global_ratio_zero {

        given "a pool with 0/1 ratio, and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(1).set_balance(100u128.into())
                .configure_threshold(&0)?
                .configure_cooldown(&0)?
                .configure_ratio(&(0u128.into(), 1u128.into()))?; }

        when "user becomes eligible for rewards"
        then "rewards are zero" {
            let mut user = pool.user(addr.clone());
            user.lock_tokens(100u128.into())?;
            user.pool.set_time(100000);
            assert_eq!(user.claimable()?,    0u128.into());
            user.pool.set_balance(200u128.into());
            assert_eq!(user.claimable()?,    0u128.into()); }

        when "ratio is set to 1/1"
        then "rewards can be claimed" {
            user.pool.configure_ratio(&(1u128.into(), 1u128.into()))?;
            user.pool.set_time(100000);
            assert_eq!(user.claimable()?,    200u128.into());
            assert_eq!(user.claim_reward()?, 200u128.into());
            user.pool.set_balance(0u128.into());
            assert_eq!(user.claimable()?,      0u128.into());
            assert_eq!(user.claimed()?,      200u128.into());
            user.pool.set_balance(300u128.into());
            assert_eq!(user.claimable()?,    300u128.into());
            assert_eq!(user.claim_reward()?, 300u128.into());
            user.pool.set_balance(0u128.into());
            assert_eq!(user.claimable()?,      0u128.into());
            assert_eq!(user.claimed()?,      500u128.into());
        } }

    pool_liquidity_ratio {

        given "a pool and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(0)
                .configure_threshold(&crate::DAY)?
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
            #[cfg(feature="user_liquidity_ratio")] user.reset_liquidity_ratio()?;
            assert_eq!(user.claim_reward()?, 75u128.into()); } }

    user_liquidity_ratio {

        given "a pool and a user" {
            let addr     = CanonicalAddr::default();
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(0)
                .configure_threshold(&crate::DAY)?.configure_cooldown(&0)?
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

    selective_memory {

        given "a pool and a user" {
            let addr1    = CanonicalAddr(Binary(vec![0,0,0,1]));
            let addr2    = CanonicalAddr(Binary(vec![0,0,0,2]));
            let mut s    = MemoryStorage::new();
            let mut pool = Pool::new(&mut s);
            pool.set_time(0).set_balance(100u128.into())
                .configure_threshold(&crate::DAY)?.configure_cooldown(&crate::DAY)?
                .configure_ratio(&(1u128.into(), 1u128.into()))?;
            let mut user1 = pool.user(addr1.clone());}

        when  "user locks tokens and becomes eligible for rewards" {
            user1.pool.set_time(1);
            user1.lock_tokens(100u128.into())?;
            user1.pool.set_time(1 + crate::DAY);
            let lifetime = user1.lifetime()?; }
        and   "user first claims rewards and then then unlocks all tokens" {
            user1.claim_reward()?;
            user1.retrieve_tokens(100u128.into())?; }
        then  "user's lifetime is preserved"
        and   "user can continue accumulating lifetime later" {
            assert_eq!(user1.lifetime()?, lifetime); }

        when  "user locks tokens and becomes eligible for rewards" {
            user1.pool.set_time(1 + crate::DAY*2).set_balance(200u128.into());
            user1.lock_tokens(100u128.into())?;
            user1.pool.set_time(1 + crate::DAY*3); }
        and   "user first unlocks all tokens and then claims rewards" {
            user1.retrieve_tokens(100u128.into())?;
            user1.claim_reward()?; }
        then  "user's lifetime is erased"
        and   "former stakers don't get a continual pension" {
            assert_eq!(user1.lifetime()?, Volume::zero()); }

        when  "share of user who has previously claimed rewards diminishes"
        then  "user is crowded out"
        and   "user can't claim" {
            user1.lock_tokens(100u128.into())?;
            user1.pool.set_time(1 + crate::DAY*4);
            user1.claim_reward()?;
            let mut user2 = user1.pool.user(addr2.clone());
            user2.lock_tokens(1000u128.into())?;
            user2.pool.set_time(1 + crate::DAY*5);
            let mut user1 = user2.pool.user(addr1.clone());
            assert!(user1.earned()? < user1.claimed()?);
            assert_eq!(user1.claimable()?, Amount::zero()); }

        when  "user unlocks all tokens"
        then  "user's lifetime is preserved"
        and   "crowded out users can't reset their negative claimable" {
            user1.retrieve_tokens(100u128.into())?;
            assert!(user1.earned()? < user1.claimed()?); } }

}
