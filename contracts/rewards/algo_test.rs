#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(unreachable_patterns)]

use crate::*;
use fadroma::*;
use fadroma::scrt_contract_harness::*;
use fadroma::secret_toolkit::snip20;
use fadroma::testing::*;

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

fn context () -> (
    Extern<MemoryStorage, MockApi, MockQuerier>,
    String,
    ContractLink<HumanAddr>,
    ContractLink<HumanAddr>,
    fn () -> Env,
    fn () -> Env,
    fn () -> Env,
) {
    (
        mock_dependencies(10, &[]),
        "reward_vk".to_string(),
        ContractLink {
            address:   HumanAddr::from("reward_addr"),
            code_hash: "reward_hash".into()
        },
        ContractLink {
            address:   HumanAddr::from("lp_addr"),
            code_hash: "lp_hash".into()
        },
        || { mock_env("Admin",  &[]) },
        || { mock_env("Badman", &[]) },
        || { mock_env("User",   &[]) },
    )
}

/// Given no instance
///  When the admin inits an instance with a configured reward token
///  Then the default values are used where applicable
///   And the rewards module emits a message that sets the reward token viewing key
#[test] fn test_init () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, user) = context();

    assert_eq!(
        Rewards::init(deps, &admin(), RewardsConfig {
            lp_token:     None,
            reward_token: Some(reward_token),
            reward_vk:    Some(reward_vk),
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        }),
        Ok(Some(snip20::set_viewing_key_msg(
            reward_vk,
            None, BLOCK_SIZE,
            reward_token.code_hash.clone(),
            reward_token.address.clone()
        )))
    );
}

/// Given no instance
///  When the admin inits an instance with an empty configuration
///  Then the default values are used where applicable
///   And no viewing key config message is returned
///  When someone else tries to set the config
///  Then the config remains unchanged
///  When the admin sets the config, including a reward token
///  Then a reward token viewing key config message is returned
#[test] fn test_configure () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, user) = context();

    assert_eq!(Rewards::init(deps, &admin(), RewardsConfig {
        lp_token:     None,
        reward_token: None,
        reward_vk:    None,
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }), Ok(None));

    assert_eq!(Rewards::handle(deps, badman(), RewardsHandle::Configure(RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    })), Err(StdError::unauthorized()));

    assert_eq!(Rewards::handle(deps, admin(), RewardsHandle::Configure(RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    })), Ok(HandleResponse {
        messages: vec![
            snip20::set_viewing_key_msg(
                reward_vk,
                None, BLOCK_SIZE,
                reward_token.code_hash.clone(),
                reward_token.address.clone()
            ).unwrap()
        ],
        data: None,
        log: vec![],
    }));
}

#[test] fn test_lock_retrieve () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, user) = context();
}

#[test] fn test_claim () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, user) = context();
}

/// Given a pool
///  When a user locks tokens
///  Then they need to keep them locked for a fixed amount of time before they can claim
///  When a user claims rewards
///  Then they need to wait a fixed amount of time before they can claim again
#[test] fn test_threshold_cooldown () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, user) = context();

    let threshold = 100u64;

    assert!(Rewards::init(deps, &admin(), RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    Some(threshold),
        cooldown:     None,
    }).is_ok());

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Claim {
    }), Err(StdError::unauthorized()));

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Claim {
    }), Err(StdError::unauthorized()));

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

}

/// Given an instance where rewards are given in the same token that is staked
///  When a user locks tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///  When a user retrieves tokens after claiming
///  Then they get the original amount
#[test] fn test_single_sided () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, user) = context();

    assert!(Rewards::init(deps, &admin(), RewardsConfig {
        lp_token:     Some(reward_token),
        reward_token: Some(reward_token),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user(), RewardsHandle::Retrieve {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));
}

/// Given a pool and a user
///  When user locks tokens and becomes eligible for rewards
///   And user first claims rewards and then unlocks all tokens
///  Then user lifetime is preserved so they can re-stake and continue
///  When user locks tokens and becomes eligible for rewards
///   And user first unlocks all tokens and then claims rewards
///  Then user lifetime and claimed is reset so they can start over
#[test] fn test_reset () {
    let ref mut deps = mock_dependencies(10, &[]);
    let env = mock_env("Admin", &[]);

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
        assert!(user1.earned()? < user1.claimed()?); }

}

/// Given a pool with some activity
///  When someone unauthorized tries to close the pool
///  Then they can't
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_close () {

    let ref mut deps = mock_dependencies(10, &[]);
    let env = mock_env("Admin", &[]);

    let admin = HumanAddr::from("Admin");
    let alice = HumanAddr::from("Alice");

    let reward_vk    = "something".to_string();
    let reward_token = ContractLink {
        address:   HumanAddr::from("address"),
        code_hash: "code_hash".into()
    };

    assert!(
        Rewards::init(deps, &env, RewardsConfig {
            lp_token:     None,
            reward_token: Some(reward_token),
            reward_vk:    Some(reward_vk),
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        }).is_ok()
    );

    assert_eq!(
        Rewards::handle(deps, env, RewardsHandle::Lock { amount: 100u128.into() }),
        Ok(HandleResponse::default())
    );

    assert_eq!(
        Rewards::handle(deps, mock_env("alice", &[]), RewardsHandle::Close { message: String::from("closed") }),
        Err(StdError::unauthorized())
    );

    assert_eq!(
        Rewards::handle(deps, env, RewardsHandle::Lock { amount: 100u128.into() }),
        Ok(HandleResponse::default())
    );

    assert_eq!(
        Rewards::handle(deps, env, RewardsHandle::Close { message: String::from("closed") }),
        Ok(HandleResponse::default())
    );

    assert_eq!(
        Rewards::handle(deps, env, RewardsHandle::Lock { amount: 100u128.into() }),
        Ok(HandleResponse::default())
    );

}

/// Given an instance
///  When non admin-tries to call release
///  Then gets rejected
///  When calling with reward token info
///  Then the viewing key changes
#[test] fn test_drain () {

    let admin = HumanAddr::from("admin");
    let alice = HumanAddr::from("alice");

    let key = "key";

    let msg = RewardsHandle::Drain {
        snip20:    Test.reward_token(),
        key:       key.into(),
        recipient: None
    };

    Test.at(1).init_configured(&admin)?

    assert!(Test.tx(20, &alice, msg.clone()).is_err());

    assert!(Test.tx(20, &admin, msg).is_ok());
    let deps = Test.deps();
    let vk = load_viewing_key(&deps.storage)?;

    assert_eq!(vk.0, String::from(key));

}

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
const DAY:        u128 = crate::DAY as u128;
const NO_REWARDS: &str = "You've already received as much as your share of the reward pool allows. Keep your liquidity tokens locked and wait for more rewards to be vested, and/or lock more liquidity tokens to grow your share of the reward pool.";
const PORTION:    u128 = 100;
const REWARD:     u128 = 100;
const STAKE:      u128 = 100;

kukumba_harnessed! {

    type Error = StdError;

    let Test: RewardsHarness<RewardsMockQuerier>;

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

    global_ratio_zero_2 {

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

}

/// Unit testing harness for Sienna Rewards.

pub struct RewardsMockQuerier {
    pub balance: Uint128
}

impl RewardsMockQuerier {
    fn mock_query_dispatch (
        &self,
        _contract: &ContractLink<HumanAddr>,
        msg:       &Snip20Query
    ) -> Snip20QueryAnswer {
        match msg {
            Snip20Query::Balance { .. } => {
                //if contract != self.reward_token {
                    //panic!("MockSnip20Querier: Expected balance query for {:?}", self.reward_token)
                //}
                Snip20QueryAnswer::Balance { amount: self.balance }
            },

            _ => unimplemented!()
        }
    }
    pub fn increment_balance (&mut self, amount: u128) {
        self.balance = self.balance + amount.into();
    }
    pub fn decrement_balance (&mut self, amount: u128) -> StdResult<()> {
        self.balance = (self.balance - amount.into())?;
        Ok(())
    }
}

impl Querier for RewardsMockQuerier {
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                let error = format!("Parsing query request: {}", e);
                let request = bin_request.into();
                return Err(SystemError::InvalidRequest { error, request })
            }
        };
        match request {
            QueryRequest::Wasm(WasmQuery::Smart { callback_code_hash, contract_addr, msg }) => {
                Ok(to_binary(&self.mock_query_dispatch(&ContractLink {
                    code_hash: callback_code_hash,
                    address: contract_addr
                }, &from_binary(&msg).unwrap())))
            },
            _ => panic!("MockSnip20Querier: Expected WasmQuery::Smart.")
        }
    }
}
