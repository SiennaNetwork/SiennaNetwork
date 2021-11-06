use crate::test::*;

#[test] fn test_parallel () {

    let mut context = Context::named("algo_022_parallel");
    let stake  = context.rng.gen_range(0..100000);
    let reward = context.rng.gen_range(0..100000)*2;
    Context::named("algo_022_parallel")
        .admin().init().fund(reward)
        .later()
            .user("Alice").set_vk("").deposits(stake)
            .user("Bob").set_vk("").deposits(stake)
        .epoch()
            .user("Alice").withdraws_claims(stake, reward/2)
            .user("Bob").withdraws_claims(stake, reward/2);

}
