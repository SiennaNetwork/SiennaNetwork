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
