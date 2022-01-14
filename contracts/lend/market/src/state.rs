use std::convert::{TryFrom, TryInto};

use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Api, Binary, CanonicalAddr, Extern,
            HumanAddr, Querier, StdResult, Storage,
            Order
        },
        cosmwasm_storage::{Bucket, ReadonlyBucket},
        schemars,
        schemars::JsonSchema,
        storage::{load, save, ns_load, ns_save},
        crypto::sha_256,
        Canonize, ContractLink, Decimal256, Humanize, StdError, Uint128, Uint256,
    },
    interfaces::market::{BorrowerInfo, Borrower},
    impl_contract_storage
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

pub struct Contracts;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
    pub initial_exchange_rate: Decimal256,
    // Fraction of interest currently set aside for reserves
    pub reserve_factor: Decimal256,
}

pub struct Account(BorrowerId);

#[derive(Serialize, Deserialize, JsonSchema, Default, Debug)]
pub struct BorrowSnapshot(pub BorrowerInfo);

#[derive(PartialEq, Clone, Debug)]
pub struct BorrowerId([u8; 32]);

pub struct GlobalData;

impl Contracts {
    impl_contract_storage!(save_interest_model, load_interest_model, b"interest_model");
    impl_contract_storage!(save_overseer, load_overseer, b"interest_model");
    impl_contract_storage!(save_underlying, load_underlying, b"underlying_asset");
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

impl GlobalData {
    const KEY_BORROW_CAP: &'static [u8] = b"borrow_cap";
    const KEY_TOTAL_BORROWS: &'static [u8] = b"total_borrows";
    const KEY_TOTAL_RESERVES: &'static [u8] = b"total_reserves";
    const KEY_TOTAL_SUPPLY: &'static [u8] = b"total_supply";
    const KEY_BORROW_INDEX: &'static [u8] = b"borrow_index";
    const KEY_ACCRUAL_BLOCK_NUMBER: &'static [u8] = b"accrual_block_number";

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
    pub fn save_total_borrows(storage: &mut impl Storage, total: &Uint128) -> StdResult<()> {
        save(storage, Self::KEY_TOTAL_BORROWS, total)
    }

    #[inline]
    pub fn load_total_reserves(storage: &impl Storage) -> StdResult<Uint128> {
        Ok(load(storage, Self::KEY_TOTAL_RESERVES)?.unwrap_or_default())
    }

    #[inline]
    pub fn save_total_reserves(storage: &mut impl Storage, reserves: &Uint128) -> StdResult<()> {
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
        Ok(load(storage, Self::KEY_TOTAL_SUPPLY)?.unwrap_or_default())
    }

    #[inline]
    fn save_total_supply(storage: &mut impl Storage, total: &Uint128) -> StdResult<()> {
        save(storage, Self::KEY_TOTAL_SUPPLY, total)
    }

    #[inline]
    pub fn load_borrow_index(storage: &impl Storage) -> StdResult<Uint256> {
        Ok(load(storage, Self::KEY_BORROW_INDEX)?.unwrap_or_default())
    }

    #[inline]
    pub fn save_borrow_index(storage: &mut impl Storage, index: &Uint256) -> StdResult<()> {
        save(storage, Self::KEY_BORROW_INDEX, index)
    }

    #[inline]
    pub fn load_accrual_block_number(storage: &impl Storage) -> StdResult<u64> {
        Ok(load(storage, Self::KEY_ACCRUAL_BLOCK_NUMBER)?.unwrap_or_default())
    }

    #[inline]
    pub fn save_accrual_block_number(storage: &mut impl Storage, block: &u64) -> StdResult<()> {
        save(storage, Self::KEY_ACCRUAL_BLOCK_NUMBER, block)
    }
}

impl Account {
    const NS_BALANCES: &'static [u8] = b"balances";
    const NS_BORROWERS: &'static [u8] = b"borrowers";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        Ok(Self(BorrowerId::new(deps, address)?))
    }

    pub fn get_balance(&self, storage: &impl Storage) -> StdResult<Uint128> {
        let result: Option<Uint128> = ns_load(
            storage,
            Self::NS_BALANCES,
            self.0.as_slice()
        )?;

        Ok(result.unwrap_or_default())
    }

    pub fn add_balance(&self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        let account_balance = self.get_balance(storage)?;

        if let Some(new_balance) = account_balance.0.checked_add(amount.0) {
            self.set_balance(storage, Uint128(new_balance))
        } else {
            Err(StdError::generic_err(
                "This deposit would overflow your balance",
            ))
        }
    }

    pub fn subtract_balance(&self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        let account_balance = self.get_balance(storage)?;

        if let Some(new_balance) = account_balance.0.checked_sub(amount.0) {
            self.set_balance(storage, Uint128(new_balance))
        } else {
            Err(StdError::generic_err(format!(
                "insufficient funds: balance={}, required={}",
                account_balance, amount
            )))
        }
    }

    pub fn save_borrow_snapshot<S: Storage>(
        &self,
        storage: &mut S,
        borrow_info: &BorrowSnapshot,
    ) -> StdResult<()> {
        let mut borrower_bucket: Bucket<'_, S, BorrowSnapshot> =
            Bucket::new(Self::NS_BORROWERS, storage);

        borrower_bucket.save(&self.0.as_slice(), borrow_info)
    }

    pub fn get_borrow_snapshot(&self, storage: &impl Storage) -> StdResult<BorrowSnapshot> {
        let borrower_bucket = ReadonlyBucket::new(Self::NS_BORROWERS, storage);

        Ok(borrower_bucket.may_load(self.0.as_slice())?.unwrap_or_default())
    }

    #[inline]
    fn set_balance(&self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        ns_save(storage, Self::NS_BALANCES, self.0.as_slice(), &amount)
    }
}

pub fn load_all_borrowers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<Binary>,
    limit: Option<u8>
) -> StdResult<Vec<Borrower>> {
    let collaterals_bucket: ReadonlyBucket<'_, S, BorrowSnapshot> =
        ReadonlyBucket::new(Account::NS_BORROWERS, &deps.storage);

    let limit = limit.unwrap_or(PAGINATION_LIMIT).min(PAGINATION_LIMIT) as usize;
    let start = calc_range_start(start_after);

    collaterals_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let id = BorrowerId::try_from(k)?;

            Ok(Borrower {
                id: id.into(),
                info: v.0,
            })
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<Binary>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_slice().to_vec();
        v.push(1);
        v
    })
}

impl BorrowerId {
    const KEY: &'static [u8] = b"salt";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        let address = address.canonize(&deps.api)?;
        let salt = Self::load_prng_seed(&deps.storage)?;

        let data = vec![ address.as_slice(), salt.as_slice() ].concat();

        Ok(Self(sha_256(&data)))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn set_prng_seed(storage: &mut impl Storage, prng_seed: &Binary) -> StdResult<()> {
        let stored: Option<Binary> = load(storage, Self::KEY)?;

        // Should only set this once, otherwise will break the contract.
        if stored.is_some() {
            return Err(StdError::generic_err("Prng seed already set."));
        }

        save(storage, Self::KEY, prng_seed)
    }

    fn load_prng_seed(storage: &impl Storage) -> StdResult<Binary> {
        Ok(load(storage, Self::KEY)?.unwrap())
    }
}

impl From<BorrowerId> for Binary {
    fn from(id: BorrowerId) -> Self {
        Binary(id.0.into())
    }
}

impl TryFrom<Vec<u8>> for BorrowerId {
    type Error = StdError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok(data) => Ok(Self(data)),
            Err(_) => Err(StdError::generic_err("Couldn't create BorrowerId from bytes."))
        }
    }
}
