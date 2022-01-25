use std::str::FromStr;

use lend_shared::{
    fadroma::{
        ensemble::MockEnv, snip20_impl::msg::HandleMsg as Snip20HandleMsg, to_binary, Binary,
        Decimal256, HumanAddr, Permit, StdError, Uint128, Uint256,
    },
    interfaces::{market, overseer::*},
};

use crate::setup::Lend;

#[test]
fn whitelist() {
    let mut lend = Lend::new();

    // can only be called by admin
    let res = lend.ensemble.execute(
        &HandleMsg::Whitelist {
            config: MarketInitConfig {
                prng_seed: Binary::from(b"seed_for_base_market"),
                underlying_asset: lend.underlying_token_one.clone(),
                ltv_ratio: Decimal256::zero(),
                config: market::Config {
                    initial_exchange_rate: Decimal256::one(),
                    reserve_factor: Decimal256::one(),
                    seize_factor: Decimal256::one(),
                },
                interest_model_contract: lend.interest_model.clone(),
            },
        },
        MockEnv::new("fake", lend.overseer.clone()),
    );

    assert_eq!(StdError::unauthorized(), res.unwrap_err());

    lend.whitelist_market(lend.underlying_token_one.clone(), Decimal256::percent(90))
        .unwrap();

    lend.whitelist_market(lend.underlying_token_two.clone(), Decimal256::percent(90))
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address,
            QueryMsg::Markets {
                pagination: Pagination {
                    start: 0,
                    limit: 10,
                },
            },
        )
        .unwrap();

    assert_eq!(res.len(), 2)
}

#[test]
fn enter_and_exit_markets() {
    let mut lend = Lend::new();

    let base_market = lend
        .whitelist_market(lend.underlying_token_one.clone(), Decimal256::percent(90))
        .unwrap();

    let quote_market = lend
        .whitelist_market(lend.underlying_token_two.clone(), Decimal256::percent(90))
        .unwrap();

    // enter market
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![base_market.contract.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::EnteredMarkets {
                permit: Permit::<OverseerPermissions>::new(
                    "borrower",
                    vec![OverseerPermissions::AccountInfo],
                    vec![lend.overseer.address.clone()],
                    "balance",
                ),
            },
        )
        .unwrap();

    assert_eq!(res.len(), 1);

    // enter another market
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![quote_market.contract.address],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::EnteredMarkets {
                permit: Permit::<OverseerPermissions>::new(
                    "borrower",
                    vec![OverseerPermissions::AccountInfo],
                    vec![lend.overseer.address.clone()],
                    "balance",
                ),
            },
        )
        .unwrap();

    assert_eq!(res.len(), 2);

    // exit market
    lend.ensemble
        .execute(
            &HandleMsg::Exit {
                market_address: base_market.contract.address.clone(),
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::EnteredMarkets {
                permit: Permit::<OverseerPermissions>::new(
                    "borrower",
                    vec![OverseerPermissions::AccountInfo],
                    vec![lend.overseer.address.clone()],
                    "balance",
                ),
            },
        )
        .unwrap();

    assert_eq!(res.len(), 1);

    // cannot exit not entered market
    let res = lend.ensemble.execute(
        &HandleMsg::Exit {
            market_address: base_market.contract.address.clone(),
        },
        MockEnv::new("borrower", lend.overseer.clone()),
    );
    assert_eq!(
        StdError::generic_err("Not entered in market."),
        res.unwrap_err()
    );
}

#[test]
fn returns_right_liquidity() {
    let mut lend = Lend::new();

    let market = lend
        .whitelist_market(lend.underlying_token_three.clone(), Decimal256::percent(50))
        .unwrap();

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.underlying_token_three.clone()),
        )
        .unwrap();

    let res = lend
        .get_liquidity(
            Some(market.contract.address),
            Uint256::from(0u128),
            Uint256::from(0u128),
            None,
        )
        .unwrap();

    // should return amount * collateralFactor * exchangeRate * underlyingPrice
    let expected = ((Uint256::from(1_000_000u128)
        .decimal_mul(Decimal256::percent(50))
        .unwrap()
        * Uint256::from(1u128))
    .unwrap()
        * Uint256::from(1u128))
    .unwrap();

    assert_eq!(expected, res.liquidity);
}

#[test]
fn liquidity_collateral_factor() {
    // fails if a market is not listed
    let mut lend = Lend::new();
    let res = lend.ensemble.execute(
        &HandleMsg::Enter {
            markets: vec!["unknown_addr".into()],
        },
        MockEnv::new("borrower", lend.overseer.clone()),
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Market is not listed.")
    );

    let market = lend
        .whitelist_market(lend.underlying_token_three.clone(), Decimal256::percent(50))
        .unwrap();

    lend.set_oracle_price(market.symbol.as_bytes(), Uint128(1_000_000_000_000_000_000))
        .unwrap();

    // not in market yet, should have no effect
    let res = lend
        .get_liquidity(
            Some(market.contract.address.clone()),
            Uint256::from(1u128),
            Uint256::from(1u128),
            None,
        )
        .unwrap();

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    // deposit
    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.underlying_token_three.clone()),
        )
        .unwrap();

    // total account liquidity after supplying `amount`
    let res = lend
        .get_liquidity(
            Some(market.contract.address.clone()),
            Uint256::from(0u128),
            Uint256::from(0u128),
            None,
        )
        .unwrap();

    assert_eq!(
        Uint256::from(1000000u128)
            .decimal_mul(Decimal256::percent(50))
            .unwrap(),
        res.liquidity,
    );
    assert_eq!(Uint256::from(0u128), res.shortfall);

    // borrow amount, should shortfall over collateralFactor
    let res = lend
        .get_liquidity(
            Some(market.contract.address.clone()),
            Uint256::from(0u128),
            Uint256::from(1_000_000u128),
            Some(12346),
        )
        .unwrap();

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(
        Uint256::from(1000000u128)
            .decimal_mul(Decimal256::percent(50))
            .unwrap(),
        res.shortfall
    );

    // hypothetically redeem `amount`, should be back to even
    let res = lend
        .get_liquidity(
            Some(market.contract.address.clone()),
            Uint256::from(1_000_000u128),
            Uint256::from(0u128),
            Some(12346),
        )
        .unwrap();

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);
}

#[test]
fn liquidity_entering_markets() {
    // allows entering 3 markets, supplying to 2 and borrowing up to collateralFactor in the 3rd
    let mut lend = Lend::new();
    let market_one = lend
        .whitelist_market(lend.underlying_token_one.clone(), Decimal256::percent(50))
        .unwrap();
    let market_two = lend
        .whitelist_market(lend.underlying_token_two.clone(), Decimal256::permille(666))
        .unwrap();
    let market_three = lend
        .whitelist_market(lend.underlying_token_three.clone(), Decimal256::zero())
        .unwrap();

    // set underlying prices
    lend.set_oracle_price(
        market_one.symbol.as_bytes(),
        Uint128(3_000_000_000_000_000_000),
    )
    .unwrap();
    lend.set_oracle_price(
        market_two.symbol.as_bytes(),
        Uint128(2_718_000_000_000_000_000),
    )
    .unwrap();
    lend.set_oracle_price(
        market_three.symbol.as_bytes(),
        Uint128(1_000_000_000_000_000_000),
    )
    .unwrap();

    // enter markets
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market_one.contract.address.clone(),
                    market_two.contract.address.clone(),
                    market_three.contract.address.clone(),
                ],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    // supply to 2 markets
    // deposit
    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market_one.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.underlying_token_one.clone()),
        )
        .unwrap();

    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market_two.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.underlying_token_two.clone()),
        )
        .unwrap();

    let collateral_one = ((Uint256::from(1_000_000u128) * Uint256::from(3u128)).unwrap())
        .decimal_mul(Decimal256::percent(50))
        .unwrap();
    let collateral_two = (Uint256::from(1_000u128)
        .decimal_mul(Decimal256::from_str("2.718").unwrap())
        .unwrap())
    .decimal_mul(Decimal256::permille(666))
    .unwrap();
    let collateral_three = (collateral_one + collateral_two).unwrap();

    let res = lend
        .get_liquidity(None, Uint256::from(0u128), Uint256::from(0u128), None)
        .unwrap();
    assert_eq!(collateral_three, res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    let res = lend
        .get_liquidity(
            Some(market_three.contract.address.clone()),
            collateral_two,
            Uint256::from(0u128),
            None,
        )
        .unwrap();
    assert_eq!(collateral_three, res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    let res = lend
        .get_liquidity(
            Some(market_three.contract.address.clone()),
            Uint256::from(0u128),
            collateral_two,
            None,
        )
        .unwrap();
    assert_eq!(collateral_one, res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    let res = lend
        .get_liquidity(
            Some(market_three.contract.address.clone()),
            Uint256::from(0u128),
            (collateral_three + collateral_one).unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(collateral_one, res.shortfall);

    let res = lend
        .get_liquidity(
            Some(market_one.contract.address.clone()),
            Uint256::from(1_000_000u128),
            Uint256::from(0u128),
            None,
        )
        .unwrap();
    assert_eq!(collateral_two, res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);
}

#[test]
fn caclulate_amount_seize() {
    let mut lend = Lend::new();

    let collateral_market = lend
        .whitelist_market(lend.underlying_token_one.clone(), Decimal256::percent(50))
        .unwrap();
    let borrowed_market = lend
        .whitelist_market(lend.underlying_token_two.clone(), Decimal256::permille(666))
        .unwrap();

    let res: Uint256 = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::SeizeAmount {
                borrowed: borrowed_market.contract.address.clone(),
                collateral: collateral_market.contract.address.clone(),
                repay_amount: Uint256::from(1_000_000_000_000_000_000u128),
            },
        )
        .unwrap();
    assert_eq!(res, Uint256::from(1_000_000_000_000_000_000u128));
}
