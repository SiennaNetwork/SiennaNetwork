use crate::test::*;

#[test] fn test_basic () {
    // Given an instance
    Context::new()
        .at(1)
            .admin()
            .init()
            .user("Alice").set_vk("")
            .staked(0u128).volume(0u128).bonding(86400)
        // When user first deposits
        .later()
            .staked(0u128).volume(0u128).bonding(86400)
            .deposits(100u128)
            // Then user's stake increments
            .staked(100u128).volume(0u128).bonding(86400)
        // And user's liquidity starts incrementing
        // And user's bonding starts decrementing
        .tick().staked(100u128).volume(100u128).bonding(86399)
        .tick().staked(100u128).volume(200u128).bonding(86398)
        .tick().staked(100u128).volume(300u128).bonding(86397);
    // When user withdraws all before bonding is over
    // Then user's liquidity and bonding reset
    //  And there are no rewards
    // 
    // When user withdraws all after bonding
    // Then rewards are automatically transferred
    //  And user's liquidity and bonding reset
    //  And user's stake is 0
    // 
    // When user claims after bonding
    // Then rewards are transferred
    //  And user's liquidity and bonding reset
    //  And user's stake remains the same
    // 
    // When the same user does the same thing later
    // Then the same thing happens
}
