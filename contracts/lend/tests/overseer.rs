use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::MockEnv,
        snip20_impl::msg::{
            HandleMsg as Snip20HandleMsg, InitMsg as Snip20InitMsg, InitialBalance,
        },
        Decimal256, StdError, Uint256, Uint128, to_binary
    },
    interfaces::{overseer::*, market},
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
fn liquidity() {
    // fails if a market is not listed
    let mut lend = Lend::new();
    let atom_market = lend.markets[1].clone();
    let res = lend.ensemble.execute(
        &HandleMsg::Enter {
            markets: vec![atom_market.address.clone()],
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
                    contract: lend.markets[1].clone(),
                    symbol: "ATOM".into(),
                    ltv_ratio: Decimal256::percent(50),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    // not in market yet, should have no effect
    let res = lend.get_liquidity(
        Some(atom_market.address.clone()),
        Uint256::from(1u128),
        Uint256::from(1u128),
        None,
    );

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![atom_market.address.clone()],
            },
            MockEnv::new("borrower", lend.overseer.clone()),
        )
        .unwrap();


    // deposit
    let _res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: atom_market.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", lend.atom_underlying_token.clone()),
        )
        .unwrap();

    // total account liquidity after supplying `amount`
    let res = lend.get_liquidity(
        Some(atom_market.address.clone()),
        Uint256::from(0u128),
        Uint256::from(0u128),
        None,
    );

    assert_eq!(
       Uint256::from(1000000u128).decimal_mul(Decimal256::percent(50)).unwrap(),
       res.liquidity, 
    );
    assert_eq!(Uint256::from(0u128), res.shortfall);

    // borrow amount, should shortfall over collateralFactor
    let res = lend.get_liquidity(
        Some(atom_market.address.clone()),
        Uint256::from(0u128),
        Uint256::from(1u128),
        Some(12346),
    );

    assert_eq!(Uint256::from(0u128), res.liquidity);
    assert_eq!(
        Uint256::from(1000000u128)
            .decimal_mul(Decimal256::percent(50))
            .unwrap(),
        res.shortfall
    );
}
