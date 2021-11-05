use crate::test::*;

#[test] fn test_deposit_withdraw_one () {

    // Given an instance
    Context::named("algo_011_deposit_withdraw_one")
        .at(1).admin().init()
        //  When user first deposits
        //  Then user's age and volume start incrementing
        .later().user("Alice").set_vk("")
            .staked(0u128).volume(0u128)
            .deposits(100u128)
            .staked(100u128).volume(0u128)
        .tick()
            .staked(100u128).volume(100u128)
        .tick()
            .staked(100u128).volume(200u128)
            //  When user withdraws half of the tokens
            //  Then user's age keeps incrementing
            //   And user's volume keeps incrementing at a halved rate
            .withdraws(50u128)
            .staked( 50u128).volume(200u128)
       .tick()
            .staked( 50u128).volume(250u128)
       .tick()
            .staked( 50u128).volume(300u128)
            //  When user withdraws other half of tokens
            //  Then user's age and volume stop incrementing
            .withdraws(50u128)
            .staked(  0u128).volume(300u128)
       .tick()
            .staked(  0u128).volume(300u128)
       .tick()
            .staked(  0u128).volume(300u128)
            //  When user deposits tokens again later
            //  Then user's age and volume start incrementing again
            .deposits(1u128)
            .staked(  1u128).volume(300u128)
       .tick()
            .staked(  1u128).volume(301u128)
       .tick()
            .staked(  1u128).volume(302);
            //  When another user deposits tokens
            //  Then the first user's volume share starts to diminish
            //
            //  When user tries to withdraw too much
            //  Then they can't
            //
            //  When a stranger tries to withdraw
            //  Then they can't

}
