use crate::test::*;

#[test] fn test_sequential () {

    let mut context = Context::named("algo_021_sequential");
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000);
    context
        .admin().init()
        .fund(reward)
            .user("Alice").set_vk("")
                .later().deposits(stake)
                .epoch().withdraws_claims(stake, reward)
        .later()
        .fund(reward)
            .user("Bob").set_vk("")
                .later().deposits(stake)
                .epoch().withdraws_claims(stake, reward);

}
