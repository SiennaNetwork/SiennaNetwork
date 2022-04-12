use crate::setup::Lend;
use lend_shared::{
    core::{utilization_rate, Pagination},
    fadroma::{
        cosmwasm_std::{to_binary, StdError, Uint128, HumanAddr},
        ensemble::MockEnv,
        snip20_impl::msg as snip20,
        permit::Permit,
        Decimal256, Uint256, one_token
    },
    interfaces::{market, overseer},
};

const BOB: &str = "Bob";
const ALICE: &str = "Alice";
const CHESTER: &str = "Chester";

#[test]
fn borrow() {
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("SSCRT", 6).unwrap();
    let underlying_2 = lend.new_underlying_token("SIENNA", 18).unwrap();

    let market_one = lend
        .whitelist_market(underlying_1, Decimal256::percent(90), None, None)
        .unwrap()
        .contract;

    let market_two = lend
        .whitelist_market(underlying_2, Decimal256::percent(90), None, None)
        .unwrap()
        .contract;

    let prefund_amount = Uint128(500 * one_token(6));
    let chester_prefund_amount = Uint128(100 * one_token(18));
    let borrow_amount = Uint128(100 * one_token(6));

    lend.prefund_and_deposit(BOB, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(ALICE, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(CHESTER, chester_prefund_amount, market_two.address.clone());

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market_one.address.clone(), market_two.address.clone()],
            },
            MockEnv::new(CHESTER, lend.overseer.clone()),
        )
        .unwrap();

    // Assuming 1:1 price conversion:
    // With LTV ratio of 90%, if Chester wants to borrow a 100 tokens, he'd have to deposit
    // 10% more as collateral - 100 + 10% = 110

    // conv_factor = 0.9 * 1 * 1 = 0.9
    // Chester's liquidity = 100 * 0.9 = 90

    let liquidity = lend
        .get_liquidity(
            CHESTER,
            Some(market_one.address.clone()),
            Uint256::zero(),
            borrow_amount.into(),
            None,
        )
        .unwrap();

    println!("{:#?}", liquidity);

    // assert_eq!(liquidity.liquidity, Uint256::zero());
    // assert_eq!(liquidity.shortfall, Uint256::from(10));

    let err = lend
        .ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(CHESTER, market_one.clone()),
        )
        .unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err("Insufficient liquidity. Shortfall: 10000000000000000000")
    );

    // 12 instead of 10 because of rounding during exchange rate divison
    lend.prefund_and_deposit(CHESTER, Uint128(12 * one_token(18)), market_two.address.clone());

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(CHESTER, market_one.clone()),
        )
        .unwrap();

    let liquidity = lend
        .get_liquidity(CHESTER, None, Uint256::zero(), Uint256::zero(), None)
        .unwrap();

    // 0.8 liquidity
    assert_eq!(liquidity.liquidity, Uint256::from(800000000000000000u128));
    assert_eq!(liquidity.shortfall, Uint256::zero());

    let state: market::State = lend
        .ensemble
        .query(
            market_one.address.clone(),
            market::QueryMsg::State { block: None },
        )
        .unwrap();

    assert_eq!(state.total_borrows, 100000000.into());
    assert_eq!(state.total_supply, 1000000000.into());

    let utilization_rate = utilization_rate(
        Decimal256::from_uint256(state.underlying_balance).unwrap(),
        Decimal256::from_uint256(state.total_borrows).unwrap(),
        Decimal256::from_uint256(state.total_reserves).unwrap(),
    )
    .unwrap();

    assert_eq!(utilization_rate, Decimal256::percent(10));

    let info: market::BorrowersResponse = lend
        .ensemble
        .query(
            market_one.address.clone(),
            market::QueryMsg::Borrowers {
                pagination: Pagination {
                    start: 0,
                    limit: 10
                },
                block: state.accrual_block,
            },
        )
        .unwrap();

    assert_eq!(info.total, 1);
    assert_eq!(info.entries.len(), 1);

    let chester = info.entries.first().unwrap();

    assert_eq!(chester.liquidity.liquidity, Uint256::from(800000000000000000u128));
    assert_eq!(chester.liquidity.shortfall, Uint256::zero());

    assert_eq!(chester.markets.len(), 2);
    assert_eq!(chester.markets[0].contract, market_one);
    assert_eq!(chester.markets[1].contract, market_two);

    assert_eq!(chester.principal_balance, borrow_amount.into());
    assert_eq!(chester.actual_balance, borrow_amount.into());

    let id = lend.id(CHESTER, market_one.address);
    assert_eq!(chester.id, id);
}

#[test]
fn cannot_increase_collateral_value_by_entering_the_same_market_multiple_times() {
    let mut lend = Lend::default();

    let token_1 = lend.new_underlying_token("ATOM", 9).unwrap();
    let token_2 = lend.new_underlying_token("SSCRT", 6).unwrap();

    let market_1 = lend.whitelist_market(
        token_1,
        Decimal256::percent(50),
        None,
        None
    ).unwrap();

    let market_2 = lend.whitelist_market(
        token_2,
        Decimal256::percent(50),
        None,
        None
    ).unwrap();

    let prefund_alice_atom = Uint128(1000 * one_token(9));
    let prefund_alice_scrt = Uint128(10000 * one_token(6));
    let prefund_bob_atom = Uint128(1000 * one_token(9));
    let borrow_bob_scrt = Uint256::from(2000 * one_token(6));

    lend.prefund_and_deposit(ALICE, prefund_alice_atom, market_1.contract.address.clone());
    lend.prefund_and_deposit(BOB, prefund_bob_atom, market_1.contract.address.clone());
    lend.prefund_and_deposit(ALICE, prefund_alice_scrt, market_2.contract.address.clone());

    lend.ensemble.execute(
        &overseer::HandleMsg::Enter {
            markets: vec![market_1.contract.address.clone()],
        },
        MockEnv::new(ALICE, lend.overseer.clone())
    ).unwrap();

    lend.ensemble.execute(
        &overseer::HandleMsg::Enter {
            markets: vec![
                market_1.contract.address.clone(),
                market_1.contract.address.clone(),
                market_2.contract.address.clone(),
                market_1.contract.address.clone(),
                market_1.contract.address.clone()
            ],
        },
        MockEnv::new(BOB, lend.overseer.clone())
    ).unwrap();

    lend.ensemble.execute(
        &overseer::HandleMsg::Enter {
            markets: vec![
                market_1.contract.address.clone(),
                market_2.contract.address.clone()
            ],
        },
        MockEnv::new(BOB, lend.overseer.clone())
    ).unwrap();

    let entered_markets: Vec<overseer::Market<HumanAddr>> = lend.ensemble.query(
        lend.overseer.address.clone(),
        overseer::QueryMsg::EnteredMarkets {
            method: Permit::new(
                BOB,
                vec![overseer::OverseerPermissions::AccountInfo],
                vec![lend.overseer.address.clone()],
                "entered_markets",
            )
            .into()
        }
    ).unwrap();

    assert_eq!(entered_markets.len(), 2);

    let err = lend.ensemble.execute(
        &market::HandleMsg::Borrow {
            amount: borrow_bob_scrt,
        },
        MockEnv::new(BOB, market_2.contract.clone()),
    )
    .unwrap_err();

    let liquidity = lend.get_liquidity(
        BOB,
        None,
        Uint256::zero(),
        Uint256::zero(),
        None
    ).unwrap();

    assert_eq!(liquidity.liquidity, Uint256::from(500_000_000_000_000_000_000u128));
    assert_eq!(liquidity.shortfall, Uint256::zero());

    assert_eq!(err, StdError::generic_err(format!("Insufficient liquidity. Shortfall: {}", 1500_000_000_000_000_000_000u128)));
}

#[test]
fn self_repay() {
    do_repay(true)
}

#[test]
fn repay() {
    do_repay(false)
}

fn do_repay(self_repay: bool) {
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("SSCRT", 6).unwrap();
    let underlying_2 = lend.new_underlying_token("SIENNA", 18).unwrap();

    let market_one = lend
        .whitelist_market(underlying_1.clone(), Decimal256::percent(90), None, None)
        .unwrap()
        .contract;

    let market_two = lend
        .whitelist_market(underlying_2, Decimal256::percent(90), None, None)
        .unwrap()
        .contract;

    let prefund_amount = Uint128(500 * one_token(6));
    let borrow_amount = Uint128(100 * one_token(6));
    let collateral_amount = Uint128(112 * one_token(18));

    lend.prefund_and_deposit(BOB, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(ALICE, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(CHESTER, collateral_amount, market_two.address.clone());

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market_one.address.clone(), market_two.address.clone()],
            },
            MockEnv::new(CHESTER, lend.overseer.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(CHESTER, market_one.clone()),
        )
        .unwrap();

    let market_one_state = lend.state(market_one.address.clone(), None);
    let market_two_state = lend.state(market_two.address.clone(), None);

    assert_eq!(market_one_state.total_borrows, borrow_amount.into());
    assert_eq!(market_one_state.total_supply, (prefund_amount.0 * 2).into());
    assert_eq!(
        market_one_state.underlying_balance,
        (prefund_amount.0 * 2 - borrow_amount.0).into()
    );

    assert_eq!(market_two_state.total_borrows, Uint256::zero());
    assert_eq!(market_two_state.total_supply, collateral_amount.into());
    assert_eq!(
        market_two_state.underlying_balance,
        collateral_amount.into()
    );

    let (repayer, id) = if self_repay {
        (CHESTER, None)
    } else {
        (ALICE, Some(lend.id(CHESTER, market_one.address.clone())))
    };

    lend.prefund_user(repayer, borrow_amount, underlying_1.clone());

    lend.ensemble
        .execute(
            &snip20::HandleMsg::Send {
                recipient: market_one.address.clone(),
                recipient_code_hash: None,
                amount: borrow_amount,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Repay { borrower: id }).unwrap()),
                memo: None,
                padding: None,
            },
            MockEnv::new(repayer, underlying_1),
        )
        .unwrap();

    let market_one_state = lend.state(market_one.address.clone(), None);

    assert_eq!(market_one_state.total_borrows, Uint256::zero());
    assert_eq!(market_one_state.total_supply, (prefund_amount.0 * 2).into());
    assert_eq!(
        market_one_state.underlying_balance,
        (prefund_amount.0 * 2).into()
    );

    let info: market::BorrowersResponse = lend
        .ensemble
        .query(
            market_one.address.clone(),
            market::QueryMsg::Borrowers {
                pagination: Pagination {
                    start: 0,
                    limit: 10
                },
                block: market_one_state.accrual_block,
            },
        )
        .unwrap();

    // Borrows repaid in full are removed from storage.
    assert_eq!(info.total, 0);
    assert_eq!(info.entries.len(), 0);
}
