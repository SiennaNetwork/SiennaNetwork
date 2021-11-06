use crate::test::*;

#[test] fn test_basic () {
    // Given an instance
    let mut context = Context::named("algo_010_basic");
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    let bonding = 86400;
    context.admin().init().user("Alice").set_vk("")
        // When user first deposits
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
            .staked(stake).volume(stake*3).bonding(bonding - 3).earned(reward)
        // When user withdraws all before bonding is over
        // Then there are no rewards
        //  And user's liquidity and bonding reset
        .later()
            .earned(reward)
            .withdraws(stake)
            .staked(0).volume(0).bonding(bonding).earned(0)
        // When user withdraws all after bonding
        // Then rewards are automatically transferred
        //  And user's liquidity and bonding reset
        .later()
            .staked(0).volume(0).bonding(bonding).earned(0)
            .deposits(stake)
            .staked(stake).volume(0).bonding(bonding).earned(0)
        .epoch()
            .earned(reward).bonding(0)
            .withdraws_claims(stake, reward).distributed(reward)
            .earned(0).bonding(bonding)
        .tick()
        // When user claims after bonding
        // Then rewards are transferred
        //  And user's liquidity and bonding reset
        //  And user's stake remains the same
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
        // When the same user does the same thing later
        // Then the same thing happens
}
