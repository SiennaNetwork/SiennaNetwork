use crate::test::*;

#[test] fn test_0101_accumulation () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;

    // Given an instance
    Context::named("test_0101_accumulation").admin().init().user("Alice").set_vk("")

        // When user deposits
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)

            // Then user's stake increments
            .staked(stake).volume(0).bonding(bonding).earned(0)

        // And user's liquidity starts incrementing
        // And user's bonding starts decrementing
        .tick()
            .staked(stake).volume(stake*1).bonding(bonding - 1).earned(reward)
        .tick()
            .staked(stake).volume(stake*2).bonding(bonding - 2).earned(reward)
        .tick()
            .staked(stake).volume(stake*3).bonding(bonding - 3).earned(reward);
}

#[test] fn test_0102_bonding () {
    let mut context = Context::named("test_0102_bonding");
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
        .at(t_lock)
            // When a user deposits tokens
            .deposits(stake)
            // Then they need to keep them deposited for a fixed amount of time before they can claim
            .must_wait(bonding)
        .tick().must_wait(bonding - 1)
        .tick().must_wait(bonding - 2)
        .tick().must_wait(bonding - 3)
        .at(t_lock + bonding - 3).must_wait(3)
        .tick().must_wait(2)
        .tick().must_wait(1)
        .tick()
            // When a user claims rewards
            .claims(reward)
            // Then they need to wait a fixed amount of time before they can claim again
            .must_wait(bonding)
        .tick().must_wait(bonding - 1)
        .tick().must_wait(bonding - 2)
        .fund(reward)
        .at(t_lock + 2*bonding - 3).must_wait(3)
        .tick().must_wait(2)
        .tick().must_wait(1)
        .tick().claims(reward);

}

#[test] fn test_0103_exit_before_bonding () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;

    // Given an instance
    Context::named("test_0103_exit_before_bonding").admin().init().user("Alice").set_vk("")

        // When user deposits
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)

        //  And withdraws all before bonding is over
        // Then there are no rewards
        //  And user's liquidity and bonding reset
        .later()
            .earned(reward)
            .withdraws(stake)
            .staked(0).volume(0).bonding(bonding).earned(0);
}

#[test] fn test_0104_exit_after_bonding () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;

    // Given an instance
    Context::named("test_0104_exit_after_bonding").admin().init().user("Alice").set_vk("")

        // When user deposits
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)

        // And  withdraws after bonding
        // Then rewards are automatically transferred
        // And  user's liquidity and bonding reset
        .later()
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)
            .staked(stake).volume(0).bonding(bonding).earned(0)
        .epoch()
            .earned(reward).bonding(0)
            .withdraws_claims(stake, reward).distributed(reward)
            .earned(0).bonding(bonding);
}

#[test] fn test_0105_claim_without_exit () {
    let mut context = Context::new();
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;

    // Given an instance
    Context::named("test_0105_claim_without_exit").admin().init().user("Alice").set_vk("")

        // When user deposits
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)

        // And  claims after bonding
        // Then rewards are transferred
        // And  user's liquidity and bonding reset
        // And  user's stake remains the same
        .later().fund(reward)
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)
            .staked(stake).volume(0).bonding(bonding).earned(0)
        .epoch()
            .staked(stake).bonding(0).volume((stake * bonding as u128).into())
            .earned(reward)
        .tick()
            .staked(stake).volume((stake * (bonding + 1) as u128).into()).bonding(0).earned(reward)
            .claims(reward).distributed(reward * 2)
            .staked(stake).volume(0).bonding(bonding).earned(0)
        .epoch().fund(reward)
            .earned(reward).bonding(0);
}

#[test] fn test_0106_deposit_withdraw_one () {

    // Given an instance
    Context::named("test_0106_deposit_withdraw_one")
        .at(1).admin().init()

        //  When user first deposits
        //  Then user's age and volume start incrementing
        .later().user("Alice")
               .set_vk("").staked(  0).volume(  0)
            .deposits(100).staked(100).volume(  0)
                   .tick().staked(100).volume(100)
                   .tick().staked(100).volume(200)
                   .tick().staked(100).volume(300)

            //  When user withdraws half of the tokens
            //  Then user's age keeps incrementing
            //   And user's volume keeps incrementing at a halved rate
            .withdraws(50).staked( 50).volume(300)
                   .tick().staked( 50).volume(350)
                   .tick().staked( 50).volume(400)

            //  When user withdraws other half of tokens
            //  Then user's age and volume reset
            .withdraws(50).staked(  0).volume(  0)
                   .tick().staked(  0).volume(  0)
                   .tick().staked(  0).volume(  0)

            //  When user deposits tokens again later
            //  Then user's age and volume start incrementing again
              .deposits(1).staked(  1).volume(  0)
                   .tick().staked(  1).volume(  1)
                   .tick().staked(  1).volume(  2);
}

#[test] fn test_0107_claim_one () {

    // Given an instance
    Context::named("test_0107_claim_one")
        .admin().init().fund(100)
        //  When users tries to claim reward before providing liquidity
        //  Then they get an error
        .user("Alice")
            .tick()
                .must_wait(86400)
            //  When users provide liquidity
            //   And they wait for rewards to accumulate
            .tick()
                .must_wait(86400)
                .deposits(100)
                .must_wait(86400)
            .tick()
                .must_wait(86399)
            .tick()
                .must_wait(86398)
            // ...
            .at(86402)
                .must_wait(1)
            //   And a provider claims rewards
            //  Then that provider receives reward tokens
            .tick()
                .claims(100)
        .fund(100)
            //  When a provider claims rewards twice within a period
            //  Then rewards are sent only the first time
            .tick()
                .must_wait(86399)
            .tick()
                .must_wait(86398)
            .tick()
                .must_wait(86397)
            // ...
            //  When a provider claims their rewards less often
            //  Then they receive equivalent rewards as long as the liquidity deposited hasn't changed
        .fund(100)
            .at(3*86400+3)
                .claims(200)
                .must_wait(86400);
}

#[test] fn test_0108_sequential () {
    let mut context = Context::named("test_0108_sequential");
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    context
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
    let mut context = Context::named("test_0109_parallel");
    let stake  = context.rng.gen_range(0..100000)*2;
    let reward = context.rng.gen_range(0..100000)*2;
    // Given an instance:
    Context::named("test_0109_parallel")
        .admin().init().fund(reward)
        //  When alice and bob first deposit lp tokens simultaneously,
        //  Then their ages and earnings start incrementing simultaneously;
        .later()
            .user("Alice").set_vk("").deposits(stake)
            .user("Bob").set_vk("").deposits(stake)
        //  When alice and bob withdraw lp tokens simultaneously,
        //  Then their ages and earnings keep changing simultaneously;
        .later()
            .user("Alice").set_vk("").withdraws(stake/2)
            .user("Bob").set_vk("").withdraws(stake/2)
        //  When alice and bob's ages reach the configured threshold,
        //  Then each is eligible to claim half of the available rewards
        //   And their rewards are proportionate to their stakes.
        .epoch()
            .user("Alice").withdraws_claims(stake/2, reward/2)
            .user("Bob").withdraws_claims(stake/2, reward/2);
}

#[test] fn test_0110_reset_after_claim () {
    let mut context = Context::new();
    let stake   = context.rng.gen_range(0..100000)*2;
    let reward  = context.rng.gen_range(0..100000);
    // Given a pool and a user
    // When the user claims reward
    // Then the user can withdraw stake
    // Then the user is reset
    Context::named("test_0110_reset_after_claim")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch().claims(reward).volume(0).withdraws(stake);
}

#[test] fn test_0111_reset_after_full_withdraw () {
    let mut context = Context::new();
    let stake   = context.rng.gen_range(0..100000)*2;
    let reward  = context.rng.gen_range(0..100000);
    // Given a pool and a user
    // When the user withdraws full stake
    // Then the user auto claims
    //  And the user is reset
    Context::named("test_0111_reset_after_full_withdraw")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch().withdraws_claims(stake, reward).volume(0);
}

#[test] fn test_0112_reset_only_full_withdraw () {
    let mut context = Context::new();
    let stake   = context.rng.gen_range(0..100000)*2;
    let reward  = context.rng.gen_range(0..100000);
    // Given a pool and a user
    // When the user withdraws partial stake
    // Then the user doesn't auto claim
    // When the user withdraws the rest of the stake
    // Then the user auto claims
    //  And the user is reset
    Context::named("test_0112_reset_only_full_withdraw")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch().withdraws(stake/2)
            .later().withdraws_claims(stake/2, reward).volume(0);
}

/// Given an instance where rewards are given in the same token that is staked
///
///  When a user deposits tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///
///  When a user withdraws tokens after claiming
///  Then they get the original amount
#[test] fn test_0113_single_sided () {
    let mut context = Context::named("test_0113_single_sided");
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
