use lend_shared::fadroma::{
    decimal::one_token,
    ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv},
    from_binary, schemars,
    schemars::JsonSchema,
    snip20_impl::msg::{
        HandleAnswer, HandleMsg as Snip20HandleMsg, InitMsg as Snip20InitMsg, InitialBalance,
    },
    to_binary, Binary, ContractLink, Decimal, Decimal256, Env, HandleResponse, HumanAddr,
    InitResponse, StdError, StdResult, Uint128,
};

use lend_shared::interfaces::{interest_model, market, oracle, overseer};
use serde::{Deserialize, Serialize};

use crate::{impl_contract_harness_default, ADMIN};
use amm_snip20;
use lend_interest_model;
use lend_market;
use lend_oracle;
use lend_overseer;

pub struct Token;
impl ContractHarness for Token {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        amm_snip20::init(deps, env, from_binary(&msg)?)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        amm_snip20::handle(deps, env, from_binary(&msg)?)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        amm_snip20::query(deps, from_binary(&msg)?)
    }
}

pub struct Overseer;
impl_contract_harness_default!(Overseer, lend_overseer);

pub struct Oracle;
impl_contract_harness_default!(Oracle, lend_oracle);

pub struct Market;
impl_contract_harness_default!(Market, lend_market);

pub struct InterestModel;
impl_contract_harness_default!(InterestModel, lend_interest_model);

pub struct MockBand;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MockBandQuery {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}

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

    fn query(&self, _deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let msg = from_binary(&msg).unwrap();
        match msg {
            MockBandQuery::GetReferenceData {
                base_symbol: _,
                quote_symbol: _,
            } => to_binary(&lend_oracle::BandResponse {
                rate: Uint128(1_000_000_000_000_000_000),
                last_updated_base: 1628544285u64,
                last_updated_quote: 3377610u64,
            }),
            MockBandQuery::GetReferenceDataBulk {
                base_symbols,
                quote_symbols: _,
            } => {
                let mut results = Vec::new();
                let data = lend_oracle::BandResponse {
                    rate: Uint128(1_000_000_000_000_000_000),
                    last_updated_base: 1628544285u64,
                    last_updated_quote: 3377610u64,
                };

                for _ in base_symbols {
                    results.push(data.clone());
                }
                to_binary(&results)
            }
        }
    }
}

pub struct Lend {
    pub ensemble: ContractEnsemble,
    pub overseer: ContractLink<HumanAddr>,
    pub market: ContractLink<HumanAddr>,
}

impl Lend {
    pub fn new() -> Self {
        let mut ensemble = ContractEnsemble::new(50);

        let overseer = ensemble.register(Box::new(Overseer));
        let oracle = ensemble.register(Box::new(Oracle));
        let mock_band = ensemble.register(Box::new(MockBand));
        let market = ensemble.register(Box::new(Market));
        let token = ensemble.register(Box::new(Token));
        let interest = ensemble.register(Box::new(InterestModel));

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

        let interest_model = ensemble
            .instantiate(
                interest.id,
                &interest_model::InitMsg {
                    admin: None,
                    base_rate_year: Decimal256::percent(2),
                    multiplier_year: Decimal256::percent(2),
                    jump_multiplier_year: Decimal256::percent(2),
                    jump_threshold: Decimal256::percent(2),
                    blocks_year: Some(6311520),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "interest_model".into(),
                        code_hash: interest.code_hash,
                    },
                ),
            )
            .unwrap();

        let overseer = ensemble
            .instantiate(
                overseer.id,
                &overseer::InitMsg {
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
                    initial_exchange_rate: Decimal256::percent(20),
                    overseer_contract: overseer.clone(),
                    interest_model_contract: interest_model,
                    reserve_factor: Decimal256::one(),
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

        Self {
            ensemble,
            overseer,
            market,
        }
    }
}