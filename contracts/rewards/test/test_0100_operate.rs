use crate::test::{*, Context};

#[test] fn test_0101_accumulation () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;

    // Given an instance
    Context::named("0100_stake").admin().init().user("Alice").set_vk("")

        // When user deposits
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)

            // Then user's stake increments
            .staked(stake).volume(0).bonding(bonding).earned(0)

        // And user's liquidity starts incrementing
        // And user's bonding starts decrementing
        .tick().staked(stake).volume(stake*1).bonding(bonding - 1).earned(reward)
        .tick().staked(stake).volume(stake*2).bonding(bonding - 2).earned(reward)
        .tick().staked(stake).volume(stake*3).bonding(bonding - 3).earned(reward);
}

#[test] fn test_0102_bonding () {
    let mut context = Context::named("0100_bonding");
    let bonding = context.rng.gen_range(0..100000);
    let t_lock  = context.rng.gen_range(0..100000);
    let reward  = context.rng.gen_range(0..100000);
    let stake   = context.rng.gen_range(0..100000);

    // Given a pool
    context
        .admin().init().configures(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            bonding:      Some(bonding),
        })
        .fund(reward)
        .user("Alice")
            // When a user deposits tokens
            // Then they need to keep them deposited for a fixed amount of time before they can claim
            .at(t_lock).deposits(stake).must_wait(bonding)
            .tick().must_wait(bonding - 1)
            .tick().must_wait(bonding - 2)
            .tick().must_wait(bonding - 3)
            .at(t_lock + bonding - 3).must_wait(3)
            .tick().must_wait(2)
            .tick().must_wait(1)
            // When a user claims rewards
            // Then they need to wait a fixed amount of time before they can claim again
            .tick().claims(reward).must_wait(bonding)
            .tick().must_wait(bonding - 1)
            .tick().must_wait(bonding - 2)
            .fund(reward)
            .at(t_lock + 2*bonding - 3).must_wait(3)
            .tick().must_wait(2)
            .tick().must_wait(1)
            .tick().claims(reward);

}

#[test] fn test_0103_exit () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;

    // Given an instance
    // When  user deposits
    Context::named("0100_exit")
        .admin().init().user("Alice").set_vk("")
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)
            .staked(stake).volume(0).bonding(bonding).earned(0)

        .branch("before_bonding", |mut context|{
            // When user withdraws all before bonding is over
            // Then there are no rewards
            // And  user's liquidity and bonding reset
            context.later()
                .earned(reward)
                .withdraws(stake)
                .staked(0).volume(0).earned(0).bonding(bonding);
        })

        .branch("after_bonding", |mut context|{
            // When user withdraws all after bonding
            // Then rewards are automatically transferred
            // And  user's liquidity and bonding reset
            context.epoch()
                .earned(reward).bonding(0)
                .withdraws_claims(stake, reward).distributed(reward)
                .staked(0).volume(0).earned(0).bonding(bonding);
        })

        .branch("after_claim", |mut context|{
            // When user claims after bonding
            // Then rewards are transferred
            // And  user's liquidity and bonding reset
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
    let stake  = context.rng.gen_range(0..100000)*2;
    let reward = context.rng.gen_range(0..100000);
    // Given an instance
    Context::named("0100_one_deposit_withdraw").admin().init()
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

#[test] fn test_0107_claim_one () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);

    // Given an instance
    Context::named("0100_one_claim")
        .admin().init().fund(100)
        //  When users tries to claim reward before providing liquidity
        //  Then they get an error
        .user("Alice")
            .tick().must_wait(86400)
            //  When users provide liquidity
            //   And they wait for rewards to accumulate
            .tick().must_wait(86400).deposits(100).must_wait(86400)
            .tick().must_wait(86399)
            .tick().must_wait(86398)
            // ...
            .at(86402).must_wait(1)
            //   And a provider claims rewards
            //  Then that provider receives reward tokens
            .tick().claims(100)
        .fund(100)
            //  When a provider claims rewards twice within a period
            //  Then rewards are sent only the first time
            .tick().must_wait(86399)
            .tick().must_wait(86398)
            .tick() .must_wait(86397)
            // ...
            //  When a provider claims their rewards less often
            //  Then they receive equivalent rewards as long as the liquidity deposited hasn't changed
        .fund(100)
            .at(3*86400+3).claims(200).must_wait(86400);
}

#[test] fn test_0108_sequential () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
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
    let stake  = context.rng.gen_range(0..100000)*2;
    let reward = context.rng.gen_range(0..100000)*2;
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

#[test] fn test_0110_reset() {
    let mut context = Context::new();
    let stake   = context.rng.gen_range(0..100000)*2;
    let reward  = context.rng.gen_range(0..100000);
    Context::named("0100_reset")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch()
            .branch("after_claim", |mut context| {
                context.claims(reward).volume(0).withdraws(stake);
            })
            .branch("after_full_withdraw", |mut context| {
                context.withdraws_claims(stake, reward).volume(0);
            })
            .branch("only_after_full_withdraw", |mut context| {
                context.withdraws(stake/2).later().withdraws_claims(stake/2, reward).volume(0);
            });
}

/// Given an instance where rewards are given in the same token that is staked
///
///  When a user deposits tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///
///  When a user withdraws tokens after claiming
///  Then they get the original amount
#[test] fn test_0113_single_sided () {
    let mut context = Context::named("0100_single_sided");
    context.lp_token = context.reward_token.clone();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    context
        .admin()
            .init().fund(reward)
        .user("Alice")
            .later().deposits(stake)
            .epoch().claims(reward)
            .later().withdraws(stake);
}

#[test] fn test_0300_close () {
    let mut context  = Context::named("0100_close");
    let reward: u128 = context.rng.gen_range(0..100000);
    let stake1: u128 = context.rng.gen_range(0..100000);
    let stake2: u128 = context.rng.gen_range(0..100000);
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
