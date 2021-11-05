use crate::test::*;

#[test] fn test_deposit_withdraw_parallel () {
    // Given an instance:
    Context::named("algo_012_deposit_withdraw_parallel")
        .admin().at(1).init()
        //  When alice and bob first deposit lp tokens simultaneously,
        //  Then their ages and earnings start incrementing simultaneously;
        .at(2).user("Alice").deposits(100)
              .user("Bob").deposits(100);
        //  When alice and bob withdraw lp tokens simultaneously,
        //  Then their ages and earnings keep changing simultaneously;
        //
        //  When alice and bob's ages reach the configured threshold,
        //  Then each is eligible to claim half of the available rewards
        //   And their rewards are proportionate to their stakes.
}
