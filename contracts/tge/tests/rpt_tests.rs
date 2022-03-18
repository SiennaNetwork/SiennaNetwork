use fadroma::{ensemble::MockEnv, ContractLink, HumanAddr, Uint128};
use sienna_mgmt::ConfigResponse;
use sienna_rpt::LinearMap;

use crate::setup::{ADMIN, TGE};

#[test]
fn test_rpt_init() {
    let tge = TGE::default();

    let _: ConfigResponse = tge
        .ensemble
        .query(tge.mgmt.address, sienna_mgmt::QueryMsg::Config {})
        .unwrap();
}

#[test]
fn test_admin_update_config() {
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
fn test_stranger_cant_update_config() {
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
