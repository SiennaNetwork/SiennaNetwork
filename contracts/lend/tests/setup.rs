use lend_shared::fadroma::{
    ensemble::{ContractHarness, MockDeps},
    from_binary, schemars,
    schemars::JsonSchema,
    to_binary, Binary, Env, HandleResponse, InitResponse, StdError, StdResult, Uint128,
};

use serde::{Deserialize, Serialize};

use amm_snip20;
use lend_oracle;
use lend_overseer;

pub struct Token;

pub struct Overseer;

impl ContractHarness for Overseer {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        lend_overseer::init(deps, env, from_binary(&msg)?, lend_overseer::DefaultImpl)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        lend_overseer::handle(deps, env, from_binary(&msg)?, lend_overseer::DefaultImpl)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let result = lend_overseer::query(deps, from_binary(&msg)?, lend_overseer::DefaultImpl)?;

        to_binary(&result)
    }
}

pub struct Oracle;

impl ContractHarness for Oracle {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        lend_oracle::init(deps, env, from_binary(&msg)?, lend_oracle::DefaultImpl)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        lend_oracle::handle(deps, env, from_binary(&msg)?, lend_oracle::DefaultImpl)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let result = lend_oracle::query(deps, from_binary(&msg)?, lend_oracle::DefaultImpl)?;

        to_binary(&result)
    }
}

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
