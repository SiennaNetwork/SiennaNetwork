use lend_shared::{
    fadroma::{decimal::one_token, ensemble::MockEnv, Decimal256, Permit, Uint128, Uint256},
    interfaces::market,
};

use crate::setup::Lend;

const BOB: &str = "Bob";
const ALICE: &str = "Alice";

#[test]
fn transfer_no_funds() {
    let mut lend = Lend::default();
    let exchange_rate = Decimal256::one();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();

    let market = lend
        .whitelist_market(underlying_1.clone(), Decimal256::one(), Some(exchange_rate), None)
        .unwrap();

    // prefund only
    lend.prefund_user(BOB, Uint128(10), underlying_1);

    // fails if not enough fund, because there's no deposit yet
    let res = lend.ensemble.execute(
        &market::HandleMsg::Transfer {
            recipient: ALICE.into(),
            amount: Uint256::from(10 * one_token(6)),
        },
        MockEnv::new(BOB, market.contract.clone()),
    );
    assert!(res.unwrap_err().to_string().contains("insufficient funds"));
}

#[test]
fn transfer_basic() {
    let mut lend = Lend::default();
    let exchange_rate = Decimal256::one();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();

    let market = lend
        .whitelist_market(underlying_1.clone(), Decimal256::one(), Some(exchange_rate), None)
        .unwrap();

    lend.prefund_and_deposit(
        BOB,
        Uint128(50 * one_token(6)),
        market.contract.address.clone(),
    );
    // simple transfer 1:1
    lend.ensemble
        .execute(
            &market::HandleMsg::Transfer {
                recipient: ALICE.into(),
                amount: Uint256::from(10 * one_token(6)),
            },
            MockEnv::new(BOB, market.contract.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BOB,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(40 * one_token(6)));

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    ALICE,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(10 * one_token(6)));
}
