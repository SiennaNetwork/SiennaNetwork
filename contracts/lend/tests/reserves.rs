use lend_shared::{
    fadroma::{
        decimal::one_token, ensemble::MockEnv, snip20_impl::msg as snip20, to_binary, Decimal256,
        Permit, Uint128, Uint256,
    },
    interfaces::{market, overseer},
};

use crate::setup::{Lend, ADMIN};

use std::str::FromStr;

const BOB: &str = "Bob";
const ALICE: &str = "Alice";

#[test]
fn reduce_all_reserves() {
    let mut lend = Lend::default();

    let initial_exchange_rate = Decimal256::percent(100);
    let deposit_amount = Uint128(one_token(18));
    let borrow_amount = Uint128(one_token(18));

    let underlying = lend.new_underlying_token("TKN", 18).unwrap();
    let market = lend
        .whitelist_market(
            underlying.clone(),
            Decimal256::percent(75),
            Some(initial_exchange_rate),
            Some(Decimal256::from_str("0.02").unwrap()),
        )
        .unwrap();

    lend.prefund_and_deposit(BOB, deposit_amount, market.contract.address.clone());
    lend.prefund_user(ALICE, Uint128(3 * one_token(18)), underlying.clone());

    lend.ensemble
        .execute(
            &snip20::HandleMsg::Send {
                recipient: market.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(2 * one_token(18)),
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(ALICE, underlying.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(BOB, lend.overseer.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(ALICE, lend.overseer.clone()),
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

    assert_eq!(res.sl_token_balance, Uint256::from(deposit_amount));
    assert_eq!(res.exchange_rate, initial_exchange_rate);

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(ALICE, market.contract.clone()),
        )
        .unwrap();

    let market_state = lend.state(market.contract.address.clone(), Some(112345));

    assert!(market_state.total_borrows > borrow_amount.into());
    assert_eq!(market_state.total_supply, Uint128(3 * one_token(18)).into());
    assert_eq!(
        market_state.total_reserves,
        Uint256::from(10140188098000u128)
    );

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
                block: Some(112345),
            },
        )
        .unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(deposit_amount));
    assert_eq!(res.exchange_rate, Decimal256(1000165623072267333u64.into()));

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
                block: Some(112345),
            },
        )
        .unwrap();

    lend.ensemble
        .execute(
            &snip20::HandleMsg::Send {
                recipient: market.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(res.borrow_balance.clamp_u128().unwrap()),
                msg: Some(
                    to_binary(&market::ReceiverCallbackMsg::Repay { borrower: None }).unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new(ALICE, underlying).height(112345),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &market::HandleMsg::ReduceReserves {
                amount: Uint128(1_014_018_809_800_0u128),
                to: Some(ADMIN.into()),
            },
            MockEnv::new(ADMIN, market.contract.clone()).height(112345),
        )
        .unwrap();

    let market_state = lend.state(market.contract.address.clone(), Some(112345));

    // reserves should be 0 now
    // borrows should be 0 now
    assert_eq!(market_state.total_reserves, Uint256::zero());
    assert_eq!(market_state.total_borrows, Uint256::zero());
}
