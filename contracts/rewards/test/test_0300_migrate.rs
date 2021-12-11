use crate::test::{*, Context};

#[test] fn test_0301_migrate () {

    let mut contract1 = Context::new("0301_migrate_v1");
    let mut contract2 = Context::new("0301_migrate_v2");

    let stake1 = contract1.rng.gen_range(1..100000)*2;
    let stake2 = contract1.rng.gen_range(1..100000)*2;

    contract1.init()
        .user("Alice").set_vk("")
            .staked(0).total_staked(0)
            .deposits(stake1)
            .staked(stake1).total_staked(stake1)
        .user("Bob").set_vk("")
            .staked(0).total_staked(stake1)
            .deposits(stake2)
            .staked(stake2).total_staked(stake1 + stake2);

    contract2.init()
        .user("Alice").set_vk("") // no vk, should migrate that too
            .cannot_migrate_from(&mut contract1)
            .staked(0).total_staked(0);

    contract1
        .admin()
            .enable_migration_to(&contract2.link);

    contract2
        .admin()
            .enable_migration_from(&contract1.link)
        .user("Alice").set_vk("") // no vk, should migrate that too
            .migrate_from(&mut contract1, stake1, 0u128)
            .staked(stake1).total_staked(stake1)
        .admin()
            .disable_migration_from(&contract1.link)
        .user("Bob").set_vk("") // no vk, should migrate that too
            .cannot_migrate_from(&mut contract1)
            .staked(0).total_staked(stake1)
        .admin()
            .enable_migration_from(&contract1.link)
        .user("Bob") // no vk, should migrate that too
            .migrate_from(&mut contract1, stake2, 0u128)
            .staked(stake2).total_staked(stake1 + stake2);

}
