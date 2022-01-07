use std::convert::{TryFrom, TryInto};

use lend_shared::{
    impl_contract_storage,
    impl_contract_storage_option,
    fadroma::{
        schemars,
        schemars::JsonSchema,
        uint256::Uint256,
        cosmwasm_std::{
            HumanAddr, CanonicalAddr, Extern,
            StdResult, Api, Storage, Querier,
            StdError, Binary
        },
        cosmwasm_storage::{Bucket, ReadonlyBucket},
        crypto::sha_256,
        storage::{load, save, ns_load, ns_save, IterableStorage},
        Canonize, Humanize,
        ContractLink, Decimal256
    },
    interfaces::overseer::{Pagination, Market}
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Constants {
    pub close_factor: Decimal256,
    pub premium: Decimal256
}

pub struct Contracts;
pub struct Markets;

#[derive(Clone)]
pub struct Borrower {
    id: BorrowerId
}

#[derive(PartialEq, Clone, Debug)]
pub struct BorrowerId([u8; 32]);

impl Constants {
    const KEY: &'static [u8] = b"constants";

    pub fn load(storage: &impl Storage) -> StdResult<Self> {
        Ok(load(storage, Self::KEY)?.unwrap())
    }

    pub fn save(
        &self,
        storage: &mut impl Storage
    ) -> StdResult<()> {
        save(storage, Self::KEY, self)
    }
}

impl Contracts {
    impl_contract_storage!(save_oracle, load_oracle, b"oracle");
}

impl Markets {
    const NS: &'static [u8] = b"markets";

    pub fn push<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        market: &Market<HumanAddr>
    ) -> StdResult<()> {
        let market = market.canonize(&deps.api)?;

        if ns_load::<Option<u64>, _>(
            &deps.storage,
            Self::NS,
            market.contract.address.as_slice()
        )?.is_some() {
            return Err(StdError::generic_err("Token is already registered as collateral."));
        }

        let index = IterableStorage::new(Self::NS)
            .push(&mut deps.storage, &market)?;

        ns_save(
            &mut deps.storage,
            Self::NS,
            market.contract.address.as_slice(),
            &index
        )
    }

    pub fn get_info<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        market: &HumanAddr
    ) -> StdResult<ContractLink<HumanAddr>> {
        let market = market.canonize(&deps.api)?;
        let index: Option<u64> = ns_load(
            &deps.storage,
            Self::NS,
            market.as_slice()
        )?;

        match index {
            Some(index) => {
                let result: ContractLink<CanonicalAddr> =
                    IterableStorage::new(Self::NS)
                    .get_at(&deps.storage, index)?
                    .unwrap();
                
                result.humanize(&deps.api)
            },
            None => Err(StdError::generic_err(
                "Token is not registered as collateral.",
            ))
        }
    }

    pub fn update<S: Storage, A: Api, Q: Querier, F>(
        deps: &mut Extern<S, A, Q>,
        market: &HumanAddr,
        update: F
    ) -> StdResult<()>
        where F: FnOnce(Market<CanonicalAddr>) -> StdResult<Market<CanonicalAddr>>
    {
        let market = market.canonize(&deps.api)?;
        let index: Option<u64> = ns_load(
            &deps.storage,
            Self::NS,
            market.as_slice()
        )?;

        match index {
            Some(index) => {
                IterableStorage::new(Self::NS)
                    .update_at(&mut deps.storage, index, update)?;
                
                Ok(())
            },
            None => Err(StdError::generic_err(
                "Token is not registered as collateral.",
            ))
        }
    }

    pub fn whitelist<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        pagination: Pagination
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        let limit = pagination.limit.min(PAGINATION_LIMIT);

        let iterator = IterableStorage::new(Self::NS)
            .iter(&deps.storage)?
            .skip(pagination.start as usize)
            .take(limit as usize);

        let mut result = Vec::with_capacity(iterator.len());

        for elem in iterator {
            let elem: Market<CanonicalAddr> = elem?;
            result.push(elem.humanize(&deps.api)?);
        }

        Ok(result)
    }
}

impl Borrower {
    const NS_COLLATERALS: &'static [u8] = b"collaterals";

    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<Self> {
        Ok(Self {
            id: BorrowerId::new(deps, address)?
        })
    }

    pub fn from_base64(bin: Binary) -> StdResult<Self> {
        Ok(Self {
            id: BorrowerId::try_from(bin.0)?
        })
    }

    pub fn id(self) -> Binary {
        self.id.into()
    }

    pub fn save_collaterals_raw<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
        collaterals: Vec<ContractLink<CanonicalAddr>>
    ) -> StdResult<()> {
        let mut collaterals_bucket: Bucket<'_, S, Vec<ContractLink<CanonicalAddr>>> =
            Bucket::new(Self::NS_COLLATERALS, &mut deps.storage);

        if collaterals.is_empty() {
            collaterals_bucket.remove(self.id.as_slice());
        } else {
            collaterals_bucket.save(self.id.as_slice(), &collaterals)?;
        }
    
        Ok(())
    }

    pub fn load_collaterals_raw<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>
    ) -> StdResult<Vec<ContractLink<CanonicalAddr>>> {
        let collaterals_bucket: ReadonlyBucket<'_, S, Vec<ContractLink<CanonicalAddr>>> =
            ReadonlyBucket::new(Self::NS_COLLATERALS, &deps.storage);

        match collaterals_bucket.load(self.id.as_slice()) {
            Ok(collaterals) => Ok(collaterals),
            _ => Ok(vec![]),
        }
    }

    #[inline]
    pub fn load_collaterals<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>
    ) -> StdResult<Vec<ContractLink<HumanAddr>>> {
        self.load_collaterals_raw(deps)?.humanize(&deps.api)
    }
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

impl Into<Binary> for BorrowerId {
    fn into(self) -> Binary {
        Binary::from(self.0)
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
