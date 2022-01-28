use std::str::FromStr;

use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::MockEnv,
        snip20_impl::msg::{HandleMsg as Snip20HandleMsg, QueryMsg as Snip20QueryMsg},
        to_binary, Binary, Decimal256, Permit, Uint128, Uint256,
    },
    interfaces::{market, overseer},
};

use crate::setup::Lend;

const BOB: &str = "Bob";
const ALICE: &str = "Alice";

#[test]
fn liquidate_basic() {
    let mut lend = Lend::default();
    let borrow_amount = Uint256::from(1 * one_token(18));

    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();

    // whitelist markets
    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(75),
            Some(Decimal256::percent(50)),
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2.clone(),
            Decimal256::percent(75),
            Some(Decimal256::percent(50)),
        )
        .unwrap();

    // set underlying prices
    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market_2.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();

    // prefund markets
    lend.prefund_and_deposit(
        BOB,
        Uint128(21 * one_token(17)),
        market_1.contract.address.clone(),
    );
    lend.prefund_user(ALICE, Uint128(5 * one_token(18)), underlying_2.clone());
    // deposit
    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market_2.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(2 * one_token(18)),
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(ALICE, underlying_2.clone()),
        )
        .unwrap();

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

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(BOB, market_2.contract.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market_2.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BOB,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market_2.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();
    assert_eq!(res.sl_token_balance, Uint256::from(0u128));
    assert_eq!(res.borrow_balance, Uint256::from(1 * one_token(18)));

    // crash the price of first token and liquidate
    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(5 * one_token(17)))
        .unwrap();

    let liquidity = lend
        .get_liquidity(
            BOB,
            Some(market_2.contract.address.clone()),
            Uint256::zero(),
            borrow_amount.into(),
            None,
        )
        .unwrap();

    // negative liquidity
    assert_ne!(liquidity.shortfall, Uint256::zero());

    let id: Binary = lend
        .ensemble
        .query(
            market_2.contract.address.clone(),
            market::QueryMsg::Id {
                method: Permit::new(
                    BOB,
                    vec![market::MarketPermissions::Id],
                    vec![market_2.contract.address.clone()],
                    "id",
                )
                .into(),
            },
        )
        .unwrap();

    // lend.ensemble
    //     .execute(
    //         &Snip20HandleMsg::Send {
    //             recipient: market_2.contract.address.clone(),
    //             recipient_code_hash: None,
    //             amount: Uint128(1 * one_token(18)),
    //             msg: Some(
    //                 to_binary(&market::ReceiverCallbackMsg::Liquidate {
    //                     borrower: id,
    //                     collateral: underlying_1.address.clone(),
    //                 })
    //                 .unwrap(),
    //             ),
    //             memo: None,
    //             padding: None,
    //         },
    //         MockEnv::new(ALICE, underlying_2.clone()),
    //     )
    //     .unwrap();
}
