use fadroma::{
    cosmwasm_std::{
        Api, BlockInfo, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage,
        Uint128,
    },
    schemars,
    storage::{load, ns_load, ns_save, save, IterableStorage},
    Canonize, ContractLink, Humanize,
};
use serde::{Deserialize, Serialize};
use sienna_schedule::{Schedule, Seconds};

use crate::{HistoryResponse, MGMTError};

pub struct Config;

impl Config {
    const KEY_TOKEN: &'static [u8] = b"token";
    const KEY_LAUNCHED: &'static [u8] = b"launched";
    const KEY_SCHEDULE: &'static [u8] = b"schedule";
    const KEY_PREFUNDED: &'static [u8] = b"prefunded";

    pub fn save_token<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        token: ContractLink<HumanAddr>,
    ) -> StdResult<()> {
        let token = token.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY_TOKEN, &token)
    }

    pub fn load_token<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
    ) -> StdResult<ContractLink<HumanAddr>> {
        let token: ContractLink<CanonicalAddr> = load(&deps.storage, Self::KEY_TOKEN)?.unwrap();

        token.humanize(&deps.api)
    }

    #[inline]
    pub fn set_is_prefunded(storage: &mut impl Storage, is_prefunded: bool) -> StdResult<()> {
        save(storage, Self::KEY_PREFUNDED, &is_prefunded)
    }

    #[inline]
    pub fn is_prefunded(storage: &impl Storage) -> StdResult<bool> {
        Ok(load(storage, Self::KEY_PREFUNDED)?.unwrap())
    }

    #[inline]
    pub fn set_launched(storage: &mut impl Storage, timestamp: Seconds) -> StdResult<()> {
        save(storage, Self::KEY_LAUNCHED, &timestamp)
    }

    #[inline]
    pub fn get_launched(storage: &impl Storage) -> StdResult<Option<Seconds>> {
        load(storage, Self::KEY_LAUNCHED)
    }

    #[inline]
    pub fn assert_launched(storage: &impl Storage) -> StdResult<Seconds> {
        match Self::get_launched(storage)? {
            Some(time) => Ok(time),
            None => Err(StdError::generic_err(MGMTError!(PRELAUNCH))),
        }
    }

    #[inline]
    pub fn assert_not_launched(storage: &impl Storage) -> StdResult<()> {
        match Self::get_launched(storage)? {
            Some(_) => Err(StdError::generic_err(MGMTError!(UNDERWAY))),
            None => Ok(()),
        }
    }

    pub fn save_schedule<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        schedule: Schedule<HumanAddr>,
    ) -> StdResult<()> {
        let schedule = schedule.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY_SCHEDULE, &schedule)
    }

    #[inline]
    pub fn load_schedule(storage: &impl Storage) -> StdResult<Schedule<CanonicalAddr>> {
        Ok(load(storage, Self::KEY_SCHEDULE)?.unwrap())
    }
}

pub struct Participant {
    pub address: CanonicalAddr,
    claimed: Uint128,
}

impl Participant {
    const KEY: &'static [u8] = b"participant";

    #[inline]
    pub fn new<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr,
    ) -> StdResult<Self> {
        let address = address.canonize(&deps.api)?;

        Ok(Self {
            claimed: ns_load(&deps.storage, Self::KEY, address.as_slice())?.unwrap_or_default(),
            address,
        })
    }

    #[inline]
    pub fn claimed(&self) -> Uint128 {
        self.claimed
    }

    pub fn set_claimed(&mut self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        self.claimed = amount;

        ns_save(storage, Self::KEY, self.address.as_slice(), &self.claimed)
    }
}

pub struct History;

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Claim<T> {
    pub claimant: T,
    pub amount: Uint128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}

impl Claim<CanonicalAddr> {
    pub fn new(participant: Participant, info: &BlockInfo, amount: Uint128) -> Self {
        Self {
            claimant: participant.address,
            timestamp: info.time,
            amount,
        }
    }
}

impl History {
    const KEY: &'static [u8] = b"history";
    const CURRENT_TOTAL: &'static [u8] = b"current_total";
    const LIMIT: u8 = 30;

    pub fn push(storage: &mut impl Storage, claim: Claim<CanonicalAddr>) -> StdResult<()> {
        let mut history = IterableStorage::new(Self::KEY);
        history.push(storage, &claim)?;

        Self::increment_total(storage, claim.amount)?;

        Ok(())
    }
    pub fn total_claimed(storage: &impl Storage) -> StdResult<Uint128> {
        let total: Uint128 = load(storage, Self::CURRENT_TOTAL)?.unwrap_or_default();
        Ok(total)
    }

    fn increment_total(storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        let mut current_total: Uint128 = load(storage, Self::CURRENT_TOTAL)?.unwrap_or_default();
        current_total += amount;
        save(storage, Self::CURRENT_TOTAL, &current_total)?;
        Ok(())
    }

    pub fn list<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        pagination: Pagination,
    ) -> StdResult<HistoryResponse> {
        let history = IterableStorage::<Claim<CanonicalAddr>>::new(Self::KEY);

        let limit = pagination.limit.min(Self::LIMIT) as usize;

        let entries = history
            .iter(&deps.storage)?
            .skip(pagination.start as usize)
            .take(limit)
            .map(|item| {
                let claim = item?;

                claim.humanize(&deps.api)
            })
            .collect::<StdResult<Vec<Claim<HumanAddr>>>>()?;

        Ok(HistoryResponse {
            entries,
            total: history.len(&deps.storage)?,
        })
    }
}

impl Canonize for Claim<HumanAddr> {
    type Output = Claim<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Claim {
            claimant: self.claimant.canonize(api)?,
            amount: self.amount,
            timestamp: self.timestamp,
        })
    }
}

impl Humanize for Claim<CanonicalAddr> {
    type Output = Claim<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Claim {
            claimant: self.claimant.humanize(api)?,
            amount: self.amount,
            timestamp: self.timestamp,
        })
    }
}
