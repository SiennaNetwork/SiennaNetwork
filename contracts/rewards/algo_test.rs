#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(unreachable_patterns)]

use crate::*;
use crate::test::*;
use fadroma::*;
use fadroma::secret_toolkit::snip20;

macro_rules! assert_error {
    ($response:expr, $msg:expr) => { assert_eq!($response, Err(StdError::generic_err($msg))) }
}

macro_rules! assert_fields {
    ($instance:expr ; $variant:path {
        $($var:ident: $expected:expr),+
    }) => { {
        let mut tw = tabwriter::TabWriter::new(std::io::stdout());
        write!(&mut tw, "field\texpected\tactual\t\n");
        $(
            write!(&mut tw, "{}\t", stringify!($var));
            write!(&mut tw, "{:?}\t", $expected);
            write!(&mut tw, "{:?}\t\n", (if $var == $expected {
                yansi::Paint::green
            } else {
                yansi::Paint::red
            })(format!("{}", &$var)));
        )+;
    }; }
}

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
//const DAY:        u128 = crate::DAY as u128;
//const NO_REWARDS: &str = "You've already received as much as your share of the reward pool allows. Keep your liquidity tokens deposited and wait for more rewards to be vested, and/or deposit more liquidity tokens to grow your share of the reward pool.";
//const PORTION:    u128 = 100;
//const REWARD:     u128 = 100;
//const STAKE:      u128 = 100;

// Look Ma, no macros! ////////////////////////////////////////////////////////////////////////////

/// Given no instance
///
///  When the admin inits an instance without providing a reward token
///  Then the init fails
///
///  When the admin inits an instance with a configured reward token
///  Then the default values are used where applicable
///   And the rewards module emits a message that sets the reward token viewing key
#[test] fn test_init () {
    let (ref mut deps, ref VK, ref SIENNA, ref LP) = entities();
    admin(deps).at(1).init_invalid();
    admin(deps).at(1).init(LP, SIENNA, VK.clone());
}

/// Given no instance
///
///  When the admin inits an instance with an empty configuration
///  Then the default values are used where applicable
///   And no viewing key config message is returned
///
///  When someone else tries to set the config
///  Then the config remains unchanged
///
///  When the admin sets the config, including a reward token
///  Then a reward token viewing key config message is returned
#[test] fn test_configure () {
    let (ref mut deps, ref VK, ref SIENNA, ref LP) = entities();
    admin(deps)
        .at(1).init(LP, SIENNA, VK.clone())
        .at(2).configure(RewardsConfig {
            lp_token:     None,
            reward_token: Some(SIENNA.link.clone()),
            reward_vk:    Some(VK.clone()),
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        });
    badman(deps)
        .at(3).cannot_configure();
    admin(deps)
        .at(4).configure(RewardsConfig {
            lp_token:     None,
            reward_token: Some(SIENNA.link.clone()),
            reward_vk:    Some(VK.clone()),
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        });
}

/// Given an instance
///
///  When user first deposits
///  Then user's age and lifetime start incrementing
///
///  When user withdraws half of the tokens
///  Then user's age keeps incrementing
///   And user's lifetime keeps incrementing at a halved datebut half as fas
///
///  When user withdraws other half of tokens
///  Then user's age and lifetime stop incrementing
///
///  When user deposits tokens again later
///  Then user's age and lifetime start incrementing again
///
///  When another user deposits tokens
///  Then the first user's lifetime share starts to diminish
///
///  When user tries to withdraw too much
///  Then they can't
///
///  When a stranger tries to withdraw
///  Then they can't
#[test] fn test_deposit_withdraw_one () {
    let (ref mut deps, VK, ref SIENNA, ref LP) = entities_init();
    user(deps, "Alice")
        .at(2).locked(0u128).lifetime(0u128)
              .deposits(LP, 100u128)
              .locked(100u128).lifetime(0u128)
        .at(3).locked(100u128).lifetime(100u128)
        .at(4).locked(100u128).lifetime(200u128)
              .withdraws(LP, 50u128)
              .locked( 50u128).lifetime(200u128)
        .at(5).locked( 50u128).lifetime(250u128)
        .at(6).locked( 50u128).lifetime(300u128)
              .withdraws(LP, 50u128)
              .locked(  0u128).lifetime(300u128)
        .at(7).locked(  0u128).lifetime(300u128)
        .at(8).locked(  0u128).lifetime(300u128)
              .deposits(LP,   1u128)
              .locked(  1u128).lifetime(300u128)
        .at(9).locked(  1u128).lifetime(301u128)
        .at(10).locked( 1u128).lifetime(302);
}

/// Given an instance:
///
///  When alice and bob first deposit lp tokens simultaneously,
///  Then their ages and earnings start incrementing simultaneously;
///
///  When alice and bob withdraw lp tokens simultaneously,
///  Then their ages and earnings keep changing simultaneously;
///
///  When alice and bob's ages reach the configured threshold,
///  Then each is eligible to claim half of the available rewards
///   And their rewards are proportionate to their stakes.
#[test] fn test_deposit_withdraw_parallel () {
    let (ref mut deps, VK, SIENNA, ref LP) = entities_init();
    user(deps, "Alice").at(2).deposits(LP, 100);
    user(deps, "Bob").at(2).deposits(LP, 100);
}

/// Given an instance
///
///  When alice deposits lp tokens,
///   And alice withdraws them after reaching the threshold;
///  Then alice is eligible to claim the whole pool
///
///  When bob deposits the same amount of tokens
///  Then alice's rewards start decreasing proportionally
///
///  When bob reaches the age threshold
///  Then each is eligible to claim some rewards
#[test] fn test_deposit_withdraw_sequential () {
    let (ref mut deps, _, _, ref LP) = entities();

    user(deps, "Alice").at(2).deposits(LP, 100u128);
    user(deps, "Bob").at(2).deposits(LP, 100u128);

    //Test.at(1).init_configured(&admin)?
              //.set_vk(&alice, "")?
              //.set_vk(&bob,   "")?
              //.fund(REWARD);

    //Test.at(    1).user(&alice,   0,   0,         0,   0, 0,   0)?.deposit(&alice, 100)?
        //.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 100, 0, 100)?.withdraw(&alice, 100)?
                  //.user(&alice, DAY,   0, DAY * 100, 100, 0, 100)?

    //Test.at(           DAY+2).user(&bob,     0,   0,         0,  0, 0,  0)?.deposit(&bob, 100)?
                             //.user(&bob,     0, 100,         0,  0, 0,  0)?
        //.at(         DAY+2+1).user(&alice, DAY,   0, DAY * 100, 97, 0, 97)?
        //.at(     DAY+2+DAY/2).user(&alice, DAY,   0, DAY * 100, 43, 0, 43)?
        //.at(DAY+2+DAY/2+1000).user(&alice, DAY,   0, DAY * 100, 40, 0, 40)?

    //Test.at(         2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 49, 0, 49)?.withdraw(&bob, 100)?
                             //.user(&bob,   DAY,   0, DAY * 100, 49, 0, 49)?
                             //.user(&alice, DAY,   0, DAY * 100, 24, 0, 24)?
}

/// Given an instance
///
///  When strangers try to claim rewards
///  Then they get an error
///
///  When users provide liquidity
///   And they wait for rewards to accumulate
///   And a provider claims rewards
///  Then that provider receives reward tokens
///
///  When a provider claims rewards twice within a period
///  Then rewards are sent only the first time
///
///  When a provider claims their rewards less often
///  Then they receive equivalent rewards as long as the liquidity deposited hasn't changed
#[test] fn test_claim_one () {
    let (ref mut deps, _, ref SIENNA, ref LP) = entities_init();
    user(deps, "Alice")
        .at(2).is_unregistered()
        .at(3).deposits(LP, 100)
        .at(103).claims(SIENNA, 100)
        .at(104).must_wait(99)
        .at(105).must_wait(98)
        .at(203).claims(SIENNA, 100)
        .at(204).must_wait(99)
        .at(403).claims(SIENNA, 200);
}

#[test] fn test_claim_parallel_sequential () {
    //claim_ratio_zero {
        //given "an instance" {
            //let admin = HumanAddr::from("admin");
            //let alice = HumanAddr::from("alice");
            //let bob   = HumanAddr::from("bob");
            //Test.at(1).init_configured(&admin)? }

        //when  "strangers try to claim rewards"
        //then  "they get an error" {
            //Test.at(1).claim_must_wait(&alice, "deposit tokens for 17280 more blocks to be eligible")?
                      //.claim_must_wait(&bob,   "deposit tokens for 17280 more blocks to be eligible")? }

        //when  "users provide liquidity"
        //and   "they wait for rewards to accumulate" {
            //Test.at(1)
                //.deposit(&alice, 100)?.claim_must_wait(&alice, "deposit tokens for 17280 more blocks to be eligible")?
                //.deposit(&bob,   100)?.claim_must_wait(&bob, "deposit tokens for 17280 more blocks to be eligible")?
                //.at(2).claim_must_wait(&alice, "deposit tokens for 17279 more blocks to be eligible")?
                //.at(3).claim_must_wait(&bob,   "deposit tokens for 17278 more blocks to be eligible")?
                //.at(4).claim_must_wait(&alice, "deposit tokens for 17277 more blocks to be eligible")?
                //.at(5).claim_must_wait(&bob,   "deposit tokens for 17276 more blocks to be eligible")? }

        //and   "a provider claims rewards"
        //then  "that provider receives reward tokens" {
            //Test.fund(REWARD)
                //.set_ratio(&admin, 0u128, 1u128)?
                //.at(1 + DAY).claim_must_wait(&alice, NO_REWARDS)? }

        //when  "a provider claims rewards twice within a period"
        //then  "rewards are sent only the first time" {
            //Test.at(1 + DAY).claim_must_wait(&alice, NO_REWARDS)?
                //.at(2 + DAY).claim_must_wait(&alice, NO_REWARDS)?
                //.at(3 + DAY).claim_must_wait(&alice, NO_REWARDS)? }

        //when  "a provider claims their rewards less often"
        //then  "they receive equivalent rewards as long as the liquidity deposited hasn't changed" {
            //Test.fund(REWARD)
                //.set_ratio(&admin, 1u128, 1u128)?
                //.at(3 + DAY * 2).claim(&alice, 100)?.claim(&bob, 100)? } }

    //two_sequential_users_and_claim {
        //given "an instance" {
            //let admin = HumanAddr::from("admin");
            //let alice = HumanAddr::from("alice");
            //let bob   = HumanAddr::from("bob");
            //Test.at(1).init_configured(&admin)?
                      //.set_vk(&alice, "")?
                      //.set_vk(&bob,   "")? }

        //when "alice deposits lp tokens,"
        //and  "alice withdraws them after reaching the threshold;"
        //then "alice is eligible to claim the whole pool" {
            //Test.fund(REWARD)
                //.at(    1).user(&alice, 0, 0, 0, 0, 0, 0)?.deposit(&alice, 100)?
                //.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 100, 0, 100)?.withdraw(&alice, 100)?
                          //.user(&alice, DAY,   0, DAY * 100, 100, 0, 100)? }

        //when "bob deposits the same amount of tokens" {
            //Test.at(DAY+2).user(&bob,    0,   0, 0, 0, 0, 0)?.deposit(&bob, 100)?
                          //.user(&bob,    0, 100, 0, 0, 0, 0)? }

        //then "alice's rewards start decreasing proportionally" {
            //Test.at(DAY+2+1).user(&alice, DAY, 0, DAY * 100, 97, 0, 97)? }

        //when "alice claims some time after maturing"
        //then "alice's state is reset because of selective_memory" {
            //Test.at(     DAY+2+DAY/2).user(&alice, DAY, 0, DAY * 100, 43, 0, 43 )?.claim(&alice, 43)?
                //.at(1000+DAY+2+DAY/2).user(&alice, DAY, 0, 0, 0, 0, 0) }

        //when "bob reaches the age threshold"
        //then "bob is eligible to claim a comparable amount of rewards" {
            //Test.at(2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 49,  0, 49)?.withdraw(&bob, 100)?
                            //.user(&bob,   DAY,   0, DAY * 100, 49,  0, 49)?
                            //.user(&alice, DAY,   0, 0, 0, 0, 0)? } }
}

/// Given a pool
///
///  When a user deposits tokens
///  Then they need to keep them deposited for a fixed amount of time before they can claim
///
///  When a user claims rewards
///  Then they need to wait a fixed amount of time before they can claim again
#[test] fn test_threshold_cooldown () {
    let (ref mut deps, VK, ref SIENNA, ref LP) = entities();
    admin(deps).at(1).configure(RewardsConfig {
        lp_token:     Some(LP.link.clone()),
        reward_token: Some(SIENNA.link.clone()),
        reward_vk:    Some(VK.clone()),
        ratio:        None,
        threshold:    Some(100),
        cooldown:     Some(200),
    });;
    user(deps, "Alice")
        .at(2).deposits(LP, 100u128)
        .at(4).must_wait(98)
        .at(5).must_wait(97)
        .at(100).must_wait(2)
        .at(101).must_wait(1)
        .at(102).claims(SIENNA, 100)
        .at(103).must_wait(200)
        .at(104).must_wait(199)
        .at(300).must_wait(3)
        .at(301).must_wait(2)
        .at(302).must_wait(1)
        .at(303).claims(SIENNA, 100);
}

/// Given an instance where rewards are given in the same token that is staked
///
///  When a user deposits tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///
///  When a user withdraws tokens after claiming
///  Then they get the original amount
#[test] fn test_single_sided () {
    let (ref mut deps, VK, ref SIENNA, _) = entities();
    admin(deps).at(1).configure(RewardsConfig {
        lp_token:     Some(SIENNA.link.clone()),
        reward_token: Some(SIENNA.link.clone()),
        reward_vk:    Some(VK),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    });
    user(deps, "Alice")
        .at(2)  .deposits(SIENNA,  100u128)
        .at(103).claims(SIENNA,    100u128)
        .at(104).withdraws(SIENNA, 100u128);
}

/// Given a pool and a user
///
///  When user deposits tokens and becomes eligible for rewards
///   And user first claims rewards and then withdraws all tokens
///  Then user lifetime is preserved so they can re-stake and continue
///
///  When user deposits tokens and becomes eligible for rewards
///   And user first withdraws all tokens and then claims rewards
///  Then user lifetime and claimed is reset so they can start over
#[test] fn test_reset () {
    let (ref mut deps, _, ref SIENNA, ref LP) = entities_init();
    assert!(Rewards::handle_configure(deps, &RewardsConfig {
        lp_token:     None,
        reward_token: None,
        reward_vk:    None,
        ratio:        None,
        threshold:    Some(0u64),
        cooldown:     None,
    }).is_ok());
    user(deps, "Alice")
        .at( 2).deposits(LP,   100u128)
        .at( 4).claims(SIENNA, 100u128)
        .at( 6).withdraws(LP,  100u128).lifetime(400u128).claimed(100u128)
        .at( 8).deposits(LP,   100u128)
        .at(10).withdraws(LP,  100u128)
        .at(12).claims(SIENNA, 100u128).lifetime(0u128).claimed(0u128);

    //when  "share of user who has previously claimed rewards diminishes"
    //then  "user is crowded out"
    //and   "user can't claim" {
        //user1.deposit_tokens(100u128.into())?;
        //user1.pool.set_time(1 + crate::DAY*4);
        //user1.claim_reward()?;
        //let mut user2 = user1.pool.user(addr2.clone());
        //user2.deposit_tokens(1000u128.into())?;
        //user2.pool.set_time(1 + crate::DAY*5);
        //let mut user1 = user2.pool.user(addr1.clone());
        //assert!(user1.earned()? < user1.claimed()?);
        //assert_eq!(user1.claimable()?, Amount::zero()); }

    //when  "user withdraws all tokens"
    //then  "user's lifetime is preserved"
    //and   "crowded out users can't reset their negative claimable" {
        //user1.withdraw_tokens(100u128.into())?;
        //assert!(user1.earned()? < user1.claimed()?); }
}

/// Given a pool with some activity
///
///  When someone unauthorized tries to close the pool
///  Then they can't
///
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_close () {
    for msg in [
        RewardsHandle::Lock     { amount: 100u128.into() },
        RewardsHandle::Retrieve { amount: 100u128.into() },
    ] {
        let (ref mut deps, _VK, _SIENNA, ref LP) = entities_init();
        user(deps, "Alice").at(2).deposits(LP, 100u128);
        badman(deps).at(3).cannot_close_pool();
        user(deps, "Alice").at(4).deposits(LP, 100u128);
        admin(deps).at(5).closes_pool();
        user(deps, "Alice").at(6).test_handle(
            msg,
            Ok(HandleResponse::default()) // should be retrieval instead of lock
        );
    }
}

/// Given an instance
///
///  When non admin-tries to call release
///  Then gets rejected
///
///  When calling with reward token info
///  Then the viewing key changes
#[test] fn test_drain () {
    let (ref mut deps, _, SIENNA, _) = entities_init();
    let key = "key";
    let msg = RewardsHandle::Drain {
        snip20:    SIENNA.link.clone(),
        key:       key.into(),
        recipient: None
    };
    badman(deps).at(2).cannot_drain();
    admin(deps).at(3).drains_pool();
    assert!(Rewards::handle(deps, &badman(deps).at(2).env, msg.clone()).is_err());
    assert!(Rewards::handle(deps, &admin(deps).at(3).env, msg.clone()).is_ok());
    let vk: Option<ViewingKey> = deps.get(crate::algo::pool::REWARD_VK).unwrap();
    assert_eq!(vk.unwrap().0, String::from(key));
}

/// Given an instance with 0/1 ratio
///
///  When user becomes eligible for rewards
///  Then rewards are zero
///
///  When ratio is set to 1/2
///  Then rewards are halved
///
///  When ratio is set to 1/1
///  Then rewards are normal
///
///  When ratio is set to 2/1
///  Then rewards are doubled
#[test] fn test_global_ratio () {
    let (ref mut deps, _, ref SIENNA, ref LP) = entities_init();
    admin(deps)
        .at(1).set_ratio((0u128, 1u128));
    user(deps, "Alice")
        .at(2).deposits(LP, 100u128)
        .at(86402).claims(SIENNA, 0u128);
    admin(deps)
        .at(86403).set_ratio((1u128, 2u128));
    user(deps, "Alice")
        .at(86402).claims(SIENNA, 50u128);
    admin(deps)
        .at(10).set_ratio((1u128, 1u128));
    user(deps, "Alice")
        .at(86402).claims(SIENNA, 50u128);
    admin(deps)
        .at(1234).set_ratio((2u128, 1u128));
    user(deps, "Alice")
        .at(86402).claims(SIENNA, 50u128);
}

/// Given a pool and a user
///
///  When LP tokens have never been deposited
///  Then the pool liquidity ratio is unknown
///
///  When LP tokens are deposited
///  Then the pool liquidity ratio is 1
///
///  When some LP tokens are withdrawn
///  Then the pool liquidity ratio remains 1
///
///  When all LP tokens are withdrawn
///  Then the pool liquidity ratio begins to decrease toward 0
///
///  When some LP tokens are deposited again
///  Then the pool liquidity ratio begins to increase toward 1
///
///  When a user is eligible to claim rewards
///  Then the rewards are diminished by the pool liquidity ratio
#[test] fn test_pool_liquidity_ratio () {
    let (ref mut deps, VK, SIENNA, LP) = entities_init();

    pool_status(deps, 1).liquid(0).existed(None);

    //assert!(user.pool.liquidity_ratio().is_err());

    //user.pool.set_time(10000);
    //assert!(user.pool.liquidity_ratio().is_err());
    //user.deposit_tokens(100u128.into())?;
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(20000);
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
    //user.withdraw_tokens(50u128.into())?;
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(30000);
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
    //user.withdraw_tokens(50u128.into())?;
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
    //user.pool.set_time(50000);
    //assert_eq!(user.pool.liquidity_ratio()?,  50000000u128.into());

    //user.deposit_tokens(50u128.into())?;
    //user.pool.set_time(90000);
    //assert_eq!(user.pool.liquidity_ratio()?,  75000000u128.into());

    //user.pool.set_balance(100u128.into());
    //user.withdraw_tokens(50u128.into())?;
    //user.reset_liquidity_ratio()?;
    //assert_eq!(user.claim_reward()?, 75u128.into());
}

    fn pool_status (deps: &mut Deps, time: Time) -> PoolAssertAdapter {
        match Rewards::query_status(&*deps, time, None, None).unwrap() {
            crate::RewardsResponse::Status { pool, .. } => PoolAssertAdapter(pool),
            _ => panic!()
        }
    }

    pub struct PoolAssertAdapter(Pool);
    impl PoolAssertAdapter {
        fn liquid (&self, t: u64) -> &Self {
            assert_eq!(self.0.liquid, t, "pool.liquid");
            self
        }
        fn existed (&self, t: Option<u64>) -> &Self {
            assert_eq!(self.0.existed, t, "pool.existed");
            self
        }
    }

/// Given a pool and a user
///
///  When LP tokens have never been deposited by this user
///  Then the user's liquidity ratio is 1
///
///  When LP tokens are deposited by this user
///  Then the user's liquidity ratio remains 1
///
///  When some LP tokens are withdrawn by this user
///  Then the user's liquidity ratio remains 1
///
///  When all LP tokens are withdrawn by this user
///  Then the user's liquidity ratio begins to decrease toward 0
///
///  When LP tokens are deposited again by this user
///  Then the user's liquidity ratio begins to increase toward 1
///
///  When the user is eligible to claim rewards
///  Then the rewards are diminished by the user's liquidity ratio
#[test] fn test_user_liquidity_ratio () {
    let (ref mut deps, _, SIENNA, ref LP) = entities_init();
    let t    =   23u64;
    let r    = 5040u128;
    let half =  120u128;
    deps.querier.increment_balance(r);
    admin(deps)
        .at(t-1).set_threshold(0u64);
    user(deps, "Alice")
        .at(t  ).set_vk("")
                .liquid(0).existed(0).claimable(0u128)
                .deposits(LP, 2 * half)
                .liquid(0).existed(0).claimable(0u128)
        .at(t+1).liquid(1).existed(1).claimable(r)
        .at(t+2) // after partial withdrawal user is still present
                .liquid(2).existed(2).claimable(r)
                .withdraws(LP, half)
                .liquid(2).existed(2).claimable(r)
        .at(t+3) // after full withdraw ratio starts going down, representing the user's absence
                .liquid(3).existed(3).claimable(r)
                .withdraws(LP, half)
                .liquid(3).existed(3).claimable(r)
        .at(t+4).liquid(3).existed(4).claimable(r*3/4)
        .at(t+5).liquid(3).existed(5).claimable(r*3/5)
        .at(t+6).liquid(3).existed(6).claimable(r*3/6)
                .deposits(LP, 1u128) // then it starts increasing again once the user is back
                .liquid(3).existed(6).claimable(r*3/6)
        .at(t+7).liquid(4).existed(7).claimable(r*4/7)
        .at(t+8).liquid(5).existed(8).claimable(r*5/8)
        .at(t+9) // user has provided liquidity for 2/3rds of the time
                .liquid(6).existed(9).claimable(r*6/9);
}
