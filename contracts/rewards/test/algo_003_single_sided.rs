use crate::test::*;

/// Given an instance where rewards are given in the same token that is staked
///
///  When a user deposits tokens and claims rewards
///  Then rewards are calculated on the basis of the reward balance only
///
///  When a user withdraws tokens after claiming
///  Then they get the original amount
#[test] fn test_single_sided () {

    let mut context = Context::named("algo_003_single_sided");
    context.lp_token = context.reward_token.clone();
    context
        .admin()
            .at(1).init().fund(100u128)
        .user("Alice")
            .at(2)    .deposits(100u128)
            .at(86402).claims(100u128)
            .at(86403).withdraws(100u128);

}
