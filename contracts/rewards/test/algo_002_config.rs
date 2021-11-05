use crate::test::*;

#[test] fn test_configure () {

    let Context { ref reward_token, ref reward_vk, .. } = Context::new();

    // Given no instance
    Context::named("algo_002_config")
        //  When the admin inits an instance with an empty configuration
        //  Then the default values are used where applicable
        //   And no viewing key config message is returned
        .at(1).admin().init()
        .later().configure(RewardsConfig {
            lp_token:     None,
            reward_token: Some(reward_token.link.clone()),
            reward_vk:    Some(reward_vk.clone()),
            ratio:        None,
            bonding:     None,
        })
        //  When someone else tries to set the config
        //  Then the config remains unchanged
        .later().badman()
            .cannot_configure()
        //  When the admin sets the config, including a reward token
        //  Then a reward token viewing key config message is returned
        .later().admin().configure(RewardsConfig {
            lp_token:     None,
            reward_token: Some(reward_token.link.clone()),
            reward_vk:    Some(reward_vk.clone()),
            ratio:        None,
            bonding:     None,
        });

}
