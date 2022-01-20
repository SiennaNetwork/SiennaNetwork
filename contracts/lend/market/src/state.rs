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
        Canonize, ContractLink, Decimal256, Humanize, StdError, Uint256,
    },
    interfaces::market::{BorrowerInfo, Borrower, Config},
    impl_contract_storage
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

pub struct Contracts;

pub struct Constants;

#[derive(PartialEq, Debug)]
pub struct Account(CanonicalAddr);

#[derive(Serialize, Deserialize, JsonSchema, Default, Debug)]
pub struct BorrowSnapshot(pub BorrowerInfo);

#[derive(PartialEq, Clone, Debug)]
pub struct BorrowerId([u8; 32]);

pub struct Global;

impl Contracts {
    impl_contract_storage!(save_interest_model, load_interest_model, b"interest_model");
    impl_contract_storage!(save_overseer, load_overseer, b"interest_model");
    impl_contract_storage!(save_underlying, load_underlying, b"underlying_asset");
    impl_contract_storage!(save_self_ref, load_self_ref, b"self");
}

impl Constants {
    const KEY: &'static [u8] = b"config";

    pub fn save(
        storage: &mut impl Storage,
        config: &Config,
    ) -> StdResult<()> {
        save(storage, Self::KEY, &config)
    }

    pub fn load(storage: &impl Storage) -> StdResult<Config> {
        let result: Config = load(storage, Self::KEY)?.unwrap();

        Ok(result)
    }
}

macro_rules! impl_uint_storage {
    ($name:ident, $data_type:ty, $key:literal) => {
        pub struct $name;

        impl $name {
            pub fn increase(
                storage: &mut impl Storage,
                amount: $data_type,
            ) -> StdResult<$data_type> {
                let current = Self::load(storage)?;
                let new = (current + amount)?;
        
                Self::save(storage, &new)?;
        
                Ok(new)
            }
        
            pub fn decrease(
                storage: &mut impl Storage,
                amount: $data_type,
            ) -> StdResult<$data_type> {
                let current = Self::load(storage)?;
                let new = (current - amount)?;
        
                Self::save(storage, &new)?;
        
                Ok(new)
            }
        
            #[inline]
            pub fn load(storage: &impl Storage) -> StdResult<$data_type> {
                Ok(load(storage, $key)?.unwrap_or_default())
            }
        
            #[inline]
            pub fn save(storage: &mut impl Storage, new: &$data_type) -> StdResult<()> {
                save(storage, $key, new)
            }
        }
    };
}

impl_uint_storage!(TotalBorrows, Uint256, b"total_borrows");
impl_uint_storage!(TotalSupply, Uint256, b"total_supply");

impl Global {
    const KEY_BORROW_CAP: &'static [u8] = b"borrow_cap";
    const KEY_BORROW_INDEX: &'static [u8] = b"borrow_index";
    const KEY_INTEREST_RESERVE: &'static [u8] = b"interest_reserve";
    const KEY_ACCRUAL_BLOCK_NUMBER: &'static [u8] = b"accrual_block_number";

    #[inline]
    pub fn save_borrow_cap(storage: &mut impl Storage, borrow_cap: &Uint256) -> StdResult<()> {
        save(storage, Self::KEY_BORROW_CAP, borrow_cap)
    }

    #[inline]
    pub fn load_borrow_cap(storage: &impl Storage) -> StdResult<Option<Uint256>> {
        load(storage, Self::KEY_BORROW_CAP)
    }

    #[inline]
    pub fn load_borrow_index(storage: &impl Storage) -> StdResult<Decimal256> {
        Ok(load(storage, Self::KEY_BORROW_INDEX)?.unwrap_or_default())
    }

    #[inline]
    pub fn save_borrow_index(storage: &mut impl Storage, index: &Decimal256) -> StdResult<()> {
        save(storage, Self::KEY_BORROW_INDEX, index)
    }

    #[inline]
    pub fn load_interest_reserve(storage: &impl Storage) -> StdResult<Uint256> {
        Ok(load(storage, Self::KEY_INTEREST_RESERVE)?.unwrap_or_default())
    }

    #[inline]
    pub fn save_interest_reserve(storage: &mut impl Storage, new: &Uint256) -> StdResult<()> {
        save(storage, Self::KEY_INTEREST_RESERVE, new)
    }

    #[inline]
    pub fn load_accrual_block_number(storage: &impl Storage) -> StdResult<u64> {
        Ok(load(storage, Self::KEY_ACCRUAL_BLOCK_NUMBER)?.unwrap_or_default())
    }

    #[inline]
    pub fn save_accrual_block_number(storage: &mut impl Storage, block: u64) -> StdResult<()> {
        save(storage, Self::KEY_ACCRUAL_BLOCK_NUMBER, &block)
    }
}

impl Account {
    const NS_BALANCES: &'static [u8] = b"balances";
    const NS_BORROWERS: &'static [u8] = b"borrowers";
    const NS_ID: &'static [u8] = b"ids";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        let account = Self(address.canonize(&deps.api)?);

        let id = BorrowerId::new(deps, address)?;
        ns_save(&mut deps.storage, Self::NS_ID, id.as_slice(), &account.0.0)?;

        Ok(account)
    }

    pub fn of<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        Ok(Self(address.canonize(&deps.api)?))
    }

    pub fn from_id(
        storage: &impl Storage,
        id: &Binary
    ) -> StdResult<Self> {
        let result: Option<Binary> =
            ns_load(storage, Self::NS_ID, id.as_slice())?;
        
        match result {
            Some(address) => Ok(Self(CanonicalAddr(address))),
            None => Err(StdError::generic_err(format!(
                "Account with id {} doesn't exist.",
                id
            )))
        }
    }

    pub fn address(&self, api: &impl Api) -> StdResult<HumanAddr> {
        self.0.humanize(api)
    }

    pub fn get_balance(&self, storage: &impl Storage) -> StdResult<Uint256> {
        let result: Option<Uint256> = ns_load(
            storage,
            Self::NS_BALANCES,
            self.0.as_slice()
        )?;

        Ok(result.unwrap_or_default())
    }

    pub fn add_balance(&self, storage: &mut impl Storage, amount: Uint256) -> StdResult<()> {
        let account_balance = self.get_balance(storage)?;

        if let Ok(new_balance) = account_balance + amount {
            self.set_balance(storage, &new_balance)
        } else {
            Err(StdError::generic_err(
                "This deposit would overflow your balance",
            ))
        }
    }

    pub fn subtract_balance(&self, storage: &mut impl Storage, amount: Uint256) -> StdResult<()> {
        let account_balance = self.get_balance(storage)?;

        if let Ok(new_balance) = account_balance - amount {
            self.set_balance(storage, &new_balance)
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
    fn set_balance(&self, storage: &mut impl Storage, amount: &Uint256) -> StdResult<()> {
        ns_save(storage, Self::NS_BALANCES, self.0.as_slice(), amount)
    }
}

impl From<CanonicalAddr> for Account {
    fn from(address: CanonicalAddr) -> Self {
        Account(address)
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
