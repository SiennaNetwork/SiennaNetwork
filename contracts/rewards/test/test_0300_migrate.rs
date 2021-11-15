use crate::test::*;

/// Given two instances
///
///  When a user tries to initiate a migration
///  Then they fail
///
///  When the admin calls SetMigrationStatus
///   And passes the address of the new contract
///  Then the old contract is in migration mode
///
///  When a user calls ImportState on the new contract
///  Then the new contract fetches data from the old one
#[test] fn test_migration () {

    let bonding = Some(86400);

    let reward_vk = Some("reward_vk".to_string());

    let reward_token = Some(ContractLink {
        address:   HumanAddr::from("SIENNA"),
        code_hash: "SIENNA_hash".into()
    });

    let lp_token = Some(ContractLink {
        address:   HumanAddr::from("LP_TOKEN"),
        code_hash: "LP_hash".into()
    });

    let timekeeper = Some(HumanAddr::from("QUASIMODO"));

    let config = RewardsConfig { bonding, lp_token, reward_token, reward_vk, timekeeper };

    let mut v1 = MockExtern {
        storage: ClonableMemoryStorage::default(),
        api:     MockApi::new(20),
        querier: RewardsMockQuerier::new()
    };

    let env1 = env(&HumanAddr::from("ADMIN"), 0);

    Contract::init(&mut v1, env1.clone(), Init { admin: None, config: config.clone() }).unwrap();

    let mut v2 = MockExtern {
        storage: ClonableMemoryStorage::default(),
        api:     MockApi::new(20),
        querier: RewardsMockQuerier::new()
    };

    let env2 = env(&HumanAddr::from("ADMIN"), 0);

    Contract::init(&mut v2, env2.clone(), Init { admin: None, config: config.clone() }).unwrap();

    Contract::handle(&mut v1, env1.clone(), Handle::Migration(MigrationHandle::EnableMigration(ContractLink {
        address:   HumanAddr::from("V2"),
        code_hash: "".into()
    }))).unwrap();

    Contract::handle(&mut v2, env2.clone(), Handle::Migration(MigrationHandle::ImportState(ContractLink {
        address:   HumanAddr::from("V1"),
        code_hash: "".into()
    }))).unwrap();

}
