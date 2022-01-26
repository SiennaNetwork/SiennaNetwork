use std::str::FromStr;

use lend_shared::{
    fadroma::{
        ensemble::{ContractHarness, MockDeps, MockEnv},
        from_binary,
        snip20_impl::msg::HandleMsg as Snip20HandleMsg,
        to_binary, Binary, Composable, Decimal256, Env, HandleResponse, HumanAddr, InitResponse,
        Permit, StdError, StdResult, Uint128, Uint256,
    },
    interfaces::{market, overseer::*},
};

use crate::setup::Lend;
use crate::ADMIN;

const BORROWER: &str = "borrower";

pub struct MarketImpl;
impl ContractHarness for MarketImpl {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        lend_market::init(deps, env, from_binary(&msg)?, lend_market::DefaultImpl)
    }
    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        lend_market::handle(deps, env, from_binary(&msg)?, lend_market::DefaultImpl)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        match from_binary(&msg).unwrap() {
            market::QueryMsg::ExchangeRate { block: _ } => {
                let res: Option<Decimal256> = deps.get(b"exchange_rate").unwrap();
                match res {
                    Some(value) => to_binary(&value),
                    None => to_binary(&Decimal256::one()),
                }
            }
            _ => lend_market::query(deps, from_binary(&msg)?, lend_market::DefaultImpl),
        }
    }
}

#[test]
fn whitelist() {
    let mut lend = Lend::default();

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
    let mut lend = Lend::default();

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
            MockEnv::new(BORROWER, lend.overseer.clone()),
        )
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::EnteredMarkets {
                method: Permit::<OverseerPermissions>::new(
                    BORROWER,
                    vec![OverseerPermissions::AccountInfo],
                    vec![lend.overseer.address.clone()],
                    "balance",
                )
                .into(),
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
            MockEnv::new(BORROWER, lend.overseer.clone()),
        )
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::EnteredMarkets {
                method: Permit::<OverseerPermissions>::new(
                    BORROWER,
                    vec![OverseerPermissions::AccountInfo],
                    vec![lend.overseer.address.clone()],
                    "balance",
                )
                .into(),
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
            MockEnv::new(BORROWER, lend.overseer.clone()),
        )
        .unwrap();

    let res: Vec<Market<HumanAddr>> = lend
        .ensemble
        .query(
            lend.overseer.address.clone(),
            QueryMsg::EnteredMarkets {
                method: Permit::<OverseerPermissions>::new(
                    BORROWER,
                    vec![OverseerPermissions::AccountInfo],
                    vec![lend.overseer.address.clone()],
                    "balance",
                )
                .into(),
            },
        )
        .unwrap();

    assert_eq!(res.len(), 1);

    // cannot exit not entered market
    let res = lend.ensemble.execute(
        &HandleMsg::Exit {
            market_address: base_market.contract.address.clone(),
        },
        MockEnv::new(BORROWER, lend.overseer.clone()),
    );
    assert_eq!(
        StdError::generic_err("Not entered in market."),
        res.unwrap_err()
    );
}

#[test]
fn returns_right_liquidity() {
    let mut lend = Lend::default();

    let market = lend
        .whitelist_market(lend.underlying_token_three.clone(), Decimal256::percent(50))
        .unwrap();

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(BORROWER, lend.overseer.clone()),
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
            MockEnv::new(BORROWER, lend.underlying_token_three.clone()),
        )
        .unwrap();

    let res = lend
        .get_liquidity(
            BORROWER,
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
    let mut lend = Lend::default();
    let res = lend.ensemble.execute(
        &HandleMsg::Enter {
            markets: vec!["unknown_addr".into()],
        },
        MockEnv::new(BORROWER, lend.overseer.clone()),
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
            BORROWER,
            Some(market.contract.address.clone()),
            Uint256::from(1u128),
            Uint256::from(1u128),
            None,
        )
        .unwrap_err();

    assert_eq!(res, StdError::generic_err("Not entered in any markets."));

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(BORROWER, lend.overseer.clone()),
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
            MockEnv::new(BORROWER, lend.underlying_token_three.clone()),
        )
        .unwrap();

    // total account liquidity after supplying `amount`
    let res = lend
        .get_liquidity(
            BORROWER,
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
            BORROWER,
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
            BORROWER,
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
    let mut lend = Lend::default();
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
            MockEnv::new(BORROWER, lend.overseer.clone()),
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
            MockEnv::new(BORROWER, lend.underlying_token_one.clone()),
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
            MockEnv::new(BORROWER, lend.underlying_token_two.clone()),
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
        .get_liquidity(BORROWER, None, Uint256::from(0u128), Uint256::from(0u128), None)
        .unwrap();
    assert_eq!(collateral_three, res.liquidity);
    assert_eq!(Uint256::from(0u128), res.shortfall);

    let res = lend
        .get_liquidity(
            BORROWER,
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
            BORROWER,
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
            BORROWER,
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
            BORROWER,
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
fn calculate_amount_seize() {
    let mut lend = Lend::new(Some(Box::new(MarketImpl)), None);
    let collateral_market = lend
        .whitelist_market(lend.underlying_token_one.clone(), Decimal256::percent(50))
        .unwrap();
    let borrowed_market = lend
        .whitelist_market(lend.underlying_token_two.clone(), Decimal256::permille(666))
        .unwrap();

    let cases = [
        (
            Decimal256::one(),
            Uint128(1_000_000_000_000_000_000),
            Uint128(1_000_000_000_000_000_000),
            Decimal256::one(),
            Uint256::from(1_000_000_000_000_000_000u128),
            Uint256::from(1_000_000_000_000_000_000u128),
        ),
        (
            Decimal256::from_uint256(2u128).unwrap(),
            Uint128(1_000_000_000_000_000_000),
            Uint128(1_000_000_000_000_000_000),
            Decimal256::one(),
            Uint256::from(1_000_000_000_000_000_000u128),
            Uint256::from(5_000_000_000_000_000_00u128),
        ),
        (
            Decimal256::from_uint256(2u128).unwrap(),
            Uint128(2_000_000_000_000_000_000),
            Uint128(1_420_000_000_000_000_000),
            Decimal256::percent(130),
            Uint256::from(2_450_000_000_000_000_000u128),
            Uint256::from(2_242_957_746_478_873_238u128),
        ),
        (
            Decimal256::from_str("2.789").unwrap(),
            Uint128(5_230_480_842_000_000_000),
            Uint128(771_320_000_000_000_000_000),
            Decimal256::percent(130),
            Uint256::from(10_002_450_000_000_000_000_000u128),
            Uint256::from(316_160_966_319_693_285_35u128),
        ),
        (
            Decimal256::from_uint256(7_009_232_529_961_056_000_000_000u128).unwrap(),
            Uint128(2_527_872_631_724_044_500_000_000),
            Uint128(2_617_711_209_324_258_500_000_00),
            Decimal256::from_uint256(1_179_713_989_619_784_000u128).unwrap(),
            Uint256::from(7_790_468_414_639_561_000_000_000u128),
            Uint256::from(1_266_202_853_996_821_037_9u128),
        ),
    ];

    for case in cases.iter() {
        let (exchange_rate, borrowed, collateral, premium, repay, result) = case;
        // set exchange rate
        lend.ensemble
            .deps_mut(collateral_market.contract.address.clone(), |s| {
                s.set(b"exchange_rate", exchange_rate).unwrap();
            })
            .unwrap();

        // set underlying prices
        lend.set_oracle_price(borrowed_market.symbol.as_bytes(), *borrowed)
            .unwrap();
        lend.set_oracle_price(collateral_market.symbol.as_bytes(), *collateral)
            .unwrap();

        // set premium
        lend.ensemble
            .execute(
                &HandleMsg::SetPremium { premium: *premium },
                MockEnv::new(ADMIN, lend.overseer.clone()),
            )
            .unwrap();

            let res: Uint256 = lend
            .ensemble
            .query(
                lend.overseer.address.clone(),
                QueryMsg::SeizeAmount {
                    borrowed: borrowed_market.contract.address.clone(),
                    collateral: collateral_market.contract.address.clone(),
                    repay_amount: *repay,
                },
            )
            .unwrap();
        assert_eq!(res, *result);
    }
}
