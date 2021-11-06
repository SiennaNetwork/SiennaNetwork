use crate::test::*;

#[test] fn test_deposit_withdraw_one () {

    // Given an instance
    Context::named("algo_011_deposit_withdraw_one")
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
            //  When another user deposits tokens
            //  Then the first user's reward share starts to diminish
            //
            //  When user tries to withdraw too much
            //  Then they can't
            //
            //  When a stranger tries to withdraw
            //  Then they can't

}
