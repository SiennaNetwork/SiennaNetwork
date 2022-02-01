use lend_shared::fadroma::{
    bucket, bucket_read, cosmwasm_std, derive_contract::*, schemars, schemars::JsonSchema, Bucket,
    HandleResponse, InitResponse, ReadonlyBucket, StdError, StdResult, Storage, Uint128,
};
use serde::{Deserialize, Serialize};

pub static PRICE: &[u8] = b"prices";

pub fn price_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(PRICE, storage)
}

pub fn price_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(PRICE, storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BandResponse {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}

#[contract(entry)]
pub trait BandOracleConsumer {
    #[init]
    fn new() -> StdResult<InitResponse> {
        Ok(InitResponse::default())
    }

    #[handle]
    fn set_price(symbol: String, price: Uint128) -> StdResult<HandleResponse> {
        price_w(&mut deps.storage).save(symbol.as_bytes(), &price)?;
        Ok(HandleResponse::default())
    }

    #[query]
    fn price(base_symbol: String, _quote_symbol: String) -> StdResult<BandResponse> {
        if let Some(price) = price_r(&deps.storage).may_load(base_symbol.as_bytes())? {
            return Ok(BandResponse {
                rate: price,
                last_updated_base: 0,
                last_updated_quote: 0,
            });
        }
        Err(StdError::generic_err("Missing Price Feed"))
    }

    #[query]
    fn prices(
        base_symbols: Vec<String>,
        _quote_symbols: Vec<String>,
    ) -> StdResult<Vec<BandResponse>> {
        let mut results = Vec::new();

        for sym in base_symbols {
            if let Some(price) = price_r(&deps.storage).may_load(sym.as_bytes())? {
                results.push(BandResponse {
                    rate: price,
                    last_updated_base: 0,
                    last_updated_quote: 0,
                });
            } else {
                return Err(StdError::GenericErr {
                    msg: "Missing Price Feed".to_string(),
                    backtrace: None,
                });
            }
        }
        Ok(results)
    }
}
