use std::convert::{TryFrom, TryInto};

use lend_shared::fadroma::{
    cosmwasm_std::{Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage},
    cosmwasm_storage::{Bucket, ReadonlyBucket},
    schemars,
    schemars::JsonSchema,
    storage::{load, save},
    Canonize, ContractLink, Decimal256, Humanize, Uint256
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

const PREFIX_BORROWER: &[u8] = b"borrower";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A> {
    pub underlying_asset: ContractLink<A>,
    pub overseer_contract: ContractLink<A>,
    pub sl_token: ContractLink<A>,
    pub interest_model_contract: ContractLink<A>,
    // Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
    pub initial_exchange_rate: Decimal256,
    // Fraction of interest currently set aside for reserves
    pub reserve_factor: Decimal256,
}

impl Config<HumanAddr> {
    const KEY: &'static [u8] = b"config";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        config: &Self,
    ) -> StdResult<()> {
        let config = config.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY, &config)
    }

    pub fn load<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Self> {
        let result: Config<CanonicalAddr> = load(&deps.storage, Self::KEY)?.unwrap();

        result.humanize(&deps.api)
    }
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            underlying_asset: self.underlying_asset.canonize(api)?,
            overseer_contract: self.overseer_contract.canonize(api)?,
            sl_token: self.sl_token.canonize(api)?,
            interest_model_contract: self.interest_model_contract.canonize(api)?,
            initial_exchange_rate: self.initial_exchange_rate,
            reserve_factor: self.reserve_factor,
        })
    }
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            underlying_asset: self.underlying_asset.humanize(api)?,
            overseer_contract: self.overseer_contract.humanize(api)?,
            sl_token: self.sl_token.humanize(api)?,
            interest_model_contract: self.interest_model_contract.humanize(api)?,
            initial_exchange_rate: self.initial_exchange_rate,
            reserve_factor: self.reserve_factor,
        })
    }
}

pub struct Borrower {
    /// The id must be created by Overseer.
    id: Binary,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowInfo {
    /// Total balance (with accrued interest), after applying the most recent balance-changing action
    principal: Uint256,
    /// Global borrowIndex as of the most recent balance-changing action
    interest_index: Decimal256,
}

impl Borrower {
    pub fn new(id: Binary) -> StdResult<Self> {
        Ok(Self { id })
    }
    pub fn store_borrow_info<S: Storage>(
        &self,
        storage: &mut S,
        borrow_info: &BorrowInfo,
    ) -> StdResult<()> {
        let mut borrower_bucket: Bucket<'_, S, BorrowInfo> =
            Bucket::new(PREFIX_BORROWER, storage);
        borrower_bucket.save(&self.id.as_slice(), borrow_info)
    }

    pub fn read_borrow_info(&self, storage: &impl Storage) -> BorrowInfo {
        let borrower_bucket = ReadonlyBucket::new(PREFIX_BORROWER, storage);
        match borrower_bucket.load(self.id.as_slice()) {
            Ok(v) => v,
            _ => BorrowInfo {
                principal: Uint256::zero(),
                interest_index: Decimal256::one(),
            },
        }
    }
}
