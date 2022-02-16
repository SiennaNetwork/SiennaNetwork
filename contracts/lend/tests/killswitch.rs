use crate::setup::{Lend, ADMIN};
use lend_shared::{
    fadroma::{
        cosmwasm_std::Uint128,
        ensemble::MockEnv,
        killswitch::{ContractStatusLevel, HandleMsg::SetStatus},
        one_token, Decimal256, Uint256,
    },
    interfaces::{market, overseer},
};

const BOB: &str = "Bob";

#[test]
fn killswitch() {
    let mut lend = Lend::default();

    let underlying = lend.new_underlying_token("TKN", 18).unwrap();

    let market = lend
        .whitelist_market(
            underlying.clone(),
            Decimal256::percent(75),
            Some(Decimal256::percent(50)),
        )
        .unwrap();

    lend.prefund_and_deposit(
        BOB,
        Uint128(2 * one_token(18)),
        market.contract.address.clone(),
    );

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(BOB, lend.overseer.clone()),
        )
        .unwrap();

    // Pause the contract
    lend.ensemble
        .execute(
            &market::HandleMsg::Killswitch(SetStatus {
                level: ContractStatusLevel::Paused,
                reason: "good reason".into(),
                new_address: None,
            }),
            MockEnv::new(ADMIN, market.contract.clone()),
        )
        .unwrap();

    // Borrow should not happen
    let res = lend.ensemble.execute(
        &market::HandleMsg::Borrow {
            amount: Uint256::from(1u128),
        },
        MockEnv::new(BOB, market.contract.clone()),
    );
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("This contract has been paused."));

    // Admin handlers are still allowed
    lend.ensemble
        .execute(
            &market::HandleMsg::UpdateConfig {
                interest_model: None,
                reserve_factor: Some(Decimal256::one()),
                borrow_cap: None,
            },
            MockEnv::new(ADMIN, market.contract.clone()),
        )
        .unwrap();

    // Make the contract operational again
    lend.ensemble
        .execute(
            &market::HandleMsg::Killswitch(SetStatus {
                level: ContractStatusLevel::Operational,
                reason: "good reason".into(),
                new_address: None,
            }),
            MockEnv::new(ADMIN, market.contract.clone()),
        )
        .unwrap();

    // Borrow is successful
    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1u128),
            },
            MockEnv::new(BOB, market.contract.clone()),
        )
        .unwrap();
}
