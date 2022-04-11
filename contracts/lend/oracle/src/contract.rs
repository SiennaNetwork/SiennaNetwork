mod state;

use serde::{Serialize, Deserialize};
use lend_shared::fadroma::{
    schemars, schemars::JsonSchema,
    admin,
    admin::{Admin, assert_admin},
    cosmwasm_std,
    derive_contract::*,
    HumanAddr, InitResponse, HandleResponse,
    QueryRequest, StdResult, WasmQuery, CosmosMsg,
    WasmMsg, Uint128, log, to_binary,
    ContractLink, Decimal256
};
use lend_shared::interfaces::oracle::{
    PriceResponse, Asset,
    AssetType, OverseerRef, ConfigResponse
};

use state::{Contracts, SymbolTable, get_symbol};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BandResponse {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceQuery {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::oracle",
    component(path = "admin")
)]
pub trait BandOracleConsumer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        source: ContractLink<HumanAddr>,
        initial_assets: Vec<Asset>,
        overseer: OverseerRef
    ) -> StdResult<InitResponse> {
        Contracts::save_source(deps, source)?;

        for asset in initial_assets {
            SymbolTable::save(deps, &asset)?;
        }

        let mut result = admin::DefaultImpl.new(admin, deps, env)?;

        match overseer {
            OverseerRef::NewInstance(callback) => {
                Contracts::save_overseer(deps, callback.contract.clone())?;

                result.messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: callback.contract.address,
                    callback_code_hash: callback.contract.code_hash,
                    send: vec![],
                    msg: callback.msg
                }));
            },
            OverseerRef::ExistingInstance(contract) => {
                Contracts::save_overseer(deps, contract)?;
            }
        }

        Ok(result)
    }

    #[handle]
    fn update_assets(assets: Vec<Asset>) -> StdResult<HandleResponse> {
        if Contracts::load_overseer(deps)?.address != env.message.sender {
            assert_admin(deps, &env)?;
        }

        for asset in assets {
            SymbolTable::save(deps, &asset)?;
        }

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "update_asset")],
            data: None
        })
    }

    #[query]
    fn config() -> StdResult<ConfigResponse> {
        Ok(ConfigResponse {
            overseer: Contracts::load_overseer(deps)?,
            source: Contracts::load_source(deps)?
        })
    }

    #[query]
    fn price(base: AssetType, quote: AssetType, decimals: u8) -> StdResult<PriceResponse> {
        let source = Contracts::load_source(deps)?;

        let res: BandResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: source.address,
            callback_code_hash: source.code_hash,
            msg: to_binary(&SourceQuery::GetReferenceData {
                base_symbol: get_symbol(deps, base)?,
                quote_symbol: get_symbol(deps, quote)?,
            })?,
        }))?;

        Ok(PriceResponse {
            rate: Decimal256((res.rate.u128() * 10u128.pow(18 - decimals as u32)).into()),
            last_updated_base: res.last_updated_base,
            last_updated_quote: res.last_updated_quote,
        })
    }
}
