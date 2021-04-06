//! `HumanAddr`<->`CanonicalAddr` conversion for `Schedule`, `Pool`, and `Account`.

use crate::{Schedule, Pool, Account};
use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult};

impl Schedule<CanonicalAddr> {
    fn humanize <A:Api> (&self, api: &A) -> StdResult<Schedule<HumanAddr>> {
        let pools: Result<Vec<_>,_> = self.pools.iter().map(|p|p.humanize(api)).collect();
        Ok(Schedule {
            total: self.total,
            pools: pools?
        })
    }
}
impl Pool<CanonicalAddr> {
    fn humanize <A:Api> (&self, api: &A) -> StdResult<Pool<HumanAddr>> {
        let accounts: Result<Vec<_>,_> = self.accounts.iter().map(|a|a.humanize(api)).collect();
        Ok(Pool {
            name:     self.name.clone(),
            total:    self.total,
            partial:  self.partial,
            accounts: accounts?
        })
    }
}
impl Account<CanonicalAddr> {
    fn humanize <A:Api> (&self, api: &A) -> StdResult<Account<HumanAddr>> {
        Ok(Account {
            name:     self.name.clone(),
            amount:   self.amount,
            cliff:    self.cliff,
            start_at: self.start_at,
            interval: self.interval,
            duration: self.duration,
            address:  api.human_address(&self.address)?
        })
    }
}

impl Schedule<HumanAddr> {
    fn canonize <A:Api> (&self, api: &A) -> StdResult<Schedule<CanonicalAddr>> {
        let pools: Result<Vec<_>,_> = self.pools.iter().map(|p|p.canonize(api)).collect();
        Ok(Schedule {
            total: self.total,
            pools: pools?
        })
    }
}
impl Pool<HumanAddr> {
    fn canonize <A:Api> (&self, api: &A) -> StdResult<Pool<CanonicalAddr>> {
        let accounts: Result<Vec<_>,_> = self.accounts.iter().map(|a|a.canonize(api)).collect();
        Ok(Pool {
            name:     self.name.clone(),
            total:    self.total,
            partial:  self.partial,
            accounts: accounts?
        })
    }
}
impl Account<HumanAddr> {
    fn canonize <A:Api> (&self, api: &A) -> StdResult<Account<CanonicalAddr>> {
        Ok(Account {
            name:     self.name.clone(),
            amount:   self.amount,
            cliff:    self.cliff,
            start_at: self.start_at,
            interval: self.interval,
            duration: self.duration,
            address:  api.canonical_address(&self.address)?,
        })
    }
}
