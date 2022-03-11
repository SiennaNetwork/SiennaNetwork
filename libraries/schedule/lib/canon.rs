//! `HumanAddr`<->`CanonicalAddr` conversion for `Schedule`, `Pool`, and `Account`.
use fadroma::{
    cosmwasm_std::{Api, HumanAddr, CanonicalAddr, StdResult},
    Humanize, Canonize
};

use crate::{Schedule, Pool, Account};

impl Humanize for Schedule<CanonicalAddr> {
    type Output = Schedule<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Schedule { total: self.total, pools: self.pools.humanize(api) ? })
    }
}

impl Humanize for Pool<CanonicalAddr> {
    type Output = Pool<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Pool {
            name: self.name,
            total: self.total,
            partial: self.partial,
            accounts: self.accounts.humanize(api)?
        })
    }
}

impl Humanize for Account<CanonicalAddr> {
    type Output = Account<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Account {
            name: self.name,
            amount: self.amount,
            cliff: self.cliff,
            start_at: self.start_at,
            interval: self.interval,
            duration: self.duration,
            address: self.address.humanize(api)?
        })
    }
}

impl Canonize for Schedule<HumanAddr> {
    type Output = Schedule<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Schedule {
            total: self.total,
            pools: self.pools.canonize(api)?
        })
    }
}

impl Canonize for Pool<HumanAddr> {
    type Output = Pool<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Pool {
            name: self.name,
            total: self.total,
            partial: self.partial,
            accounts: self.accounts.canonize(api)?
        })
    }
}

impl Canonize for Account<HumanAddr> {
    type Output = Account<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Account {
            name: self.name,
            amount: self.amount,
            cliff: self.cliff,
            start_at: self.start_at,
            interval: self.interval,
            duration: self.duration,
            address: self.address.canonize(api)?
        })
    }
}
