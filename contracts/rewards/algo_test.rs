#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(unreachable_patterns)]

use crate::*;
use fadroma::*;
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

// duration of rewards period as u128 instead of u64
// to allow in-place (DAY * Amount) volume calculations
// (volume is also represented as u128 instead of u256)
// i.e. need to call .into(), harness up/downcasts accordingly
//const DAY:        u128 = crate::DAY as u128;
//const NO_REWARDS: &str = "You've already received as much as your share of the reward pool allows. Keep your liquidity tokens locked and wait for more rewards to be vested, and/or lock more liquidity tokens to grow your share of the reward pool.";
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
    let (ref mut deps, reward_vk, reward_token, _, admin, _, _) = context();

    assert!(
        Rewards::init(deps, &admin(1), RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        }).is_err(),
    );

    assert_eq!(
        Rewards::init(deps, &admin(1), RewardsConfig {
            lp_token:     None,
            reward_token: Some(reward_token.link.clone()),
            reward_vk:    Some(reward_vk.clone()),
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        }).unwrap(),
        Some(snip20::set_viewing_key_msg(
            reward_vk,
            None, BLOCK_SIZE,
            reward_token.link.code_hash.clone(),
            reward_token.link.address.clone()
        ).unwrap())
    );
}

    // Helpers will be indented 1 level above the test cases

    pub struct RewardsMockQuerier { pub balance: Amount }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all="snake_case")]
    pub enum Snip20Query { Balance {} }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all="snake_case")]
    pub enum Snip20Response { Balance { amount: Amount } }

    impl Querier for RewardsMockQuerier {
        fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => unimplemented!()
            };
            match request {
                QueryRequest::Wasm(WasmQuery::Smart { callback_code_hash, contract_addr, msg }) => {
                    Ok(to_binary(&self.mock_query_dispatch(&ContractLink {
                        code_hash: callback_code_hash,
                        address:   contract_addr
                    }, &from_binary(&msg).unwrap())))
                },
                _ => unimplemented!()
            }
        }
    }

    impl RewardsMockQuerier {
        fn mock_query_dispatch(
            &self, _: &ContractLink<HumanAddr>, msg: &Snip20Query
        ) -> Snip20Response {
            match msg {
                Snip20Query::Balance { .. } => Snip20Response::Balance { amount: self.balance },
                _ => unimplemented!()
            }
        }
        pub fn increment_balance (&mut self, amount: u128) -> () {
            self.balance += amount.into();
        }
        pub fn decrement_balance (&mut self, amount: u128) -> StdResult<()> {
            self.balance = (self.balance - amount.into())?;
            Ok(())
        }
    }

    type Deps = Extern<MemoryStorage, MockApi, RewardsMockQuerier>;

    type Context = (
        Deps,                  // deps
        String,                // reward_vk
        ISnip20,               // reward_token
        ISnip20,               // lp_token
        fn (u64) -> Env,       // admin env - always init contract with this
        fn (u64) -> Env,       // badman env - never register in the contract
        fn (&str, u64) -> Env, // user envs - pass
    );

    fn context () -> Context {
        (
            Extern {
                storage: MemoryStorage::default(),
                api:     MockApi::new(20),
                querier: RewardsMockQuerier { balance: 0u128.into() }
            },
            "reward_vk".to_string(),
            ISnip20::attach(
                ContractLink { address: HumanAddr::from("reward_addr"), code_hash: "reward_hash".into() }
            ),
            ISnip20::attach(
                ContractLink { address: HumanAddr::from("lp_addr"),     code_hash: "lp_hash".into() }
            ),
            |t: u64| env(&HumanAddr::from("Admin"),  t),
            |t: u64| env(&HumanAddr::from("Badman"), t),
            |id: &str, t: u64| env(&HumanAddr::from(format!("User{}", id)), t),
        )
    }

    fn context_init () -> Context {
        let mut context = context();
        assert_eq!(
            Rewards::init(&mut context.0, &context.4(1), RewardsConfig {
                lp_token:     Some(context.3.link.clone()),
                reward_token: Some(context.2.link.clone()),
                reward_vk:    Some(context.1.clone()),
                ratio:        None,
                threshold:    None,
                cooldown:     None,
            }).unwrap(),
            Some(snip20::set_viewing_key_msg(
                context.1.clone(),
                None, BLOCK_SIZE,
                context.2.link.code_hash.clone(),
                context.2.link.address.clone()
            ).unwrap())
        );
        context
    }

    fn env (signer: &HumanAddr, time: u64) -> Env {
        let mut env = mock_env(signer, &[]);
        env.block.time = time;
        env
    }

// And more test cases, with gradually fewer helper functions as the defined ones are reused //////

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
    let (ref mut deps, reward_vk, reward_token, _, admin, badman, _) = context();

    assert_eq!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk.clone()),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }), Ok(Some(snip20::set_viewing_key_msg(
        reward_vk.clone(),
        None, BLOCK_SIZE,
        reward_token.link.code_hash.clone(),
        reward_token.link.address.clone()
    ).unwrap())));

    assert_eq!(Rewards::handle(deps, admin(2), RewardsHandle::Configure(RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk.clone()),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    })), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, badman(3), RewardsHandle::Configure(RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk.clone()),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    })), Err(StdError::unauthorized()));

    assert_eq!(Rewards::handle(deps, admin(4), RewardsHandle::Configure(RewardsConfig {
        lp_token:     None,
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk.clone()),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    })), Ok(HandleResponse {
        messages: vec![
            snip20::set_viewing_key_msg(
                reward_vk,
                None, BLOCK_SIZE,
                reward_token.link.code_hash.clone(),
                reward_token.link.address.clone()
            ).unwrap()
        ],
        data: None,
        log: vec![],
    }));
}

/// Given an instance
///
///  When user first deposits
///  Then user's age and lifetime start incrementing
///
///  When user retrieves half of the tokens
///  Then user's age keeps incrementing
///   And user's lifetime keeps incrementing at a halved datebut half as fas
///
///  When user retrieves other half of tokens
///  Then user's age and lifetime stop incrementing
///
///  When user locks tokens again later
///  Then user's age and lifetime start incrementing again
///
///  When another user locks tokens
///  Then the first user's lifetime share starts to diminish
///
///  When user tries to withdraw too much
///  Then they can't
///
///  When a stranger tries to withdraw
///  Then they can't
#[test] fn test_lock_retrieve_one () {
    let (ref mut deps, reward_vk, ref reward_token, ref lp_token, admin, badman, user) = context();

    assert!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(lp_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    deposit(deps, lp_token, user("Alice", 2), 100u128);

    successful_withdrawal(deps, lp_token, user("Alice", 4), 50u128);

    successful_withdrawal(deps, lp_token, user("Alice", 6), 50u128);

    deposit(deps, lp_token, user("Alice", 8), 100u128);

    deposit(deps, lp_token, user("Alice", 10), 100u128);

    successful_withdrawal(deps, lp_token, user("Alice", 12), 50u128);

    successful_withdrawal(deps, lp_token, user("Alice", 14), 50u128);

}

    fn deposit (deps: &mut Deps, lp_token: &ISnip20, env: Env, amount: u128) {
        assert_eq!(
            Rewards::handle(deps, env.clone(), RewardsHandle::Lock { amount: amount.into() }),
            HandleResponse::default().msg(lp_token.transfer_from(
                &env.message.sender,
                &env.contract.address,
                amount.into()
            ).unwrap()));
    }

    fn successful_withdrawal (deps: &mut Deps, lp_token: &ISnip20, env: Env, amount: u128) {
        assert_eq!(
            Rewards::handle(deps, env.clone(), RewardsHandle::Lock { amount: amount.into() }),
            HandleResponse::default().msg(lp_token.transfer(
                &env.message.sender,
                amount.into()
            ).unwrap()));
    }

/// Given an instance:
///
///  When alice and bob first lock lp tokens simultaneously,
///  Then their ages and earnings start incrementing simultaneously;
///
///  When alice and bob retrieve lp tokens simultaneously,
///  Then their ages and earnings keep changing simultaneously;
///
///  When alice and bob's ages reach the configured threshold,
///  Then each is eligible to claim half of the available rewards
///   And their rewards are proportionate to their stakes.
#[test] fn test_lock_retrieve_parallel () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, _, user) = context();

    assert!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(lp_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    deposit(deps, &lp_token, user("Alice", 2), 100u128);

    deposit(deps, &lp_token, user("Bob",   2), 200u128);


    //Test.at(1).init_configured(&admin)?
              //.fund(REWARD)
              //.set_vk(&alice, "")?
              //.set_vk(&bob,   "")?
              //.user(&alice, 0, 0, 0, 0, 0, 0)?
              //.user(&bob,   0, 0, 0, 0, 0, 0)?

    //Test.at(1).user(&alice, 0,   0,   0, 0,  0, 0)?.lock(&alice, 100)?;
    //Test.at(1).user(&bob,   0,   0,   0, 0,  0, 0)?.lock(&bob,   100)?;
    //Test.at(1).user(&alice, 0, 100,   0, 0,  0, 0)?;
    //Test.at(1).user(&bob,   0, 100,   0, 0,  0, 0)?;
    //Test.at(2).user(&alice, 1, 100, 100, 50, 0, 0)?;
    //Test.at(2).user(&bob,   1, 100, 100, 50, 0, 0)?;
    //Test.at(3).user(&alice, 2, 100, 200, 50, 0, 0)?;
    //Test.at(3).user(&bob,   2, 100, 200, 50, 0, 0)?;

    //Test.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 50, 0, 50)?
                  //.user(&bob,   DAY, 100, DAY * 100, 50, 0, 50)?
}

/// Given an instance
///
///  When alice locks lp tokens,
///   And alice retrieves them after reaching the threshold;
///  Then alice is eligible to claim the whole pool
///
///  When bob locks the same amount of tokens
///  Then alice's rewards start decreasing proportionally
///
///  When bob reaches the age threshold
///  Then each is eligible to claim some rewards
#[test] fn test_lock_retrieve_sequential () {
    let (ref mut deps, _, _, _, _, _, user) = context();

    assert_eq!(Rewards::handle(deps, user("Alice", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Bob", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    //Test.at(1).init_configured(&admin)?
              //.set_vk(&alice, "")?
              //.set_vk(&bob,   "")?
              //.fund(REWARD);

    //Test.at(    1).user(&alice,   0,   0,         0,   0, 0,   0)?.lock(&alice, 100)?
        //.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 100, 0, 100)?.retrieve(&alice, 100)?
                  //.user(&alice, DAY,   0, DAY * 100, 100, 0, 100)?

    //Test.at(           DAY+2).user(&bob,     0,   0,         0,  0, 0,  0)?.lock(&bob, 100)?
                             //.user(&bob,     0, 100,         0,  0, 0,  0)?
        //.at(         DAY+2+1).user(&alice, DAY,   0, DAY * 100, 97, 0, 97)?
        //.at(     DAY+2+DAY/2).user(&alice, DAY,   0, DAY * 100, 43, 0, 43)?
        //.at(DAY+2+DAY/2+1000).user(&alice, DAY,   0, DAY * 100, 40, 0, 40)?

    //Test.at(         2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 49, 0, 49)?.retrieve(&bob, 100)?
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
///  Then they receive equivalent rewards as long as the liquidity locked hasn't changed
#[test] fn test_claim () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, _, user) = context_init();

    assert_eq!(Rewards::handle(deps, user("Alice", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 3), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    //claim {
            //let admin = HumanAddr::from("admin");
            //let alice = HumanAddr::from("alice");
            //let bob   = HumanAddr::from("bob");
            //Test.at(1).init_configured(&admin)? }

            //Test.at(1).claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                      //.claim_must_wait(&bob,   "lock tokens for 17280 more blocks to be eligible")? }

            //Test.at(1)
                //.lock(&alice, 100)?.claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                //.lock(&bob,   100)?.claim_must_wait(&bob, "lock tokens for 17280 more blocks to be eligible")?
                //.at(2).claim_must_wait(&alice, "lock tokens for 17279 more blocks to be eligible")?
                //.at(3).claim_must_wait(&bob,   "lock tokens for 17278 more blocks to be eligible")?
                //.at(4).claim_must_wait(&alice, "lock tokens for 17277 more blocks to be eligible")?
                //.at(5).claim_must_wait(&bob,   "lock tokens for 17276 more blocks to be eligible")? }

            //Test.fund(REWARD)
                //.at(1 + DAY).claim(&alice, 50)? }

            //Test.at(1 + DAY).claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                //.at(2 + DAY).claim_must_wait(&alice, "lock tokens for 17279 more blocks to be eligible")?
                //.at(3 + DAY).claim_must_wait(&alice, "lock tokens for 17278 more blocks to be eligible")? }

            //Test.fund(REWARD)
                //.at(3 + DAY * 2).claim(&alice, 50)?.claim(&bob, 100)? } }

    //claim_ratio_zero {
        //given "an instance" {
            //let admin = HumanAddr::from("admin");
            //let alice = HumanAddr::from("alice");
            //let bob   = HumanAddr::from("bob");
            //Test.at(1).init_configured(&admin)? }

        //when  "strangers try to claim rewards"
        //then  "they get an error" {
            //Test.at(1).claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                      //.claim_must_wait(&bob,   "lock tokens for 17280 more blocks to be eligible")? }

        //when  "users provide liquidity"
        //and   "they wait for rewards to accumulate" {
            //Test.at(1)
                //.lock(&alice, 100)?.claim_must_wait(&alice, "lock tokens for 17280 more blocks to be eligible")?
                //.lock(&bob,   100)?.claim_must_wait(&bob, "lock tokens for 17280 more blocks to be eligible")?
                //.at(2).claim_must_wait(&alice, "lock tokens for 17279 more blocks to be eligible")?
                //.at(3).claim_must_wait(&bob,   "lock tokens for 17278 more blocks to be eligible")?
                //.at(4).claim_must_wait(&alice, "lock tokens for 17277 more blocks to be eligible")?
                //.at(5).claim_must_wait(&bob,   "lock tokens for 17276 more blocks to be eligible")? }

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
        //then  "they receive equivalent rewards as long as the liquidity locked hasn't changed" {
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

        //when "alice locks lp tokens,"
        //and  "alice retrieves them after reaching the threshold;"
        //then "alice is eligible to claim the whole pool" {
            //Test.fund(REWARD)
                //.at(    1).user(&alice, 0, 0, 0, 0, 0, 0)?.lock(&alice, 100)?
                //.at(DAY+1).user(&alice, DAY, 100, DAY * 100, 100, 0, 100)?.retrieve(&alice, 100)?
                          //.user(&alice, DAY,   0, DAY * 100, 100, 0, 100)? }

        //when "bob locks the same amount of tokens" {
            //Test.at(DAY+2).user(&bob,    0,   0, 0, 0, 0, 0)?.lock(&bob, 100)?
                          //.user(&bob,    0, 100, 0, 0, 0, 0)? }

        //then "alice's rewards start decreasing proportionally" {
            //Test.at(DAY+2+1).user(&alice, DAY, 0, DAY * 100, 97, 0, 97)? }

        //when "alice claims some time after maturing"
        //then "alice's state is reset because of selective_memory" {
            //Test.at(     DAY+2+DAY/2).user(&alice, DAY, 0, DAY * 100, 43, 0, 43 )?.claim(&alice, 43)?
                //.at(1000+DAY+2+DAY/2).user(&alice, DAY, 0, 0, 0, 0, 0) }

        //when "bob reaches the age threshold"
        //then "bob is eligible to claim a comparable amount of rewards" {
            //Test.at(2*DAY+2).user(&bob,   DAY, 100, DAY * 100, 49,  0, 49)?.retrieve(&bob, 100)?
                            //.user(&bob,   DAY,   0, DAY * 100, 49,  0, 49)?
                            //.user(&alice, DAY,   0, 0, 0, 0, 0)? } }
}

/// Given a pool
///
///  When a user locks tokens
///  Then they need to keep them locked for a fixed amount of time before they can claim
///
///  When a user claims rewards
///  Then they need to wait a fixed amount of time before they can claim again
#[test] fn test_threshold_cooldown () {
    let (ref mut deps, reward_vk, ref reward_token, ref lp_token, admin, _, user) = context();

    assert_eq!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(lp_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk.clone()),
        ratio:        None,
        threshold:    Some(100),
        cooldown:     Some(200),
    }), Ok(Some(snip20::set_viewing_key_msg(
        reward_vk.clone(),
        None, BLOCK_SIZE,
        reward_token.link.code_hash.clone(),
        reward_token.link.address.clone()
    ).unwrap())));

    deposit(deps, lp_token, user("Alice", 2), 100u128);

    claim_must_wait(deps, user("Alice", 4), 98);
    claim_must_wait(deps, user("Alice", 5), 97);
    // ...
    claim_must_wait(deps, user("Alice", 100), 2);
    claim_must_wait(deps, user("Alice", 101), 1);

    successful_claim(deps, reward_token, user("Alice", 102), 100);

    claim_must_wait(deps, user("Alice", 103), 200);
    claim_must_wait(deps, user("Alice", 104), 199);
    // ...
    claim_must_wait(deps, user("Alice", 300), 3);
    claim_must_wait(deps, user("Alice", 301), 2);
    claim_must_wait(deps, user("Alice", 302), 1);

    successful_claim(deps, reward_token, user("Alice", 303), 100);
}

    fn successful_claim (deps: &mut Deps, reward_token: &ISnip20, env: Env, amount: u128) {
        assert_eq!(
            Rewards::handle(deps, env.clone(), RewardsHandle::Claim {}),
            HandleResponse::default().msg(reward_token.transfer(
                &env.message.sender,
                amount.into()
            ).unwrap()));
    }

    fn claim_must_wait (deps: &mut Deps, env: Env, remaining: u64) {
        assert_eq!(
            Rewards::handle(deps, env.clone(), RewardsHandle::Claim {}),
            Err(StdError::generic_err(
                format!("lock tokens for {} more blocks to be eligible", remaining))))
    }

/// Given an instance where rewards are given in the same token that is staked
///
///  When a user locks tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///
///  When a user retrieves tokens after claiming
///  Then they get the original amount
#[test] fn test_single_sided () {
    let (ref mut deps, reward_vk, reward_token, _lp_token, admin, _, user) = context();

    assert!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(reward_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    assert_eq!(Rewards::handle(deps, user("Alice", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 3), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 4), RewardsHandle::Retrieve {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));
}

/// Given a pool and a user
///
///  When user locks tokens and becomes eligible for rewards
///   And user first claims rewards and then unlocks all tokens
///  Then user lifetime is preserved so they can re-stake and continue
///
///  When user locks tokens and becomes eligible for rewards
///   And user first unlocks all tokens and then claims rewards
///  Then user lifetime and claimed is reset so they can start over
#[test] fn test_reset () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, _, user) = context();

    assert!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(lp_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    assert_eq!(Rewards::handle(deps, user("Alice", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 4), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 6), RewardsHandle::Retrieve {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 8), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 10), RewardsHandle::Retrieve {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 12), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    //when  "share of user who has previously claimed rewards diminishes"
    //then  "user is crowded out"
    //and   "user can't claim" {
        //user1.lock_tokens(100u128.into())?;
        //user1.pool.set_time(1 + crate::DAY*4);
        //user1.claim_reward()?;
        //let mut user2 = user1.pool.user(addr2.clone());
        //user2.lock_tokens(1000u128.into())?;
        //user2.pool.set_time(1 + crate::DAY*5);
        //let mut user1 = user2.pool.user(addr1.clone());
        //assert!(user1.earned()? < user1.claimed()?);
        //assert_eq!(user1.claimable()?, Amount::zero()); }

    //when  "user unlocks all tokens"
    //then  "user's lifetime is preserved"
    //and   "crowded out users can't reset their negative claimable" {
        //user1.retrieve_tokens(100u128.into())?;
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
    let (ref mut deps, reward_vk, reward_token, _, admin, badman, user) = context_init();

    assert_eq!(Rewards::handle(deps, user("Alice", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, badman(3), RewardsHandle::Close {
        message: String::from("closed")
    }), Err(StdError::unauthorized()));

    assert_eq!(Rewards::handle(deps, user("Alice", 4), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, admin(5), RewardsHandle::Close {
        message: String::from("closed")
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 6), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));
}

/// Given an instance
///
///  When non admin-tries to call release
///  Then gets rejected
///
///  When calling with reward token info
///  Then the viewing key changes
#[test] fn test_drain () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, badman, _) = context();

    assert!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(lp_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk),
        ratio:        None,
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    let key = "key";
    let msg = RewardsHandle::Drain {
        snip20:    reward_token.link.clone(),
        key:       key.into(),
        recipient: None
    };

    assert!(Rewards::handle(deps, badman(2), msg.clone()).is_err());

    assert!(Rewards::handle(deps, admin(3), msg.clone()).is_ok());

    let vk: Option<ViewingKey> = deps.get(crate::algo::pool::REWARD_VK).unwrap();
    assert_eq!(vk.unwrap().0, String::from(key));
}

/// Given an instance with 0/1 ratio
///
///  When user becomes eligible for rewards
///  Then rewards are zero
///
///  When ratio is set to 1/1
///  Then rewards are normal
#[test] fn test_global_ratio () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, _, user) = context();

    assert!(Rewards::init(deps, &admin(1), RewardsConfig {
        lp_token:     Some(lp_token.link.clone()),
        reward_token: Some(reward_token.link.clone()),
        reward_vk:    Some(reward_vk.clone()),
        ratio:        Some((0u128.into(), 1u128.into())),
        threshold:    None,
        cooldown:     None,
    }).is_ok());

    assert_eq!(Rewards::handle(deps, user("Alice", 2), RewardsHandle::Lock {
        amount: 100u128.into()
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 4), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 6), RewardsHandle::Configure(RewardsConfig {
        lp_token:     None,
        reward_token: None,
        reward_vk:    None,
        ratio:        Some((1u128.into(), 1u128.into())),
        threshold:    None,
        cooldown:     None,
    })), Ok(HandleResponse::default()));

    assert_eq!(Rewards::handle(deps, user("Alice", 8), RewardsHandle::Claim {
    }), Ok(HandleResponse::default()));
}

/// Given a pool and a user
///
///  When LP tokens have never been locked
///  Then the pool liquidity ratio is unknown
///
///  When LP tokens are locked
///  Then the pool liquidity ratio is 1
///
///  When some LP tokens are unlocked
///  Then the pool liquidity ratio remains 1
///
///  When all LP tokens are unlocked
///  Then the pool liquidity ratio begins to decrease toward 0
///
///  When some LP tokens are locked again
///  Then the pool liquidity ratio begins to increase toward 1
///
///  When a user is eligible to claim rewards
///  Then the rewards are diminished by the pool liquidity ratio
#[test] fn test_pool_liquidity_ratio () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, _, _) = context_init();

    match Rewards::query(deps, RewardsQuery::Status { at: 1, address: None, key: None }).unwrap() {
        crate::algo::RewardsResponse::Status { pool, .. } => assert_eq!(pool.liquid, pool.existed.unwrap()),
        _ => unimplemented!()
    }

    //assert!(user.pool.liquidity_ratio().is_err());

    //user.pool.set_time(10000);
    //assert!(user.pool.liquidity_ratio().is_err());
    //user.lock_tokens(100u128.into())?;
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(20000);
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
    //user.retrieve_tokens(50u128.into())?;
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(30000);
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
    //user.retrieve_tokens(50u128.into())?;
    //assert_eq!(user.pool.liquidity_ratio()?, 100000000u128.into());
    //user.pool.set_time(50000);
    //assert_eq!(user.pool.liquidity_ratio()?,  50000000u128.into());

    //user.lock_tokens(50u128.into())?;
    //user.pool.set_time(90000);
    //assert_eq!(user.pool.liquidity_ratio()?,  75000000u128.into());

    //user.pool.set_balance(100u128.into());
    //user.retrieve_tokens(50u128.into())?;
    //user.reset_liquidity_ratio()?;
    //assert_eq!(user.claim_reward()?, 75u128.into());
}

/// Given a pool and a user
///
///  When LP tokens have never been locked by this user
///  Then the user's liquidity ratio is 1
///
///  When LP tokens are locked by this user
///  Then the user's liquidity ratio remains 1
///
///  When some LP tokens are unlocked by this user
///  Then the user's liquidity ratio remains 1
///
///  When all LP tokens are unlocked by this user
///  Then the user's liquidity ratio begins to decrease toward 0
///
///  When LP tokens are locked again by this user
///  Then the user's liquidity ratio begins to increase toward 1
///
///  When the user is eligible to claim rewards
///  Then the rewards are diminished by the user's liquidity ratio
#[test] fn test_user_liquidity_ratio () {
    let (ref mut deps, reward_vk, reward_token, lp_token, admin, _, _) = context_init();

    //let mut user = pool.user(addr.clone());

    //assert_eq!(user.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(10000);
    //user.lock_tokens(100u128.into())?;
    //assert_eq!(user.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(20000);
    //user.retrieve_tokens(50u128.into())?;
    //assert_eq!(user.liquidity_ratio()?, 100000000u128.into());
    //user.pool.set_time(30000);
    //user.retrieve_tokens(50u128.into())?;
    //assert_eq!(user.liquidity_ratio()?, 100000000u128.into());

    //user.pool.set_time(40000);
    //assert_eq!(user.liquidity_ratio()?,  66666666u128.into());

    //user.pool.set_time(50000);
    //user.lock_tokens(50u128.into())?;
    //assert_eq!(user.liquidity_ratio()?,  50000000u128.into());

    //user.pool.set_time(90000);
    //assert_eq!(user.liquidity_ratio()?,  75000000u128.into());

    //user.retrieve_tokens(50u128.into())?;
    //user.pool.set_balance(100u128.into());

    //user.pool.reset_liquidity_ratio()?;

    //assert_eq!(user.claim_reward()?, 75u128.into());
}
