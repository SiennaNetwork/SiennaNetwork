use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::{ContractEnsemble, MockEnv},
        secret_toolkit::utils::InitCallback,
        snip20_impl::msg::{
            HandleAnswer, HandleMsg as Snip20HandleMsg, InitMsg as Snip20InitMsg, InitialBalance,
        },
        to_binary, Binary, Callback, ContractInstantiationInfo, ContractLink, Decimal256,
        HumanAddr, Uint128,
        StdError
    },
    interfaces::{market, oracle, overseer::*},
};

use crate::setup::{Market as MarketContract, MockBand, Oracle, Overseer, Token};
use crate::ADMIN;

pub struct InitResult {
    ensemble: ContractEnsemble,
    overseer: ContractLink<HumanAddr>,
    market: ContractLink<HumanAddr>,
}

fn init() -> InitResult {
    let mut ensemble = ContractEnsemble::new(50);

    let overseer = ensemble.register(Box::new(Overseer));
    let oracle = ensemble.register(Box::new(Oracle));
    let mock_band = ensemble.register(Box::new(MockBand));
    let market = ensemble.register(Box::new(MarketContract));
    let token = ensemble.register(Box::new(Token));

    let decimals = 6;
    let underlying_token = ensemble
        .instantiate(
            token.id,
            &Snip20InitMsg {
                name: "Underlying Token".into(),
                admin: None,
                symbol: "UNDR".into(),
                decimals,
                initial_allowances: None,
                initial_balances: Some(vec![InitialBalance {
                    address: ADMIN.into(),
                    amount: Uint128(one_token(decimals)),
                }]),
                prng_seed: Binary::from(b"whatever"),
                config: None,
                callback: None,
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "underlying_token".into(),
                    code_hash: token.code_hash.clone(),
                },
            ),
        )
        .unwrap();

    let mock_band = ensemble
        .instantiate(
            mock_band.id,
            &{},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "mock_band".into(),
                    code_hash: mock_band.code_hash,
                },
            ),
        )
        .unwrap();

    let overseer = ensemble
        .instantiate(
            overseer.id,
            &InitMsg {
                admin: None,
                prng_seed: Binary::from(b"whatever"),
                close_factor: Decimal256::from_uint256(50000000000000000u128).unwrap(),
                premium: Decimal256::one(),
                oracle_contract: oracle,
                oracle_source: mock_band,
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "overseer".into(),
                    code_hash: overseer.code_hash,
                },
            ),
        )
        .unwrap();

    let market = ensemble
        .instantiate(
            market.id,
            &market::InitMsg {
                admin: None,
                prng_seed: Binary::from(b"market"),
                underlying_asset: underlying_token,
                sl_token_info: token,
                initial_exchange_rate: Decimal256::percent(20),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "market".into(),
                    code_hash: market.code_hash,
                },
            ),
        )
        .unwrap();

    InitResult {
        ensemble,
        overseer,
        market,
    }
}

#[test]
fn test_init() {
    let _result = init();
}

#[test]
fn whitelist() {
    let mut result = init();

    result
        .ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: result.market.clone(),
                    symbol: "SIENNA".into(),
                    ltv_ratio: Decimal256::percent(90),
                },
            },
            MockEnv::new(ADMIN, result.overseer.clone()),
        )
        .unwrap();

    let res = result
        .ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: result.market,
                    symbol: "SIENNA".into(),
                    ltv_ratio: Decimal256::percent(90),
                },
            },
            MockEnv::new(ADMIN, result.overseer),
        );

    assert_eq!(res.unwrap_err(), StdError::generic_err("Token is already registered as collateral."));
}
