use lend_shared::fadroma::{
    cosmwasm_std::{Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage},
    cosmwasm_storage::{Bucket, ReadonlyBucket},
    schemars,
    schemars::JsonSchema,
    storage::{load, save},
    Canonize, ContractLink, Decimal256, Humanize, StdError, Uint128, Uint256,
};
use lend_shared::impl_contract_storage;
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

pub struct Contracts;

impl Contracts {
    impl_contract_storage!(save_interest_model, load_interest_model, b"interest_model");
    impl_contract_storage!(save_overseer, load_overseer, b"interest_model");
    impl_contract_storage!(save_underlying, load_underlying, b"underlying_asset");
    impl_contract_storage!(save_sl_token, load_sl_token, b"sl_token");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
    pub initial_exchange_rate: Decimal256,
    // Fraction of interest currently set aside for reserves
    pub reserve_factor: Decimal256,
}

impl Config {
    const KEY: &'static [u8] = b"config";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        config: &Self,
    ) -> StdResult<()> {
        save(&mut deps.storage, Self::KEY, &config)
    }

    pub fn load<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Self> {
        let result: Config = load(&deps.storage, Self::KEY)?.unwrap();

        Ok(result)
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
    const KEY: &'static [u8] = b"borrower";
    pub fn new(id: Binary) -> StdResult<Self> {
        Ok(Self { id })
    }
    pub fn store_borrow_info<S: Storage>(
        &self,
        storage: &mut S,
        borrow_info: &BorrowInfo,
    ) -> StdResult<()> {
        let mut borrower_bucket: Bucket<'_, S, BorrowInfo> = Bucket::new(Self::KEY, storage);
        borrower_bucket.save(&self.id.as_slice(), borrow_info)
    }

    pub fn read_borrow_info(&self, storage: &impl Storage) -> BorrowInfo {
        let borrower_bucket = ReadonlyBucket::new(Self::KEY, storage);
        match borrower_bucket.load(self.id.as_slice()) {
            Ok(v) => v,
            _ => BorrowInfo {
                principal: Uint256::zero(),
                interest_index: Decimal256::one(),
            },
        }
    }
}

pub struct GlobalData;

impl GlobalData {
    const KEY_BORROW_CAP: &'static [u8] = b"borrow_cap";
    const KEY_TOTAL_BORROWS: &'static [u8] = b"total_borrows";
    const KEY_TOTAL_RESERVES: &'static [u8] = b"total_reserves";
    const KEY_TOTAL_SUPPLY: &'static [u8] = b"total_supply";
    const KEY_BORROW_INDEX: &'static [u8] = b"borrow_index";

    #[inline]
    pub fn save_borrow_cap(storage: &mut impl Storage, borrow_cap: &Uint128) -> StdResult<()> {
        save(storage, Self::KEY_BORROW_CAP, borrow_cap)
    }

    #[inline]
    pub fn load_borrow_cap(storage: &impl Storage) -> StdResult<Option<Uint128>> {
        load(storage, Self::KEY_BORROW_CAP)
    }

    pub fn increase_total_borrows(
        storage: &mut impl Storage,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        let current = Self::load_total_borrows(storage)?;
        let new = current
            .0
            .checked_add(amount.0)
            .ok_or_else(|| StdError::generic_err("Total borrows amount overflowed."))?;

        let new = Uint128(new);
        Self::save_total_borrows(storage, &new)?;

        Ok(new)
    }

    pub fn decrease_borrow_cap(storage: &mut impl Storage, amount: Uint128) -> StdResult<Uint128> {
        let current = Self::load_total_borrows(storage)?;
        let new = (current - amount)?;

        Self::save_total_borrows(storage, &new)?;

        Ok(new)
    }

    #[inline]
    pub fn load_total_borrows(storage: &impl Storage) -> StdResult<Uint128> {
        Ok(load(storage, Self::KEY_TOTAL_BORROWS)?.unwrap_or_default())
    }

    #[inline]
    fn save_total_borrows(storage: &mut impl Storage, total: &Uint128) -> StdResult<()> {
        save(storage, Self::KEY_TOTAL_BORROWS, total)
    }

    #[inline]
    pub fn load_total_reserves(storage: &impl Storage) -> StdResult<Uint128> {
        Ok(load(storage, Self::KEY_TOTAL_RESERVES)?.unwrap_or_default())
    }

    #[inline]
    fn save_total_reserves(storage: &mut impl Storage, reserves: &Uint128) -> StdResult<()> {
        save(storage, Self::KEY_TOTAL_RESERVES, reserves)
    }

    pub fn increase_total_supply(
        storage: &mut impl Storage,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        let current = Self::load_total_supply(storage)?;
        let new = current
            .0
            .checked_add(amount.0)
            .ok_or_else(|| StdError::generic_err("Total supply amount overflowed."))?;

        let new = Uint128(new);
        Self::save_total_supply(storage, &new)?;

        Ok(new)
    }

    pub fn decrease_total_supply(
        storage: &mut impl Storage,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        let current = Self::load_total_supply(storage)?;
        let new = (current - amount)?;

        Self::save_total_supply(storage, &new)?;

        Ok(new)
    }

    #[inline]
    pub fn load_total_supply(storage: &impl Storage) -> StdResult<Uint128> {
        Ok(load(storage, Self::KEY_TOTAL_BORROWS)?.unwrap_or_default())
    }

    #[inline]
    fn save_total_supply(storage: &mut impl Storage, total: &Uint128) -> StdResult<()> {
        save(storage, Self::KEY_TOTAL_BORROWS, total)
    }

    #[inline]
    pub fn load_borrow_index(storage: &impl Storage) -> StdResult<Decimal256> {
        Ok(load(storage, Self::KEY_BORROW_INDEX)?.unwrap_or_default())
    }

    #[inline]
    fn save_borrow_index(storage: &mut impl Storage, index: &Decimal256) -> StdResult<()> {
        save(storage, Self::KEY_BORROW_INDEX, index)
    }
}
