use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::MockEnv,
        snip20_impl::msg::HandleMsg as Snip20HandleMsg,
        permit::Permit,
        cosmwasm_std::{
            Uint128, StdError, to_binary
        },
        Decimal256, Uint256
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
            None,
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2.clone(),
            Decimal256::percent(75),
            Some(Decimal256::percent(50)),
            None,
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
        Uint128(2 * one_token(18)),
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

    let id = lend.id(BOB, market_2.contract.address.clone());

    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market_2.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1 * one_token(18)),
                msg: Some(
                    to_binary(&market::ReceiverCallbackMsg::Liquidate {
                        borrower: id,
                        collateral: market_1.contract.address.clone(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new(ALICE, underlying_2.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market_1.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BOB,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market_1.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();
    assert_eq!(res.sl_token_balance, Uint256::zero());

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
    assert_eq!(
        res.sl_token_balance,
        Uint256::from(3_888_000_000_000_000_000u128)
    );
}

#[test]
fn borrower_accrues_interest_and_goes_underwater() {
    let mut lend = Lend::default();
    let borrow_amount = Uint128(20 * one_token(18));

    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();

    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(100),
            None,
            None,
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2.clone(),
            Decimal256::percent(100),
            None,
            None,
        )
        .unwrap();

    // set underlying prices
    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market_2.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();

    let topup_amount = Uint128(2 * one_token(16));
    lend.prefund_and_deposit(
        BOB,
        (borrow_amount + topup_amount).into(),
        market_1.contract.address.clone(),
    );

    let alice_deposit = Uint128(100 * one_token(18));

    lend.prefund_and_deposit(
        ALICE,
        alice_deposit,
        market_1.contract.address.clone(),
    );

    lend.prefund_and_deposit(
        ALICE,
        borrow_amount,
        market_2.contract.address.clone(),
    );

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
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(BOB, market_2.contract.clone()),
        )
        .unwrap();

    let res = lend.account_info(BOB, market_2.contract.address.clone());
    assert_eq!(res.sl_token_balance, Uint256::zero());
    assert_eq!(res.borrow_balance, borrow_amount.into());

    let liquidity = lend
        .get_liquidity(
            BOB,
            None,
            Uint256::zero(),
            Uint256::zero(),
            None,
        )
        .unwrap();
    
    assert_eq!(liquidity.liquidity, topup_amount.into());
    assert_eq!(liquidity.shortfall, Uint256::zero());

    let mut current_block = 12360;

    let liquidity = lend
        .get_liquidity(
            BOB,
            None,
            Uint256::zero(),
            Uint256::zero(),
            Some(current_block),
        )
        .unwrap();

    assert!(liquidity.liquidity < topup_amount.into());
    assert_eq!(liquidity.shortfall, Uint256::zero());

    let borrowers: Vec<market::Borrower> = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: current_block,
            start_after: None,
            limit: None
        }
    ).unwrap();

    assert_eq!(borrowers.len(), 1);

    lend.prefund_user(ALICE, Uint128(100 * one_token(18)), underlying_2.clone());
    let err = lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market_2.contract.address.clone(),
            recipient_code_hash: None,
            amount: borrow_amount,
            msg: Some(
                to_binary(&market::ReceiverCallbackMsg::Liquidate {
                    borrower: borrowers[0].id.clone(),
                    collateral: market_1.contract.address.clone(),
                })
                .unwrap(),
            ),
            memo: None,
            padding: None,
        },
        MockEnv::new(ALICE, underlying_2.clone()).height(current_block)
    )
    .unwrap_err();

    assert_eq!(err, StdError::generic_err("Borrower cannot be liquidated."));

    current_block += 1000000;

    let liquidity = lend
        .get_liquidity(
            BOB,
            None,
            Uint256::zero(),
            Uint256::zero(),
            Some(current_block),
        )
        .unwrap();

    assert_eq!(liquidity.liquidity, Uint256::zero());
    assert_ne!(liquidity.shortfall, Uint256::zero());

    let borrowers: Vec<market::Borrower> = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: current_block,
            start_after: None,
            limit: None
        }
    ).unwrap();

    assert_eq!(borrowers.len(), 1);

    let interest = (borrowers[0].actual_balance - borrowers[0].principal_balance).unwrap();

    lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market_2.contract.address.clone(),
            recipient_code_hash: None,
            amount: borrow_amount,
            msg: Some(
                to_binary(&market::ReceiverCallbackMsg::Liquidate {
                    borrower: borrowers[0].id.clone(),
                    collateral: market_1.contract.address.clone(),
                })
                .unwrap(),
            ),
            memo: None,
            padding: None,
        },
        MockEnv::new(ALICE, underlying_2.clone()).height(current_block)
    )
    .unwrap();

    let borrowers: Vec<market::Borrower> = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: current_block,
            start_after: None,
            limit: None
        }
    ).unwrap();

    assert_eq!(borrowers[0].principal_balance, interest);
}
