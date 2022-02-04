use std::convert::{TryFrom, TryInto};

use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Api, Binary, CanonicalAddr, Extern,
            HumanAddr, Querier, StdResult, Storage
        },
        schemars,
        schemars::JsonSchema,
        storage::{IterableStorage, load, save, ns_load, ns_save, ns_remove},
        crypto::sha_256,
        Canonize, ContractLink, Decimal256, Humanize, StdError, Uint256,
    },
    interfaces::market::{BorrowerInfo, Config},
    core::AuthenticatedUser,
    impl_contract_storage
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

pub struct Contracts;

pub struct Constants;

#[derive(PartialEq, Debug)]
pub struct Account(CanonicalAddr);

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct BorrowSnapshot {
    pub info: BorrowerInfo,
    address: CanonicalAddr,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Debug)]
pub struct BorrowerId([u8; 32]);

pub struct BorrowerRecord {
    pub id: Binary,
    pub address: HumanAddr,
    pub info: BorrowerInfo
}

pub struct Global;

impl Contracts {
    impl_contract_storage!(save_interest_model, load_interest_model, b"interest_model");
    impl_contract_storage!(save_overseer, load_overseer, b"overseer");
    impl_contract_storage!(save_underlying, load_underlying, b"underlying_asset");
    impl_contract_storage!(save_self_ref, load_self_ref, b"self");
}

impl Constants {
    const KEY_CONFIG: &'static [u8] = b"config";
    const KEY_VK: &'static [u8] = b"underlying_vk";

    pub fn save_config(
        storage: &mut impl Storage,
        config: &Config,
    ) -> StdResult<()> {
        save(storage, Self::KEY_CONFIG, &config)
    }

    pub fn load_config(storage: &impl Storage) -> StdResult<Config> {
        let result: Config = load(storage, Self::KEY_CONFIG)?.unwrap();

        Ok(result)
    }

    pub fn save_vk(
        storage: &mut impl Storage,
        key: &String,
    ) -> StdResult<()> {
        save(storage, Self::KEY_VK, &key)
    }

    pub fn load_vk(storage: &impl Storage) -> StdResult<String> {
        let result: String = load(storage, Self::KEY_VK)?.unwrap();

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
    const NS_BORROW_INFO: &'static [u8] = b"borrow_info";
    const NS_ID_TO_ADDR: &'static [u8] = b"ids";
    const NS_ADDR_TO_ID: &'static [u8] = b"addr";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        let account = Self(address.canonize(&deps.api)?);

        let is_new_user = Self::load_id(&deps.storage, &account.0)?.is_none();

        if is_new_user {
            let id = BorrowerId::new(deps, address)?;

            ns_save(&mut deps.storage, Self::NS_ID_TO_ADDR, id.as_slice(), &account.0.0)?;
            ns_save(&mut deps.storage, Self::NS_ADDR_TO_ID, account.0.0.as_slice(), &id)?;
        }

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
            ns_load(storage, Self::NS_ID_TO_ADDR, id.as_slice())?;
        
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
        borrow_info: BorrowSnapshot,
    ) -> StdResult<()> {
        let mut borrowers = IterableStorage::<BorrowSnapshot>::new(Self::NS_BORROWERS);

        let index = ns_load(storage, Self::NS_BORROW_INFO, self.0.as_slice())?;

        if let Some(index) = index {
            if borrow_info.info.principal.is_zero() {
                // If a swap occurred, update the stored borrower index to the new one. 
                if let Some(swapped) = borrowers.swap_remove(storage, index)? {
                    ns_save(storage, Self::NS_BORROW_INFO, swapped.address.as_slice(), &index)?;
                }

                ns_remove(storage, Self::NS_BORROW_INFO, self.0.as_slice());
            } else {
                borrowers.update_at(storage, index, |_| {
                    Ok(borrow_info)
                })?;
            }
        } else {
            let index = borrowers.push(storage, &borrow_info)?;
            ns_save(storage, Self::NS_BORROW_INFO, self.0.as_slice(), &index)?;
        }

        Ok(())
    }

    pub fn get_borrow_snapshot(&self, storage: &impl Storage) -> StdResult<BorrowSnapshot> {
        let index = ns_load(storage, Self::NS_BORROW_INFO, self.0.as_slice())?;

        match index {
            Some(index) => {
                let borrowers = IterableStorage::new(Self::NS_BORROWERS);
                
                Ok(borrowers.get_at(storage, index)?.unwrap())
            }
            None => Ok(BorrowSnapshot {
                address: self.0.clone(),
                info: BorrowerInfo::default()
            })
        }
    }

    #[inline]
    pub fn get_id(
        &self,
        storage: &impl Storage
    ) -> StdResult<Binary> {
        match Self::load_id(storage, &self.0)? {
            Some(id) => Ok(id.into()),
            None => Err(StdError::generic_err("ID doesn't exist. Need to have borrowed at least once."))
        }
    }

    #[inline]
    fn load_id(
        storage: &impl Storage,
        address: &CanonicalAddr
    ) -> StdResult<Option<BorrowerId>> {
        ns_load(storage, Self::NS_ADDR_TO_ID, address.as_slice())
    }

    #[inline]
    fn set_balance(&self, storage: &mut impl Storage, amount: &Uint256) -> StdResult<()> {
        ns_save(storage, Self::NS_BALANCES, self.0.as_slice(), amount)
    }
}

impl AuthenticatedUser for Account {
    fn from_canonical<S: Storage, A: Api, Q: Querier>(
        _deps: &Extern<S, A, Q>,
        address: CanonicalAddr
    ) -> StdResult<Self> {
        Ok(Self(address))
    }
}

pub fn load_borrowers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<u64>,
    limit: Option<u8>
) -> StdResult<Vec<BorrowerRecord>> {
    let borrowers = IterableStorage::<BorrowSnapshot>::new(Account::NS_BORROWERS);

    let limit = limit.unwrap_or(PAGINATION_LIMIT).min(PAGINATION_LIMIT) as usize;

    borrowers
        .iter(&deps.storage)?
        .skip(start_after.unwrap_or(0) as usize)
        .take(limit)
        .map(|item| {
            let item = item?;
            let id = Account::load_id(&deps.storage, &item.address)?
                .unwrap()
                .into();

            Ok(BorrowerRecord {
                id,
                address: item.address.humanize(&deps.api)?,
                info: item.info
            })
        })
        .collect()
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
