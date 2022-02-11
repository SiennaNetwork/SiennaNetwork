use lend_shared::{
    impl_contract_storage,
    fadroma::{
        cosmwasm_std::{
            HumanAddr, CanonicalAddr, Extern,
            StdResult, Api, Storage, Querier,
            StdError, ReadonlyStorage
        },
        storage::{load, save, ns_load, ns_save, IterableStorage},
        Canonize, Humanize, ContractLink,
        ContractInstantiationInfo, Decimal256
    },
    interfaces::overseer::{Pagination, Market, Config},
    core::AuthenticatedUser
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;
const MAX_MARKETS_ENTERED: usize = 5;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Constants {
    pub premium: Decimal256,
    close_factor: Decimal256
}

pub struct Contracts;
pub struct Markets;

pub struct Whitelisting;

#[derive(Clone)]
pub struct Account(pub CanonicalAddr);

type UserMarkets = Vec<u64>;

impl Constants {
    const KEY: &'static [u8] = b"constants";

    #[inline]
    pub fn load(storage: &impl Storage) -> StdResult<Config> {
        Ok(load(storage, Self::KEY)?.unwrap())
    }

    #[inline]
    pub fn save(
        storage: &mut impl Storage,
        config: &Config
    ) -> StdResult<()> {
        save(storage, Self::KEY, config)
    }
}

impl Contracts {
    impl_contract_storage!(save_oracle, load_oracle, b"oracle");
    impl_contract_storage!(save_self_ref, load_self_ref, b"self");
}

impl Whitelisting {
    const KEY_MARKET_CONTRACT: &'static [u8] = b"market_contract";
    const KEY_PENDING: &'static [u8] = b"pending";

    #[inline]
    pub fn save_market_contract(
        storage: &mut impl Storage,
        contract: &ContractInstantiationInfo
    ) -> StdResult<()> {
        save(storage, Self::KEY_MARKET_CONTRACT, contract)
    }
    
    #[inline]
    pub fn load_market_contract(
        storage: &impl Storage
    ) -> StdResult<ContractInstantiationInfo> {
        Ok(load(storage, Self::KEY_MARKET_CONTRACT)?.unwrap())
    }

    pub fn set_pending(
        storage: &mut impl Storage,
        market: &Market<HumanAddr>
    ) -> StdResult<()> {
        save(storage, Self::KEY_PENDING, market)
    }

    pub fn pop_pending(
        storage: &mut impl Storage
    ) -> StdResult<Market<HumanAddr>> {
        let result: Option<Market<HumanAddr>> =
            load(storage, Self::KEY_PENDING)?;

        match result {
            Some(market) => {
                storage.remove(Self::KEY_PENDING);

                Ok(market)
            },
            None => Err(StdError::unauthorized())
        }
    }
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

    pub fn get_by_addr<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        market: &HumanAddr
    ) -> StdResult<(u64, Market<HumanAddr>)> {
        let id = Self::get_id(deps, market)?;
        let result = Self::load(
            &deps.storage,
            id
        )?.unwrap();

        Ok((id, result.humanize(&deps.api)?))
    }

    pub fn get_by_id<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        id: u64
    ) -> StdResult<Option<Market<HumanAddr>>> {
        let result = Self::load(&deps.storage, id)?;

        match result {
            Some(market) => Ok(Some(market.humanize(&deps.api)?)),
            None => Ok(None)
        }
    }

    pub fn get_id<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        market: &HumanAddr
    ) -> StdResult<u64> {
        let market = market.canonize(&deps.api)?;

        let result: Option<u64> = ns_load(
            &deps.storage,
            Self::NS,
            market.as_slice()
        )?;

        match result {
            Some(id) => Ok(id),
            None => Err(StdError::generic_err("Market is not listed."))
        }
    }

    pub fn update<S: Storage, A: Api, Q: Querier, F>(
        deps: &mut Extern<S, A, Q>,
        market: &HumanAddr,
        update: F
    ) -> StdResult<()>
        where F: FnOnce(Market<CanonicalAddr>) -> StdResult<Market<CanonicalAddr>>
    {
        let id = Self::get_id(deps, market)?;

        IterableStorage::new(Self::NS)
            .update_at(&mut deps.storage, id, update)?;
    
        Ok(())
    }

    pub fn list<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        pagination: Pagination
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        let limit = pagination.limit.min(PAGINATION_LIMIT);

        let storage = IterableStorage::new(Self::NS);
        let iterator = storage
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

    #[inline]
    fn load(
        storage: &impl Storage,
        index: u64
    ) -> StdResult<Option<Market<CanonicalAddr>>> {
        IterableStorage::new(Self::NS)
            .get_at(storage, index)
    }
}

impl Account {
    const NS: &'static [u8] = b"accounts";

    pub fn new(
        api: &impl Api,
        address: &HumanAddr
    ) -> StdResult<Self> {
        Ok(Self(address.canonize(api)?))
    }

    pub fn add_markets<S: Storage>(
        &self,
        storage: &mut S,
        ids: Vec<u64>
    ) -> StdResult<()> {
        let mut markets = self.load_markets(storage)?;

        if markets.len() + ids.len() > MAX_MARKETS_ENTERED {
            return Err(StdError::generic_err(format!(
                "Cannot enter more than {} markets at a time.",
                MAX_MARKETS_ENTERED
            )));
        }

        for id in ids {
            markets.push(id);
        }

        self.save_markets(storage, &markets)
    }

    pub fn remove_market<S: Storage>(
        &self,
        storage: &mut S,
        id: u64
    ) -> StdResult<()> {
        let mut markets = self.load_markets(storage)?;

        if let Some(index) = markets.iter().position(|x| *x == id) {
            markets.swap_remove(index);
            self.save_markets(storage, &markets)
        } else {
            Ok(())
        }
    }

    pub fn get_market<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
        address: &HumanAddr
    ) -> StdResult<(u64, Market<HumanAddr>)> {
        let (id, market) = Markets::get_by_addr(deps, address)?;

        let markets = self.load_markets(&deps.storage)?;

        match markets.into_iter().position(|x| x == id) {
            Some(_) => Ok((id, market)),
            None => Err(StdError::generic_err("Not entered in market."))
        }
    }

    pub fn list_markets<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        let markets = self.load_markets(&deps.storage)?;

        let mut result = Vec::with_capacity(markets.len());

        for id in markets {
            let market = Markets::get_by_id(deps, id)?.unwrap();

            result.push(market);
        }

        Ok(result)
    }

    #[inline]
    fn save_markets(
        &self,
        storage: &mut impl Storage,
        markets: &UserMarkets
    ) -> StdResult<()> {
        ns_save(storage, Self::NS, self.0.as_slice(), markets)
    }

    #[inline]
    fn load_markets(&self, storage: &impl ReadonlyStorage) -> StdResult<UserMarkets> {
        Ok(ns_load(storage, Self::NS, self.0.as_slice())?.unwrap_or_default())
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
