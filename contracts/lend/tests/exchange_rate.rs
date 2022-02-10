use std::str::FromStr;

use lend_shared::{
    fadroma::{
        cosmwasm_std::Uint128,
        Decimal256, Uint256, one_token
    },
    interfaces::market
};

use crate::setup::Lend;

const BOB: &str = "Bob";

#[test]
fn initial_exchange_rate() {
    let mut lend = Lend::default();

    let initial_rate = Decimal256::from_str("5.5").unwrap();

    let token = lend.new_underlying_token("TKN", 9).unwrap();
    let market = lend.whitelist_market(
        token,
        Decimal256::one(),
        Some(initial_rate)
    ).unwrap();

    let rate: Decimal256 = lend.ensemble.query(
        market.contract.address,
        market::QueryMsg::ExchangeRate {
            block: None
        }
    ).unwrap();

    assert_eq!(initial_rate, rate);
}

#[test]
fn initial_exchange_rate_mint() {
    let mut lend = Lend::default();

    let initial_rate = Decimal256::from_uint256(Uint256::from(5_000_000_000u64)).unwrap();

    let token = lend.new_underlying_token("TKN", 18).unwrap();
    let market = lend.whitelist_market(
        token.clone(),
        Decimal256::one(),
        Some(initial_rate)
    ).unwrap();

    let deposit_amount = Uint128(50 * one_token(18));

    lend.prefund_and_deposit(BOB, deposit_amount, market.contract.address.clone());

    let state: market::State = lend.ensemble.query(
        market.contract.address.clone(),
        market::QueryMsg::State { block: None }
    ).unwrap();

    let expected = Uint256::from(10000000000u64);

    assert_eq!(state.total_supply, expected);

    let info = lend.account_info(BOB, market.contract.address);
    assert_eq!(info.sl_token_balance, expected);
}
