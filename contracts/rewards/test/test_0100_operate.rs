use crate::test::{*, Context};

#[test] fn test_0101_empty () {
    let mut context = Context::named("0101_defaults");
    let bonding = context.bonding;
    // Given an instance
    context.init()
        // When nobody has deposited yet
        // And the status of a user is queried
        .later().user("Alice").set_vk("")
            // Then their stake is 0
            .staked(0)
            // And their volume is 0
            .volume(0)
            // And their entry is 0
            .entry(0)
            // And their bonding is max
            .bonding(bonding);
}

#[test] fn test_0102_stake_volume () {
    let mut context = Context::named("0102_stake");
    let stake1    = context.rng.gen_range(1..100);
    let stake2    = context.rng.gen_range(1..100);
    let n_ticks_1 = 10;
    let n_ticks_2 = 20;
    // Given an instance
    context.init().later()
        // When user deposits
        .user("Alice").set_vk("").deposits(stake1)
            // Then user's stake increments
            .staked(stake1)
            .volume(0)
            // And user's liquidity starts incrementing from the next tick
            .during(n_ticks_1, |i, context|{
                context
                    .staked(stake1)
                    .volume(i as u128 * stake1);
            })
        // When user deposits again
        .deposits(stake2)
            // Then user's stake increments
            .staked(stake1 + stake2)
            .volume(n_ticks_1 as u128 * stake1)
            // And user's liquidity starts incrementing at a faster rate
            .during(n_ticks_2, |i, context|{
                context
                    .staked(stake1 + stake2)
                    .volume(n_ticks_1 as u128 * stake1
                                  + i as u128 * (stake1 + stake2));
            });
}

#[test] fn test_0103_entry () {
    let mut context = Context::named("0103_entry");
    let stake1  = context.rng.gen_range(1..100000);
    let stake2  = context.rng.gen_range(1..100000);
    let n_ticks = 10;
    // Given an instance
    context.init()
        // When the first user hasn't deposited
        .user("Alice").set_vk("")
            // Then their entry is 0, corresponding to the initially empty pool
            .entry(0)
            // When the first user deposits
            .deposits(stake1)
            // Then their entry becomes fixed at its current value
            .entry(0)
            // When some time passes
            // Then the pool's liquidity is equal to the user's liquidity
            .after(n_ticks)
                .entry(0)
                .volume(stake1 * n_ticks as u128)
                .pool_volume(stake1 * n_ticks as u128)
        // When a subsequent user hasn't deposited
        .user("Bob").set_vk("")
            // Then their entry is equal to the curent volume of the pool
            .entry(stake1 * n_ticks as u128)
            .after(n_ticks).entry(stake1 * n_ticks as u128 * 2)
            // When a subsequent user deposits
            .deposits(stake2)
            // Then their entry becomes fixed at its current value
            .entry(stake1 * n_ticks as u128 * 2)
            // And their liquidity starts incrementing from the next tick
            .during(n_ticks, |i, context| {
                context
                    .staked(stake2)
                    .volume(stake2 * i as u128)
                    .entry(stake1 * n_ticks as u128 * 2);
            })
            .entry(stake1 * n_ticks as u128 * 2);
}

#[test] fn test_0104_bonding () {
    let mut context = Context::named("0104_bonding");
    let bonding = context.bonding;
    let stake   = context.rng.gen_range(1..100000);
    let n_ticks = context.rng.gen_range(1..100);
    // Given a pool
    context.init()
        // When a user has not deposited tokens
        .user("Alice").set_vk("")
            // Then their bonding stays at max
            .during(n_ticks, |_, context| { context.bonding(bonding); })
            // When a user deposits tokens
            .deposits(stake)
            // Then their bonding starts decrementing from the next block
            .during(bonding, |i, context| { context.bonding(bonding - i); })
            // Then their bonding remains at 0
            .during(n_ticks, |_, context| { context.bonding(0); });
}

#[test] fn test_0105_reset () {
    let mut context = Context::named("0106_exit");
    let stake  = context.rng.gen_range(1..100000) * 2;
    let reward = context.rng.gen_range(1..100000);
    let bonding = context.bonding;

    // Given an instance
    // When user deposits
    context.init().fund(reward).later()
        .user("Alice").set_vk("")
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)
            .staked(stake).volume(0).bonding(bonding).earned(0)

        // When the bonding period is not over
        .branch("before_bonding", |mut context|{
            context.after(10)
                .volume(10 * stake)
                .bonding(bonding - 10)
                .earned(reward)

                // And the user withdraws all tokens
                .branch("1", |mut context|{
                    context.withdraws(stake)
                        // Then user's volume and bonding reset
                        .volume(0)
                        .bonding(bonding)
                        // And there are no rewards
                        .earned(0);
                })

                // And the user withdraws some tokens
                .branch("2", |mut context|{
                    context.withdraws(stake/2)
                        // Then user's volume is preserved
                        .volume(10 * stake)
                        .bonding(bonding - 10)
                        .earned(reward)
                    .after(10)
                        // And the volume keeps incrementing
                        .volume(10 * stake + 10 * stake / 2)
                        // And the bonding keeps decrementing
                        .earned(reward)
                        .bonding(bonding - 20)
                    // When user withdraws the rest of the tokens
                    // Then the user's volume and bonding reset
                    .withdraws(stake/2)
                        .staked(0)
                        .volume(0)
                        .earned(0)
                        .bonding(bonding);
                });
        })

        // When the bonding period is over
        .branch("after_bonding", |mut context|{
            context.epoch()
                .earned(reward).bonding(0)

                // And user withdraws all tokens
                .branch("1", |mut context|{
                    // Then rewards are automatically transferred
                    context.withdraws_claims(stake, reward)
                        .staked(0)
                        .volume(0)
                        .earned(0)
                        .bonding(bonding)
                        .distributed(reward);
                })

                // And user withdraws some tokens
                .branch("2", |mut context|{
                    context.withdraws(stake/2)
                        // Then user's volume is preserved
                        .volume(bonding as u128 * stake)
                        .bonding(0)
                        .earned(reward)
                    .after(10)
                        // And the volume keeps incrementing
                        .volume(bonding as u128 * stake + 10 * stake / 2)
                        // And the bonding keeps decrementing
                        .bonding(0)
                        .earned(reward)
                    // When user withdraws the rest of the tokens
                    // Then the user's volume and bonding reset
                    .withdraws_claims(stake/2, reward)
                        .staked(0)
                        .volume(0)
                        .bonding(bonding)
                        .earned(0)
                        .distributed(reward);
                });
        })

        // When user claims after bonding
        .branch("after_claim", |mut context|{
            // Then rewards are transferred
            // And  user's volume and bonding reset
            // And  user's stake remains the same
            context.epoch()
                .staked(stake).bonding(0).volume((stake * bonding as u128).into())
                .earned(reward)
            .tick()
                .staked(stake).volume((stake * (bonding + 1) as u128).into()).bonding(0).earned(reward)
                .claims(reward).distributed(reward)
                .staked(stake).volume(0).bonding(bonding).earned(0)
            .epoch().fund(reward)
                .earned(reward).bonding(0);
        });
}

#[test] fn test_0106_deposit_withdraw_one () {
    let mut context = Context::new();
    let stake = context.rng.gen_range(1..100000)*2;
    // Given an instance
    Context::named("0106_deposit_withdraw_one").admin().init()
        //  When user first deposits
        //  Then user's age and volume start incrementing
        .later().user("Alice")
                 .set_vk("").staked(    0).volume(      0)
            .deposits(stake).staked(stake).volume(      0)
                     .tick().staked(stake).volume(stake*1)
                     .tick().staked(stake).volume(stake*2)
                     .tick().staked(stake).volume(stake*3)
            //  When user withdraws half of the tokens
            //  Then user's age keeps incrementing
            //   And user's volume keeps incrementing at a halved rate
            .withdraws(stake/2).staked(stake/2).volume(stake*3)
                        .tick().staked(stake/2).volume(stake*3+stake/2)
                        .tick().staked(stake/2).volume(stake*3+stake)
            //  When user withdraws other half of tokens
            //  Then user's age and volume reset
            .withdraws(stake/2).staked(0).volume(0)
                        .tick().staked(0).volume(0)
                        .tick().staked(0).volume(0)
            //  When user deposits tokens again later
            //  Then user's age and volume start incrementing again
                   .deposits(1).staked(1).volume(0)
                        .tick().staked(1).volume(1)
                        .tick().staked(1).volume(2);
}

#[test] fn test_0107_claim () {
    let mut context = Context::new();
    let bonding = context.bonding;
    let stake  = context.rng.gen_range(1..100000);
    let reward = context.rng.gen_range(1..100000);

    // Given an instance
    Context::named("0107_claim").init().fund(reward)
        .user("Alice").set_vk("")
        //  When users tries to claim reward before providing liquidity
        //  Then they get an error
        .later()
            .must_wait(bonding)
        //  When users provide liquidity
        //   And they wait for rewards to accumulate
        .later()
            .must_wait(bonding)
            .deposits(stake)
            .must_wait(bonding)
        .tick()
            .must_wait(bonding-1)
        .tick()
            .must_wait(bonding-2)
        // ...
        .after(bonding-3)
            .must_wait(1)
        //   And a provider claims rewards
        //  Then that provider receives reward tokens
        .tick()
            .claims(reward)
        //  When a provider claims rewards twice within a period
        //  Then rewards are sent only the first time
        .fund(reward)
        .tick()
            .must_wait(bonding-1)
        .tick()
            .must_wait(bonding-2)
        .tick()
            .must_wait(bonding-3)
        // ...
        //  When a provider claims their rewards less often
        //  Then they receive equivalent rewards as long as the liquidity deposited hasn't changed
        .fund(reward)
            .epoch()
            .epoch()
            .claims(reward*2)
            .must_wait(bonding);
}

#[test] fn test_0108_sequential () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(1..100000);
    let reward = context.rng.gen_range(1..100000);
    Context::named("0100_two_sequential")
        .admin().init()
        .later().fund(reward)
            .user("Alice").set_vk("")
                .later().deposits(stake)
                .epoch().withdraws_claims(stake, reward)
        .later().fund(reward)
            .user("Bob").set_vk("")
                .later().deposits(stake)
                .epoch().withdraws_claims(stake, reward);
}

#[test] fn test_0109_parallel () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(1..100000)*2;
    let reward = context.rng.gen_range(1..100000)*2;
    // Given an instance:
    Context::named("0100_two_parallel")
        .admin().init().fund(reward)
        //  When alice and bob first deposit lp tokens simultaneously,
        //  Then their ages and earnings start incrementing simultaneously;
        .later()
            .user("Alice").set_vk("").deposits(stake).earned(0)
            .user("Bob").set_vk("").deposits(stake).earned(0)
        //  When alice and bob withdraw lp tokens simultaneously,
        //  Then their ages and earnings keep changing simultaneously;
        .later()
            .user("Alice").set_vk("").withdraws(stake/2).earned(reward/2)
            .user("Bob").set_vk("").withdraws(stake/2).earned(reward/2)
        //  When alice and bob's ages reach the configured threshold,
        //  Then each is eligible to claim half of the available rewards
        //   And their rewards are proportionate to their stakes.
        .epoch()
            .user("Alice").earned(reward/2).withdraws_claims(stake/2, reward/2)
            .user("Bob").earned(reward/2).withdraws_claims(stake/2, reward/2);
}

/// Given an instance where rewards are given in the same token that is staked
///
///  When a user deposits tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///
///  When a user withdraws tokens after claiming
///  Then they get the original amount
#[test] fn test_0113_single_sided () {
    let mut context = Context::named("0113_single_sided");
    context.lp_token = context.reward_token.clone();
    let stake  = context.rng.gen_range(1..100000);
    let reward = context.rng.gen_range(1..100000);
    context
        .admin()
            .init().fund(reward)
        .user("Alice")
            .later().deposits(stake)
            .epoch().claims(reward)
            .later().withdraws(stake);
}

#[test] fn test_0114_close () {
    let mut context  = Context::named("0114_close");
    let reward: u128 = context.rng.gen_range(1..100000);
    let stake1: u128 = context.rng.gen_range(1..100000);
    let stake2: u128 = context.rng.gen_range(1..100000);
    let return_funds = context.lp_token.transfer(
        &HumanAddr::from("Alice"), (stake1+stake2).into()
    ).unwrap();
    // Given a pool with some activity
    // When someone unauthorized tries to close the pool
    // Then they can't
    context
        .admin().init().fund(reward)
        .later().badman().cannot_close_pool()
        .later().user("Alice").deposits(stake1)
        .later().badman().cannot_close_pool()
        .later().user("Alice").deposits(stake2)
        .later().badman().cannot_close_pool()

        // When the admin closes the pool
        .later().admin().closes_pool();

    // Then the pool is closed
    let (ref when, ref why) = context.closed.clone().unwrap();

    // And every user transaction returns all LP tokens to the user
    context.later().user("Alice")
        .branch("then_lock", |mut context|{
            context.test_handle(
                RewardsHandle::Lock { amount: 100u128.into() },
                HandleResponse::default()
                    .msg(return_funds.clone()).unwrap()
                    .log("close_time",   &format!("{}", when)).unwrap()
                    .log("close_reason", why));
        })
        .branch("then_retrieve", |mut context|{
            context.test_handle(
                RewardsHandle::Retrieve { amount: 100u128.into() },
                HandleResponse::default()
                    .msg(return_funds.clone()).unwrap()
                    .log("close_time",   &format!("{}", when)).unwrap()
                    .log("close_reason", why));
        })
        .branch("then_claim", |mut context|{
            context.test_handle(
                RewardsHandle::Claim {},
                HandleResponse::default()
                    .msg(return_funds.clone()).unwrap()
                    .log("close_time",   &format!("{}", when)).unwrap()
                    .log("close_reason", why));
        });
}
