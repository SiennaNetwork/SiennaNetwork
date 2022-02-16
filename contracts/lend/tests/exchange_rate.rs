use std::str::FromStr;

use lend_shared::{
    fadroma::{cosmwasm_std::Uint128, ensemble::MockEnv, one_token, Decimal256, Permit, Uint256},
    interfaces::{market, overseer},
};

use crate::{setup::Lend};

const BOB: &str = "Bob";
const ALICE: &str = "Alice";

#[test]
fn initial_exchange_rate() {
    let mut lend = Lend::default();

    let initial_rate = Decimal256::from_str("5.5").unwrap();

    let token = lend.new_underlying_token("TKN", 9).unwrap();
    let market = lend
        .whitelist_market(token, Decimal256::one(), Some(initial_rate))
        .unwrap();

    let rate = lend.exchange_rate(market.contract.address, None);

    assert_eq!(initial_rate, rate);
}

#[test]
fn initial_exchange_rate_mint() {
    let mut lend = Lend::default();

    let initial_rate = Decimal256::from_uint256(Uint256::from(5_000_000_000u64)).unwrap();

    let token = lend.new_underlying_token("TKN", 18).unwrap();
    let market = lend
        .whitelist_market(token.clone(), Decimal256::one(), Some(initial_rate))
        .unwrap();

    let deposit_amount = Uint128(50 * one_token(18));

    lend.prefund_and_deposit(BOB, deposit_amount, market.contract.address.clone());

    let state: market::State = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            market::QueryMsg::State { block: None },
        )
        .unwrap();

    let expected = Uint256::from(10000000000u64);

    assert_eq!(state.total_supply, expected);

    let info = lend.account_info(BOB, market.contract.address);
    assert_eq!(info.sl_token_balance, expected);
}

#[test]
fn accrue_interest_basic() {
    let mut lend = Lend::default();
    let borrow_amount = Uint256::from(10 * one_token(18));

    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();

    // whitelist markets
    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(75),
            Some(Decimal256::percent(100)),
        )
        .unwrap();
    let market_2 = lend
        .whitelist_market(
            underlying_2.clone(),
            Decimal256::percent(75),
            Some(Decimal256::percent(100)),
        )
        .unwrap();

    // set underlying prices
    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market_2.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();

    // fund markets
    lend.prefund_and_deposit(
        BOB,
        Uint128(50 * one_token(18)),
        market_1.contract.address.clone(),
    );

    lend.prefund_and_deposit(
        ALICE,
        Uint128(20 * one_token(18)),
        market_1.contract.address.clone(),
    );

    // enter markets
    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![
                    market_1.contract.address.clone(),
                    market_2.contract.address.clone(),
                ],
            },
            MockEnv::new(BOB, lend.overseer.clone()),
        )
        .unwrap();
    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![
                    market_1.contract.address.clone(),
                    market_2.contract.address.clone(),
                ],
            },
            MockEnv::new(ALICE, lend.overseer.clone()),
        )
        .unwrap();
    // borrow
    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(ALICE, market_1.contract.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market_1.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    ALICE,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market_1.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();
    assert_eq!(res.borrow_balance, borrow_amount);

    // Should have accrued some interest over a period of 100_000 blocks
    let res: Uint128 = lend
        .ensemble
        .query(
            market_1.contract.address.clone(),
            market::QueryMsg::BalanceUnderlying {
                method: Permit::<market::MarketPermissions>::new(
                    BOB,
                    vec![market::MarketPermissions::Balance],
                    vec![market_1.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: Some(112345),
            },
        )
        .unwrap();

    assert!(res > Uint128(50 * one_token(18)))
}
