use crate::test::*;

#[test] fn test_claim_one () {

    // Given an instance
    Context::named("algo_020_claim_one")
        .admin()
            .at(1)
                .init()
        .fund(100)
        .user("Alice")
            //  When users tries to claim reward before providing liquidity
            //  Then they get an error
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
