use crate::test::{*, Context};

#[test] fn test_0301_migrate () {

    let mut contract1 = Context::new("0301_migrate_v1");

    let mut contract2 = Context::new("0301_migrate_v2");

    let stake = contract1.rng.gen_range(1..100000)*2;

    contract1.init()
        .user("Alice").set_vk("")
            .staked(0).total_staked(0)
            .deposits(stake)
            .staked(stake).total_staked(stake)
        .admin()
            .enable_migration_to(&contract2.link);

    contract2.init()
        .user("Alice") // no vk, should migrate that too
            .migrate_from(&mut contract1, stake, 0u128)
            .staked(stake).total_staked(stake);

}
