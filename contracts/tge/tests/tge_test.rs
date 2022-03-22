use fadroma::{ensemble::MockEnv, ContractLink, HumanAddr, Uint128};
use sienna_mgmt::ConfigResponse;
use sienna_rpt::LinearMap;
use sienna_schedule::Account;

use crate::setup::{ADMIN, TGE};

#[test]
fn rpt_init() {
    let tge = TGE::default();

    let _: ConfigResponse = tge
        .ensemble
        .query(tge.mgmt.address, sienna_mgmt::QueryMsg::Config {})
        .unwrap();
}

#[test]
fn admin_update_distribution() {
    let mut tge = TGE::default();
    let token1 = HumanAddr::from("secret1TOKEN1");
    let token2 = HumanAddr::from("secret1TOKEN2");
    let updated_distribution = LinearMap(vec![
        (token1.clone(), Uint128::from(1000u128)),
        (token2.clone(), Uint128::from(1500u128)),
    ]);

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::SetDistribution {
                distribution: updated_distribution.clone(),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash,
                },
            ),
        )
        .unwrap();

    let response: sienna_rpt::ConfigResponse = tge
        .ensemble
        .query(tge.rpt.address, &sienna_rpt::QueryMsg::Configuration {})
        .unwrap();

    assert_eq!(response.distribution, updated_distribution);
}
#[test]
fn stranger_cant_update_distribution() {
    let mut tge = TGE::default();
    let token1 = HumanAddr::from("secret1TOKEN1");
    let token2 = HumanAddr::from("secret1TOKEN2");
    let updated_distribution = LinearMap(vec![
        (token1.clone(), Uint128::from(1000u128)),
        (token2.clone(), Uint128::from(1500u128)),
    ]);

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::SetDistribution {
                distribution: updated_distribution.clone(),
            },
            MockEnv::new(
                "stranger",
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash,
                },
            ),
        )
        .unwrap_err();
}
#[test]
fn only_valid_distribution() {
    let mut tge = TGE::default();
    let token1 = HumanAddr::from("secret1TOKEN1");
    let token2 = HumanAddr::from("secret1TOKEN2");
    //must add up to 2500
    let updated_distribution = LinearMap(vec![
        (token1.clone(), Uint128::from(1200u128)),
        (token2.clone(), Uint128::from(1500u128)),
    ]);

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::SetDistribution {
                distribution: updated_distribution.clone(),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash,
                },
            ),
        )
        .unwrap_err();
}
#[test]
fn should_distribute() {
    let mut tge = TGE::default();
    let initial_balance = tge.query_balance(ADMIN);

    tge.add_account(
        "TEST".into(),
        Account {
            name: "RPT_SPLIT".into(),
            address: tge.rpt.address.clone(),
            amount: Uint128(25),
            cliff: Uint128(0),
            start_at: 0,
            interval: 180,
            duration: 0,
        },
    )
    .unwrap();

    tge.ensemble
        .execute(
            &sienna_mgmt::HandleMsg::Launch {},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "MGMT_CONTRACT".into(),
                    code_hash: tge.mgmt.code_hash.clone(),
                },
            ),
        )
        .unwrap();

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::Vest {},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash.clone(),
                },
            )
            .time(1978897420),
        )
        .unwrap();

    let updated_balance = tge.query_balance(ADMIN);

    assert_ne!(initial_balance, updated_balance);
}
