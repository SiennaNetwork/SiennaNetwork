use std::str::FromStr;

use lend_shared::{
    core::Pagination,
    fadroma::{
        admin,
        decimal::one_token,
        ensemble::{ContractHarness, MockDeps, MockEnv},
        from_binary,
        snip20_impl::msg::HandleMsg as Snip20HandleMsg,
        to_binary, Binary, Composable, ContractLink, Decimal256, Env, HandleResponse, HumanAddr,
        InitResponse, Permit, StdError, StdResult, Uint128, Uint256,
    },
    interfaces::{market, oracle, overseer::*},
};

use crate::setup::{Lend, LendConfig, ADMIN};

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

    let underlying_1_symbol = "ONE";
    let underlying_1 = lend.new_underlying_token(underlying_1_symbol, 6).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 3).unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying_1.clone());
    lend.prefund_user(BORROWER, Uint128(5 * one_token(3)), underlying_2.clone());

    // can only be called by admin
    let res = lend.ensemble.execute(
        &HandleMsg::Whitelist {
            config: MarketInitConfig {
                admin: None,
                token_symbol: underlying_1_symbol.into(),
                prng_seed: Binary::from(b"seed_for_base_market"),
                entropy: Binary::from(b"entropy_for_base_market"),
                underlying_asset: underlying_1.clone(),
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

    let admin = HumanAddr::from("joe");
    lend.ensemble.execute(
        &HandleMsg::Whitelist {
            config: MarketInitConfig {
                admin: Some(admin.clone()),
                token_symbol: underlying_1_symbol.into(),
                prng_seed: Binary::from(b"seed_for_base_market"),
                entropy: Binary::from(b"entropy_for_base_market"),
                underlying_asset: underlying_1.clone(),
                ltv_ratio: Decimal256::percent(90),
                config: market::Config {
                    initial_exchange_rate: Decimal256::one(),
                    reserve_factor: Decimal256::one(),
                    seize_factor: Decimal256::one(),
                },
                interest_model_contract: lend.interest_model.clone(),
            },
        },
        MockEnv::new(ADMIN, lend.overseer.clone()),
    ).unwrap();

    lend.whitelist_market(underlying_2.clone(), Decimal256::percent(90), None, None)
        .unwrap();

    let res: MarketsResponse = lend
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

    assert_eq!(res.total, 2);
    assert_eq!(res.entries.len(), 2);

    let admin_res: HumanAddr = lend.ensemble.query(
        res.entries[0].contract.address.clone(),
        market::QueryMsg::Admin(admin::QueryMsg::Admin {})
    ).unwrap();

    assert_eq!(admin_res, admin);

    let admin_res: HumanAddr = lend.ensemble.query(
        res.entries[1].contract.address.clone(),
        market::QueryMsg::Admin(admin::QueryMsg::Admin {})
    ).unwrap();

    assert_eq!(admin_res, ADMIN.into());
}

#[test]
fn enter_and_exit_markets() {
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 3).unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying_1.clone());
    lend.prefund_user(BORROWER, Uint128(5 * one_token(3)), underlying_2.clone());

    let base_market = lend
        .whitelist_market(underlying_1.clone(), Decimal256::percent(90), None, None)
        .unwrap();

    let quote_market = lend
        .whitelist_market(underlying_2.clone(), Decimal256::percent(90), None, None)
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
    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();
    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying_1.clone());

    let market = lend
        .whitelist_market(underlying_1.clone(), Decimal256::percent(50), None, None)
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
            MockEnv::new(BORROWER, underlying_1.clone()),
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

    // should return amount * collateralFactor * exchangeRate * underlyingPrice with 18 decimals
    let expected = ((Uint256::from(1_000_000_000_000_000_000u128)
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
    let mut lend = Lend::default();
    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying_1.clone());

    // fails if a market is not listed
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
        .whitelist_market(underlying_1.clone(), Decimal256::percent(50), None, None)
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
            MockEnv::new(BORROWER, underlying_1.clone()),
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

    assert_eq!(Uint256::from(5_000_000_000_000_000_00u128), res.liquidity,);
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
    assert_eq!(Uint256::from(5_000_000_000_000_000_00u128), res.shortfall);

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
    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();
    let underlying_3 = lend.new_underlying_token("TRES", 18).unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(18)), underlying_1.clone());
    lend.prefund_user(BORROWER, Uint128(5 * one_token(18)), underlying_2.clone());
    lend.prefund_user(BORROWER, Uint128(5 * one_token(18)), underlying_3.clone());

    let market_one = lend
        .whitelist_market(underlying_1.clone(), Decimal256::percent(50), None, None)
        .unwrap();
    let market_two = lend
        .whitelist_market(underlying_2.clone(), Decimal256::permille(666), None, None)
        .unwrap();
    let market_three = lend
        .whitelist_market(underlying_3.clone(), Decimal256::zero(), None, None)
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
            MockEnv::new(BORROWER, underlying_1.clone()),
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
            MockEnv::new(BORROWER, underlying_2.clone()),
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
        .get_liquidity(
            BORROWER,
            None,
            Uint256::from(0u128),
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
    let config = LendConfig::new().market(Box::new(MarketImpl));
    let mut lend = Lend::new(config);

    let underlying_1 = lend.new_underlying_token("ONE", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("TWO", 18).unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(18)), underlying_1.clone());
    lend.prefund_user(BORROWER, Uint128(5 * one_token(18)), underlying_2.clone());

    let collateral_market = lend
        .whitelist_market(underlying_1.clone(), Decimal256::percent(50), None, None)
        .unwrap();
    let borrowed_market = lend
        .whitelist_market(underlying_2.clone(), Decimal256::permille(666), None, None)
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
                &HandleMsg::ChangeConfig {
                    premium_rate: Some(*premium),
                    close_factor: None,
                    oracle: None,
                },
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

#[test]
fn liquidity_oracle_low_price() {
    let mut lend = Lend::default();
    let underlying = lend.new_underlying_token("ONE", 6).unwrap();

    let market = lend
        .whitelist_market(underlying.clone(), Decimal256::percent(50), None, None)
        .unwrap();

    lend.set_oracle_price(market.symbol.as_bytes(), Uint128(1u128))
        .unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying.clone());

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(BORROWER, lend.overseer.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new("MALLORY", lend.overseer.clone()),
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
            MockEnv::new(BORROWER, underlying.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1_000u128),
            },
            MockEnv::new("MALLORY", market.contract.clone()),
        )
        .unwrap_err();
}

#[test]
fn test_ltv() {
    let mut lend = Lend::default();
    let underlying1 = lend.new_underlying_token("ONE", 6).unwrap();
    let underlying2 = lend.new_underlying_token("TWO", 6).unwrap();

    // whitelist markets
    // markets with ltv_ratio of 0 are not valid collaterals
    let market1 = lend
        .whitelist_market(underlying1.clone(), Decimal256::zero(), None, None)
        .unwrap();

    let market2 = lend
        .whitelist_market(underlying2.clone(), Decimal256::percent(10), None, None)
        .unwrap();

    lend.set_oracle_price(market1.symbol.as_bytes(), Uint128(1))
        .unwrap();

    lend.set_oracle_price(
        market2.symbol.as_bytes(),
        Uint128(10_000_000_000_000_000_000u128),
    )
    .unwrap();

    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying1.clone());
    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying2.clone());

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market1.contract.address.clone(),
                    market2.contract.address.clone(),
                ],
            },
            MockEnv::new(BORROWER, lend.overseer.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market1.contract.address.clone(),
                    market2.contract.address.clone(),
                ],
            },
            MockEnv::new("MALLORY", lend.overseer.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market1.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new(BORROWER, underlying1.clone()),
        )
        .unwrap();

    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market2.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(1_000_000),
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new(BORROWER, underlying2.clone()),
        )
        .unwrap();

    // borrow fails because no collateral provided
    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1_000u128),
            },
            MockEnv::new("MALLORY", market2.contract.clone()),
        )
        .unwrap_err();

    // liquidity should be 0
    let res = lend
        .get_liquidity(
            "MALLORY",
            Some(market1.contract.address.clone()),
            Uint256::from(1_000_000u128),
            Uint256::from(0u128),
            None,
        )
        .unwrap();

    assert_eq!(
        res,
        AccountLiquidity {
            shortfall: Uint256::zero(),
            liquidity: Uint256::zero(),
        }
    );

    // set adequate price for 1st market asset
    lend.set_oracle_price(
        market1.symbol.as_bytes(),
        Uint128(1_000_000_000_000_000_000),
    )
    .unwrap();

    // provide some collateral in 1st market, should not affect liquidity because ltv is 0
    lend.prefund_and_deposit(
        "MALLORY",
        Uint128(1_000_000),
        market1.contract.address.clone(),
    );

    let res = lend
        .get_liquidity(
            "MALLORY",
            Some(market1.contract.address.clone()),
            Uint256::from(0u128),
            Uint256::from(0u128),
            None,
        )
        .unwrap();

    assert_eq!(
        res,
        AccountLiquidity {
            shortfall: Uint256::zero(),
            liquidity: Uint256::zero(),
        }
    );
    // borrow still fails
    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1_000u128),
            },
            MockEnv::new("MALLORY", market2.contract.clone()),
        )
        .unwrap_err();

    // provide some collateral to 2nd market, where ltv is 50%, borrow should succeed
    lend.prefund_and_deposit(
        "MALLORY",
        Uint128(1_000_00),
        market2.contract.address.clone(),
    );

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1_000_0u128),
            },
            MockEnv::new("MALLORY", market2.contract.clone()),
        )
        .unwrap();

    // new user enters market with 0 ltv_ratio and tries to borrow
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![market1.contract.address.clone()],
            },
            MockEnv::new("ALICE", lend.overseer.clone()),
        )
        .unwrap();

    // borrow fails
    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1_000u128),
            },
            MockEnv::new("ALICE", market1.contract.clone()),
        )
        .unwrap_err();

    // provide funds to 0 ltv_ratio market and try to borrow again
    lend.prefund_and_deposit(
        "ALICE",
        Uint128(1_000_000),
        market2.contract.address.clone(),
    );

    // should still fail
    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: Uint256::from(1_000u128),
            },
            MockEnv::new("ALICE", market1.contract.clone()),
        )
        .unwrap_err();

    // crash the price of first token and liquidate
    lend.set_oracle_price(market2.symbol.as_bytes(), Uint128(1))
        .unwrap();
    let id = lend.id("MALLORY", market2.contract.address.clone());

    // should fail with invalid price
    let res = lend
        .ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market2.contract.address.clone(),
                recipient_code_hash: None,
                amount: Uint128(4_000_000),
                msg: Some(
                    to_binary(&market::ReceiverCallbackMsg::Liquidate {
                        borrower: id.clone(),
                        collateral: market2.contract.address.clone(),
                    })
                    .unwrap(),
                ),
                memo: None,
                padding: None,
            },
            MockEnv::new(BORROWER, underlying2.clone()),
        )
        .unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("Invalid price reported by the oracle.")
    );

    // try to transfer
    let res = lend
        .ensemble
        .execute(
            &market::HandleMsg::Transfer {
                recipient: "MALLORY".into(),
                amount: Uint256::from(1_000_000u128),
            },
            MockEnv::new(BORROWER, market2.contract.clone()),
        )
        .unwrap_err();
    assert_eq!(
        res,
        StdError::generic_err("Invalid price reported by the oracle.")
    );

    // set valid price and try to transfer again
    lend.set_oracle_price(
        market2.symbol.as_bytes(),
        Uint128(1_000_000_000_000_000_000u128),
    )
    .unwrap();

    // should be ok
    lend.ensemble
        .execute(
            &market::HandleMsg::Transfer {
                recipient: "MALLORY".into(),
                amount: Uint256::from(1_000_000u128),
            },
            MockEnv::new(BORROWER, market2.contract.clone()),
        )
        .unwrap();
}

#[test]
fn faulty_oracle_price_causes_liquidity_check_to_error() {
    let bob = "bob";
    let mallory = "mallory";
    let alice = "alice";

    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("SLATOM", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("SLSCRT", 18).unwrap();

    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(70),
            Some(Decimal256::percent(20)),
            Some(Decimal256::one()),
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2,
            Decimal256::percent(70),
            Some(Decimal256::percent(20)),
            Some(Decimal256::one()),
        )
        .unwrap();

    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market_2.symbol.as_bytes(), Uint128(10000))
        .unwrap();

    lend.prefund_user(bob, Uint128(1000), underlying_1.clone());

    lend.prefund_user(mallory, Uint128(300), underlying_1);

    lend.prefund_and_deposit(alice, Uint128(100), market_2.contract.address.clone());

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market_1.contract.address.clone(),
                    market_2.contract.address.clone(),
                ],
            },
            MockEnv::new(bob, lend.overseer.clone()),
        )
        .unwrap();

    let err = lend
        .ensemble
        .execute(
            &market::HandleMsg::Borrow { amount: 100.into() },
            MockEnv::new(bob, market_2.contract),
        )
        .unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err("Invalid price reported by the oracle.")
    );
}

struct MockBand;

impl ContractHarness for MockBand {
    fn init(&self, _deps: &mut MockDeps, _env: Env, _msg: Binary) -> StdResult<InitResponse> {
        Ok(InitResponse::default())
    }

    fn handle(&self, _deps: &mut MockDeps, _env: Env, _msg: Binary) -> StdResult<HandleResponse> {
        Err(StdError::GenericErr {
            msg: "Not Implemented".to_string(),
            backtrace: None,
        })
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let msg = from_binary(&msg).unwrap();

        match msg {
            lend_oracle::SourceQuery::GetReferenceData { base_symbol, .. } => {
                let key: &[u8] = base_symbol.as_bytes();
                match deps.get(key).unwrap() {
                    Some(value) => to_binary(&lend_oracle::BandResponse {
                        rate: value,
                        last_updated_base: 1628544285u64,
                        last_updated_quote: 3377610u64,
                    }),
                    None => Err(StdError::generic_err(format!(
                        "No price for {} found.",
                        String::from_utf8(key.into()).unwrap()
                    ))),
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[test]
fn change_oracle() {
    let mallory = "mallory";
    let alice = "alice";

    let mut lend = Lend::default();

    let prefund_amount = 200u128;
    let borrow_amount = 100u128;

    let underlying_1 = lend.new_underlying_token("ATOM", 18).unwrap();
    let underlying_2 = lend.new_underlying_token("SSCRT", 18).unwrap();

    let market_1 = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::one(),
            Some(Decimal256::percent(20)),
            Some(Decimal256::one()),
        )
        .unwrap();

    let market_2 = lend
        .whitelist_market(
            underlying_2,
            Decimal256::one(),
            Some(Decimal256::percent(20)),
            Some(Decimal256::one()),
        )
        .unwrap();

    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market_2.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();

    lend.prefund_and_deposit(
        alice,
        prefund_amount.into(),
        market_2.contract.address.clone(),
    );

    lend.prefund_and_deposit(
        mallory,
        prefund_amount.into(),
        market_1.contract.address.clone(),
    );

    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market_1.contract.address.clone(),
                    market_2.contract.address.clone(),
                ],
            },
            MockEnv::new(mallory, lend.overseer.clone()),
        )
        .unwrap();

    let liquidity = lend
        .get_liquidity(mallory, None, Uint256::zero(), Uint256::zero(), None)
        .unwrap();

    assert_eq!(liquidity.liquidity, prefund_amount.into());
    assert_eq!(liquidity.shortfall, Uint256::zero());

    lend.ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(mallory, market_2.contract.clone()),
        )
        .unwrap();

    let info = lend.ensemble.register(Box::new(MockBand));
    lend.mock_band = lend
        .ensemble
        .instantiate(
            info.id,
            &{},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "mock_band_new".into(),
                    code_hash: info.code_hash.clone(),
                },
            ),
        )
        .unwrap();

    lend.set_oracle_price(market_1.symbol.as_bytes(), Uint128(1 * one_token(18)))
        .unwrap();

    let consumer = lend
        .ensemble
        .instantiate(
            2,
            &oracle::InitMsg {
                admin: None,
                source: lend.mock_band.clone(),
                initial_assets: vec![],
                overseer: oracle::OverseerRef::ExistingInstance(lend.overseer.clone()),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "new_oracle".into(),
                    code_hash: info.code_hash,
                },
            ),
        )
        .unwrap();

    let old: ContractLink<HumanAddr> = lend
        .ensemble
        .query(lend.overseer.address.clone(), QueryMsg::OracleContract {})
        .unwrap();

    lend.ensemble
        .execute(
            &HandleMsg::ChangeConfig {
                premium_rate: None,
                close_factor: None,
                oracle: Some(consumer.clone()),
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    let new: ContractLink<HumanAddr> = lend
        .ensemble
        .query(lend.overseer.address.clone(), QueryMsg::OracleContract {})
        .unwrap();

    assert_ne!(old, new);
    assert_eq!(new, consumer);

    let err = lend
        .get_liquidity(mallory, None, Uint256::zero(), Uint256::zero(), None)
        .unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err(format!("No price for {} found.", market_2.symbol))
    );

    let err = lend
        .ensemble
        .execute(
            &market::HandleMsg::Borrow {
                amount: borrow_amount.into(),
            },
            MockEnv::new(mallory, market_2.contract),
        )
        .unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err(format!("No price for {} found.", market_2.symbol))
    );

    let config: oracle::ConfigResponse = lend
        .ensemble
        .query(new.address, oracle::QueryMsg::Config {})
        .unwrap();

    assert_eq!(config.overseer, lend.overseer);
    assert_eq!(config.source, lend.mock_band);
}

#[test]
fn different_tokens_liquidity() {
    let mut lend = Lend::default();

    let alice = "ALICE";

    let underlying1 = lend.new_underlying_token("ATOM", 8).unwrap();
    let underlying2 = lend.new_underlying_token("SCRT", 6).unwrap();
    let underlying3 = lend.new_underlying_token("LUNA", 9).unwrap();
    let underlying4 = lend.new_underlying_token("ETH", 18).unwrap();

    // ltv 70%, ex rate 0.02
    let market1 = lend
        .whitelist_market(
            underlying1.clone(),
            Decimal256::percent(70),
            Some(Decimal256::percent(2)),
            Some(Decimal256::zero()),
        )
        .unwrap();

    // ltv 70%, ex rate 0.02
    let market2 = lend
        .whitelist_market(
            underlying2.clone(),
            Decimal256::percent(70),
            Some(Decimal256::percent(2)),
            Some(Decimal256::zero()),
        )
        .unwrap();

    // ltv 70%, ex rate 0.02
    let market3 = lend
        .whitelist_market(
            underlying3.clone(),
            Decimal256::percent(70),
            Some(Decimal256::percent(2)),
            Some(Decimal256::zero()),
        )
        .unwrap();

    // ltv 70%, ex rate 0.02
    let market4 = lend
        .whitelist_market(
            underlying4.clone(),
            Decimal256::percent(70),
            Some(Decimal256::percent(2)),
            Some(Decimal256::zero()),
        )
        .unwrap();

    // set prices
    lend.set_oracle_price(market1.symbol.as_bytes(), Uint128(24 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market2.symbol.as_bytes(), Uint128(5 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market3.symbol.as_bytes(), Uint128(84 * one_token(18)))
        .unwrap();
    lend.set_oracle_price(market4.symbol.as_bytes(), Uint128(3000 * one_token(18)))
        .unwrap();

    // Deposit 10 ATOM and 10 SCRT
    lend.prefund_and_deposit(
        alice,
        Uint128(10 * one_token(8)),
        market1.contract.address.clone(),
    );

    lend.prefund_and_deposit(
        alice,
        Uint128(10 * one_token(6)),
        market2.contract.address.clone(),
    );

    // Enter markets
    lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market1.contract.address.clone(),
                    market2.contract.address.clone(),
                ],
            },
            MockEnv::new(alice, lend.overseer.clone()),
        )
        .unwrap();

    // Verify liquidity is correct
    // (10 / 0.02 * (0.7 * 0.02 * 24)) + (10 / 0.02 * (0.7 * 0.02 * 5)) = 203$
    let liquidity = lend
        .get_liquidity(alice, None, Uint256::zero(), Uint256::zero(), None)
        .unwrap();

    assert_eq!(
        liquidity.liquidity,
        Uint256::from(203_000_000_000_000_000_000u128)
    );

    // Hypothetical liquidity after borrowing 5 SCRT = 178$
    let liquidity = lend
        .get_liquidity(
            alice,
            Some(market2.contract.address.clone()),
            Uint256::zero(),
            Uint256::from(5_000_000u128),
            None,
        )
        .unwrap();

    assert_eq!(
        liquidity.liquidity,
        Uint256::from(178_000_000_000_000_000_000u128)
    );

    // Can't borrow more than liquidity
    // Attempt to borrow 41 SCRT should fall 0.4 SCRT short * 5$ = 2$
    let liquidity = lend
        .get_liquidity(
            alice,
            Some(market2.contract.address.clone()),
            Uint256::zero(),
            Uint256::from(41_000_000u128),
            None,
        )
        .unwrap();

    assert_eq!(
        liquidity.shortfall,
        Uint256::from(2_000_000_000_000_000_000u128)
    );

    // Hypothetical liquidity after borrowing 5 ATOM(120$) = 83$
    let liquidity = lend
        .get_liquidity(
            alice,
            Some(market1.contract.address.clone()),
            Uint256::zero(),
            Uint256::from(5_000_000_00u128),
            None,
        )
        .unwrap();

    assert_eq!(
        liquidity.liquidity,
        Uint256::from(83_000_000_000_000_000_000u128)
    );

    // Deposit some 10 LUNA and 10 ETH
     lend.prefund_and_deposit(
        alice,
        Uint128(10 * one_token(9)),
        market3.contract.address.clone(),
    );

    lend.prefund_and_deposit(
        alice,
        Uint128(10 * one_token(18)),
        market4.contract.address.clone(),
    );
    
    // Enter markets
     lend.ensemble
        .execute(
            &HandleMsg::Enter {
                markets: vec![
                    market3.contract.address.clone(),
                    market4.contract.address.clone(),
                ],
            },
            MockEnv::new(alice, lend.overseer.clone()),
        )
        .unwrap();

    // Liquidity now should be
    // (10 / 0.02 * (0.7 * 0.02 * 24)) + (10 / 0.02 * (0.7 * 0.02 * 5)) +
    // (10 / 0.02 * (0.7 * 0.02 * 84)) + (10 / 0.02 * (0.7 * 0.02 * 3000)) = 21791$
    let liquidity = lend
        .get_liquidity(alice, None, Uint256::zero(), Uint256::zero(), None)
        .unwrap();

    assert_eq!(
        liquidity.liquidity,
        Uint256::from(21791_000_000_000_000_000_000u128)
    );
}