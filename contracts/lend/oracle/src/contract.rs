mod state;

use serde::{Serialize, Deserialize};
use lend_shared::fadroma::{
    schemars, schemars::JsonSchema,
    admin,
    admin::{Admin, assert_admin},
    Callback,
    cosmwasm_std,
    derive_contract::*,
    HumanAddr, InitResponse, HandleResponse,
    QueryRequest, StdResult, WasmQuery, CosmosMsg,
    WasmMsg, Uint128, log, to_binary,
    ContractLink, Decimal256
};
use lend_shared::interfaces::oracle::{
    PriceResponse, PricesResponse, Asset, AssetType
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
        callback: Callback<HumanAddr>
    ) -> StdResult<InitResponse> {
        Contracts::save_source(deps, source)?;
        Contracts::save_overseer(deps, callback.contract.clone())?;

        for asset in initial_assets {
            SymbolTable::save(deps, &asset)?;
        }

        let mut result = admin::DefaultImpl.new(admin, deps, env)?;
        result.messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: callback.contract.address,
            callback_code_hash: callback.contract.code_hash,
            send: vec![],
            msg: callback.msg
        }));

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
    fn price(base: AssetType, quote: AssetType) -> StdResult<PriceResponse> {
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
            rate: Decimal256(res.rate.u128().into()),
            last_updated_base: res.last_updated_base,
            last_updated_quote: res.last_updated_quote,
        })
    }

    #[query]
    fn prices(base: Vec<AssetType>, quote: Vec<AssetType>) -> StdResult<PricesResponse> {
        let source = Contracts::load_source(deps)?;

        let prices: Vec<BandResponse> = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: source.address,
            callback_code_hash: source.code_hash,
            msg: to_binary(&SourceQuery::GetReferenceDataBulk {
                base_symbols: base.into_iter()
                    .map(|x| get_symbol(deps, x))
                    .collect::<StdResult<Vec<String>>>()?,
                quote_symbols: quote.into_iter()
                    .map(|x| get_symbol(deps, x))
                    .collect::<StdResult<Vec<String>>>()?,
            })?,
        }))?;

        let prices: StdResult<Vec<PriceResponse>> = prices.into_iter().map(|price| 
            Ok(PriceResponse{
                rate: Decimal256::from_uint256(price.rate)?,
                last_updated_base: price.last_updated_base,
                last_updated_quote: price.last_updated_quote,
            })
        ).collect();

        let prices = prices?;

        Ok(PricesResponse { prices })
    }
}
