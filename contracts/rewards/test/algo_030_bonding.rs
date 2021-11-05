use crate::test::*;

#[test] fn test_bonding () {
    let mut context = Context::named("algo_030_bonding");
    let bonding = context.rng.gen_range(0..100000);
    let t_lock  = context.rng.gen_range(0..100000);
    let reward  = context.rng.gen_range(0..100000);
    let stake   = context.rng.gen_range(0..100000);

    // Given a pool
    context
        .admin().init().configure(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            ratio:        None,
            bonding:      Some(bonding),
        })
        .fund(reward)
        .user("Alice")
        .at(t_lock)
            // When a user deposits tokens
            .deposits(stake)
            // Then they need to keep them deposited for a fixed amount of time before they can claim
            .must_wait(bonding)
        .tick().must_wait(bonding - 1)
        .tick().must_wait(bonding - 2)
        .tick().must_wait(bonding - 3)
        .at(t_lock + bonding - 3).must_wait(3)
        .tick().must_wait(2)
        .tick().must_wait(1)
        .tick()
            // When a user claims rewards
            .claims(reward)
            // Then they need to wait a fixed amount of time before they can claim again
            .must_wait(bonding)
        .tick().must_wait(bonding - 1)
        .tick().must_wait(bonding - 2)
        .fund(reward)
        .at(t_lock + 2*bonding - 3).must_wait(3)
        .tick().must_wait(2)
        .tick().must_wait(1)
        .tick().claims(reward);

}
