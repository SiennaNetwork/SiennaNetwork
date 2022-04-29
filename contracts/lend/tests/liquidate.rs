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
    core::Pagination
};

use crate::setup::{Lend, LendConfig, ADMIN};

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

    let height = 12360;
    lend.ensemble.block().height = height;
    lend.ensemble.block().freeze();

    let liquidity = lend
        .get_liquidity(
            BOB,
            None,
            Uint256::zero(),
            Uint256::zero(),
            Some(height),
        )
        .unwrap();

    assert!(liquidity.liquidity < topup_amount.into());
    assert_eq!(liquidity.shortfall, Uint256::zero());

    let borrowers: market::BorrowersResponse = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: height,
            pagination: Pagination {
                start: 0,
                limit: 10
            }
        }
    ).unwrap();

    assert_eq!(borrowers.total, 1);
    assert_eq!(borrowers.entries.len(), 1);

    lend.prefund_user(ALICE, Uint128(100 * one_token(18)), underlying_2.clone());
    let err = lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market_2.contract.address.clone(),
            recipient_code_hash: None,
            amount: borrow_amount,
            msg: Some(
                to_binary(&market::ReceiverCallbackMsg::Liquidate {
                    borrower: borrowers.entries[0].id.clone(),
                    collateral: market_1.contract.address.clone(),
                })
                .unwrap(),
            ),
            memo: None,
            padding: None,
        },
        MockEnv::new(ALICE, underlying_2.clone())
    )
    .unwrap_err();

    assert_eq!(err, StdError::generic_err("Borrower cannot be liquidated."));

    let err = lend.simulate_liquidation(
        market_2.contract.address.clone(),
        borrowers.entries[0].id.clone(),
        market_1.contract.address.clone(),
        borrow_amount.into()
    ).unwrap_err();

    assert_eq!(err, StdError::generic_err("Borrower cannot be liquidated."));

    lend.ensemble.block().height += 1000000;
    let height = lend.ensemble.block().height;

    let liquidity = lend
        .get_liquidity(
            BOB,
            None,
            Uint256::zero(),
            Uint256::zero(),
            Some(height),
        )
        .unwrap();

    assert_eq!(liquidity.liquidity, Uint256::zero());
    assert_ne!(liquidity.shortfall, Uint256::zero());

    let borrowers: market::BorrowersResponse = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: height,
            pagination: Pagination {
                start: 0,
                limit: 10
            }
        }
    ).unwrap();

    assert_eq!(borrowers.total, 1);
    assert_eq!(borrowers.entries.len(), 1);

    let interest = (borrowers.entries[0].actual_balance - borrowers.entries[0].principal_balance).unwrap();

    let liquidate_result = lend.simulate_liquidation(
        market_2.contract.address.clone(),
        borrowers.entries[0].id.clone(),
        market_1.contract.address.clone(),
        borrow_amount.into()
    ).unwrap();

    assert_eq!(liquidate_result.seize_amount, borrow_amount.into());
    assert_eq!(liquidate_result.shortfall, Uint256::zero());

    lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market_2.contract.address.clone(),
            recipient_code_hash: None,
            amount: borrow_amount,
            msg: Some(
                to_binary(&market::ReceiverCallbackMsg::Liquidate {
                    borrower: borrowers.entries[0].id.clone(),
                    collateral: market_1.contract.address.clone(),
                })
                .unwrap(),
            ),
            memo: None,
            padding: None,
        },
        MockEnv::new(ALICE, underlying_2.clone())
    )
    .unwrap();

    let borrowers: market::BorrowersResponse = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: height,
            pagination: Pagination {
                start: 0,
                limit: 10
            }
        }
    ).unwrap();

    assert_eq!(borrowers.total, 1);
    assert_eq!(borrowers.entries[0].principal_balance, interest);
}

#[test]
fn close_factor() {
    let mut lend = Lend::new(
        LendConfig::new()
            .close_factor(Decimal256::percent(50))
    );

    let height = lend.ensemble.block().height;
    lend.ensemble.block().freeze();

    let borrow_amount = Uint256::from(10 * one_token(18));

    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();

    // whitelist markets
    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::one(),
            None,
            None,
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2.clone(),
            Decimal256::one(),
            None,
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
        borrow_amount.low_u128().into(),
        market_1.contract.address.clone(),
    );

    let alice_deposit = (borrow_amount * Uint256::from(2)).unwrap();
    lend.prefund_user(ALICE, alice_deposit.low_u128().into(), underlying_2.clone());
    // deposit
    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market_2.contract.address.clone(),
                recipient_code_hash: None,
                amount: borrow_amount.low_u128().into(),
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
    assert_eq!(res.borrow_balance, borrow_amount);

    // crash the price of first token and liquidate
    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(5 * one_token(17))).unwrap();

    let id = lend.id(BOB, market_2.contract.address.clone());

    let err = lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market_2.contract.address.clone(),
            recipient_code_hash: None,
            amount: borrow_amount.low_u128().into(),
            msg: Some(
                to_binary(&market::ReceiverCallbackMsg::Liquidate {
                    borrower: id.clone(),
                    collateral: market_1.contract.address.clone(),
                })
                .unwrap(),
            ),
            memo: None,
            padding: None,
        },
        MockEnv::new(ALICE, underlying_2.clone()),
    )
    .unwrap_err();

    assert_eq!(err, StdError::generic_err("Repay amount is too high. Amount: 10000000000000000000, Max: 5000000000000000000"));

    let liquidate_amount: Uint128 = (borrow_amount.low_u128() / 2).into();
    lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market_2.contract.address.clone(),
            recipient_code_hash: None,
            amount: liquidate_amount,
            msg: Some(
                to_binary(&market::ReceiverCallbackMsg::Liquidate {
                    borrower: id.clone(),
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

    let state = lend.state(market_1.contract.address.clone(), None);

    let alice_acc = lend.account_info(ALICE, market_1.contract.address.clone());
    assert_eq!(alice_acc.borrow_balance, Uint256::zero());

    // We crashed the collateral price, so seize amount has doubled.
    let seized_amount = Uint256::from(liquidate_amount.0 * 2);
    assert_eq!(alice_acc.sl_token_balance, (seized_amount - state.total_reserves).unwrap());

    let borrowers: market::BorrowersResponse = lend.ensemble.query(
        market_2.contract.address.clone(),
        market::QueryMsg::Borrowers {
            block: height,
            pagination: Pagination {
                start: 0,
                limit: 10
            }
        }
    ).unwrap();

    assert_eq!(borrowers.total, 1);
    assert_eq!(borrowers.entries.len(), 1);
    assert_eq!(borrowers.entries[0].actual_balance, liquidate_amount.into());

    let info = lend.account_info(BOB, market_1.contract.address);
    assert_eq!(info.borrow_balance, Uint256::zero());
    assert_eq!(info.sl_token_balance, Uint256::zero());

    let info = lend.account_info(BOB, market_2.contract.address);
    assert_eq!(info.borrow_balance, liquidate_amount.into());
    assert_eq!(info.sl_token_balance, Uint256::zero());
}

#[test]
#[should_panic(expected = "Premium rate cannot be less than 1.")]
fn cannot_set_premium_rate_less_than_one() {
    Lend::new(LendConfig::new().premium(Decimal256::percent(99)));
}

#[test]
fn premium_rate() {
    let mut lend = Lend::default();

    let borrow_amount = Uint256::from(10 * one_token(18));

    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();

    // whitelist markets
    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::one(),
            None,
            None,
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2.clone(),
            Decimal256::one(),
            None,
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
        borrow_amount.low_u128().into(),
        market_1.contract.address.clone(),
    );

    let alice_deposit = (borrow_amount * Uint256::from(2)).unwrap();
    lend.prefund_user(ALICE, alice_deposit.low_u128().into(), underlying_2.clone());
    // deposit
    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market_2.contract.address.clone(),
                recipient_code_hash: None,
                amount: borrow_amount.low_u128().into(),
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
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(BOB, market_2.contract.clone()),
        )
        .unwrap();

    let liquidate_amount: Uint256 = (borrow_amount.low_u128() / 2).into();

    let expected: Uint256 = (borrow_amount.low_u128() / 2).into();
    let seize_amount: Uint256 = lend.ensemble.query(
        lend.overseer.address.clone(),
        overseer::QueryMsg::SeizeAmount {
            borrowed: market_2.contract.address.clone(),
            collateral: market_1.contract.address.clone(),
            repay_amount: liquidate_amount
        }
    ).unwrap();

    assert_eq!(seize_amount, expected);

    lend.ensemble.execute(
        &overseer::HandleMsg::ChangeConfig {
            premium_rate: Some(Decimal256::percent(150)),
            close_factor: None,
            oracle: None
        },
        MockEnv::new(ADMIN, lend.overseer.clone())
    ).unwrap();

    let expected = liquidate_amount.decimal_mul(Decimal256::percent(150)).unwrap();
    let seize_amount: Uint256 = lend.ensemble.query(
        lend.overseer.address.clone(),
        overseer::QueryMsg::SeizeAmount {
            borrowed: market_2.contract.address.clone(),
            collateral: market_1.contract.address.clone(),
            repay_amount: liquidate_amount
        }
    ).unwrap();

    assert_eq!(seize_amount, expected);

    lend.ensemble.execute(
        &overseer::HandleMsg::ChangeConfig {
            premium_rate: Some(Decimal256::percent(200)),
            close_factor: None,
            oracle: None
        },
        MockEnv::new(ADMIN, lend.overseer.clone())
    ).unwrap();

    let expected = borrow_amount;
    let seize_amount: Uint256 = lend.ensemble.query(
        lend.overseer.address,
        overseer::QueryMsg::SeizeAmount {
            borrowed: market_2.contract.address,
            collateral: market_1.contract.address,
            repay_amount: liquidate_amount
        }
    ).unwrap();

    assert_eq!(seize_amount, expected);
}
