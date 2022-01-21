use std::str::FromStr;

use lend_shared::{
    fadroma::{
        ensemble::MockEnv, snip20_impl::msg::HandleMsg as Snip20HandleMsg, to_binary, Decimal256,
        StdError, Uint128, Uint256,
    },
    interfaces::{market, overseer::*},
};

use crate::setup::Lend;
use crate::ADMIN;

#[test]
fn whitelist() {
    let mut lend = Lend::new();

    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.markets[0].clone(),
                    symbol: "SIENNA".into(),
                    ltv_ratio: Decimal256::percent(90),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    // cannot list a market a second time
    let res = lend.ensemble.execute(
        &HandleMsg::Whitelist {
            market: Market {
                contract: lend.markets[0].clone(),
                symbol: "SIENNA".into(),
                ltv_ratio: Decimal256::percent(90),
            },
        },
        MockEnv::new(ADMIN, lend.overseer.clone()),
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Token is already registered as collateral.")
    );

    // can list two different markets
    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.markets[1].clone(),
                    symbol: "ATOM".into(),
                    ltv_ratio: Decimal256::percent(90),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();
    let res = lend
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

    if let QueryResponse::Markets { whitelist } = res {
        assert_eq!(whitelist.len(), 2)
    }
}

#[test]
fn liquidity_collateral_factor() {
    // fails if a market is not listed
    let mut lend = Lend::new();
    let market = lend.markets[2].clone();
    let res = lend.ensemble.execute(
        &HandleMsg::Enter {
            markets: vec![market.address.clone()],
        },
        MockEnv::new("borrower", lend.overseer.clone()),
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Market is not listed.")
    );

    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.markets[2].clone(),
                    symbol: "SLSC".into(),
                    ltv_ratio: Decimal256::percent(50),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    // not in market yet, should have no effect
    let res = lend.get_liquidity(
        Some(market.address.clone()),
        Uint256::from(1u128),
        Uint256::from(1u128),
        None,
    );

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();

    // deposit
    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.secret_underlying_token.clone()),
        )
        .unwrap();

    // total account liquidity after supplying `amount`
    let res = lend.get_liquidity(
        Some(market.address.clone()),
        Uint256::from(0u128),
        Uint256::from(0u128),
        None,
    );

    assert_eq!(
        Uint256::from(1000000u128)
            .decimal_mul(Decimal256::percent(50))
            .unwrap(),
        res.liquidity,
    );
    assert_eq!(Uint256::from(0u128), res.shortfall);

    // borrow amount, should shortfall over collateralFactor
    let res = lend.get_liquidity(
        Some(market.address.clone()),
        Uint256::from(0u128),
        Uint256::from(1_000_000u128),
        Some(12346),
    );

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(
        Uint256::from(1000000u128)
            .decimal_mul(Decimal256::percent(50))
            .unwrap(),
        res.shortfall
    );

    // hypothetically redeem `amount`, should be back to even
    let res = lend.get_liquidity(
        Some(market.address.clone()),
        Uint256::from(1_000_000u128),
        Uint256::from(0u128),
        Some(12346),
    );

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);
}

#[test]
fn liquidity_entering_markets() {
    // allows entering 3 markets, supplying to 2 and borrowing up to collateralFactor in the 3rd
    let mut lend = Lend::new();
    let sienna_market = lend.markets[0].clone();
    let atom_market = lend.markets[1].clone();
    let secret_market = lend.markets[2].clone();

    // Whitelist the markets
    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.markets[0].clone(),
                    symbol: "SLSN".into(),
                    ltv_ratio: Decimal256::percent(50),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();
    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.markets[1].clone(),
                    symbol: "SLAT".into(),
                    ltv_ratio: Decimal256::permille(666),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();
    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.markets[2].clone(),
                    symbol: "SLSC".into(),
                    ltv_ratio: Decimal256::zero(),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    // enter markets
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![sienna_market.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![atom_market.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![secret_market.address.clone()],
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
                recipient: sienna_market.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.sienna_underlying_token.clone()),
        )
        .unwrap();

    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: atom_market.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.atom_underlying_token.clone()),
        )
        .unwrap();

    let res = lend.get_liquidity(
        None,
        Uint256::from(0u128),
        Uint256::from(0u128),
        None,
    );

    let collateral_one = ((Uint256::from(1_000_000u128) * Uint256::from(3u128)).unwrap())
        .decimal_mul(Decimal256::percent(50))
        .unwrap();
    let collateral_two = (Uint256::from(1_000u128).decimal_mul(Decimal256::from_str("2.718").unwrap()).unwrap())
        .decimal_mul(Decimal256::permille(666))
        .unwrap();
    let collateral_three = (collateral_one + collateral_two).unwrap();
    assert_eq!(collateral_three, res.liquidity);
}
