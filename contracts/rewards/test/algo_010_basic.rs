use crate::test::*;

#[test] fn test_basic () {
    // Given an instance
    let mut context = Context::named("algo_010_basic");
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    
    context
        .admin()
            .init()
            
        .user("Alice")
            .set_vk("")
            // When user first deposits
            .later().fund(reward)
                .staked(    0).volume(  0).bonding(86400).earned(0)
                .deposits(stake)
            // Then user's stake increments
                .staked(stake).volume(  0).bonding(86400).earned(0)
            // And user's liquidity starts incrementing
            // And user's bonding starts decrementing
            .tick().staked(stake).volume(stake*1).bonding(86399).earned(0)
            .tick().staked(stake).volume(stake*2).bonding(86398).earned(0)
            .tick().staked(stake).volume(stake*3).bonding(86397).earned(0)
            // When user withdraws all before bonding is over
            // Then there are no rewards
            //  And user's liquidity and bonding reset
            .later()
                .earned(0)
                .withdraws(stake)
                .staked(    0).volume(  0).bonding(86400).earned(0)
            // When user withdraws all after bonding
            // Then rewards are automatically transferred
            //  And user's liquidity and bonding reset
            .later().fund(reward)
                .staked(    0).volume(  0).bonding(86400).earned(0)
                .deposits(stake)
                .staked(stake).volume(  0).bonding(86400).earned(0)
            .epoch()
                .earned(reward).bonding(0)
            .tick()
            // When user claims after bonding
            // Then rewards are transferred
            //  And user's liquidity and bonding reset
            //  And user's stake remains the same
            .later().fund(reward)
                .staked(    0).volume(  0).bonding(86400).earned(0)
                .deposits(stake)
                .staked(stake).volume(  0).bonding(86400).earned(0)
            .epoch()
                .earned(reward).bonding(0)
            .tick()
                .earned(reward).bonding(0)
                .claims(reward)
                .staked(stake).volume(stake * 86400).bonding(86400)
            .epoch().fund(reward)
                .earned(reward).bonding(0);
            // When the same user does the same thing later
            // Then the same thing happens
}
