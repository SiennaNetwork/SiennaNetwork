use crate::test::*;

#[test] fn test_basic () {
    // Given an instance
    let mut context = Context::named("algo_010_basic");
    
    context
        .admin()
            .init()
            
        .user("Alice")
            .set_vk("")
            // When user first deposits
            .later().fund(100)
                .staked(  0).volume(  0).bonding(86400).claimable(0)
                .deposits(100)
            // Then user's stake increments
                .staked(100).volume(  0).bonding(86400).claimable(0)
            // And user's liquidity starts incrementing
            // And user's bonding starts decrementing
            .tick()
                .staked(100).volume(100).bonding(86399).claimable(0)
            .tick()
                .staked(100).volume(200).bonding(86398).claimable(0)
            .tick()
                .staked(100).volume(300).bonding(86397).claimable(0)
            // When user withdraws all before bonding is over
            // Then there are no rewards
            //  And user's liquidity and bonding reset
            .later()
                .claimable(0)
                .withdraws(100)
                .staked(  0).volume(  0).bonding(86400).claimable(0)
            // When user withdraws all after bonding
            // Then rewards are automatically transferred
            //  And user's liquidity and bonding reset
            .later().fund(100)
                .staked(  0).volume(  0).bonding(86400).claimable(0)
                .deposits(100)
                .staked(100).volume(  0).bonding(86400).claimable(0)
            .epoch()
                .claimable(100).bonding(0)
            .tick()
            // When user claims after bonding
            // Then rewards are transferred
            //  And user's liquidity and bonding reset
            //  And user's stake remains the same
            .later().fund(100)
                .staked(  0).volume(  0).bonding(86400).claimable(0)
                .deposits(100)
                .staked(100).volume(  0).bonding(86400).claimable(0)
            .epoch()
                .claimable(100).bonding(0)
            .tick()
                .claimable(100).bonding(0)
                .claims(100)
                .staked(100).volume(100 * 86400).bonding(86400)
            .epoch().fund(100)
                .claimable(100).bonding(0);
            // When the same user does the same thing later
            // Then the same thing happens
}
