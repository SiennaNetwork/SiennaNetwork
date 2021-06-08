//! `HumanAddr`<->`CanonicalAddr` conversion for `Schedule`, `Pool`, and `Account`.

use crate::{Schedule, Pool, Account};
use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult};

pub trait Humanize<T> {
    fn humanize <A: Api> (&self, api: &A) -> StdResult<T>;
}
impl<T: Humanize<U>, U> Humanize<Vec<U>> for Vec<T> {
    fn humanize <A: Api> (&self, api: &A) -> StdResult<Vec<U>> {
        self.iter().map(|x|x.humanize(api)).collect()
    }
}

pub trait Canonize<T> {
    fn canonize <A: Api> (&self, api: &A) -> StdResult<T>;
}
impl<T: Canonize<U>, U> Canonize<Vec<U>> for Vec<T> {
    fn canonize <A: Api> (&self, api: &A) -> StdResult<Vec<U>> {
        self.iter().map(|x|x.canonize(api)).collect()
    }
}

impl Humanize<Schedule<HumanAddr>> for Schedule<CanonicalAddr> {
    fn humanize <A:Api> (&self, api: &A) -> StdResult<Schedule<HumanAddr>> {
        Ok(Schedule { total: self.total, pools: self.pools.humanize(api) ? })
    }
}
impl Humanize<Pool<HumanAddr>> for Pool<CanonicalAddr> {
    fn humanize <A:Api> (&self, api: &A) -> StdResult<Pool<HumanAddr>> {
        let accounts = self.accounts.humanize(api)?;
        let &Pool { total, partial, .. } = self;
        Ok(Pool { name: self.name.clone(), total, partial, accounts })
    }
}
impl Humanize<Account<HumanAddr>> for Account<CanonicalAddr> {
    fn humanize <A:Api> (&self, api: &A) -> StdResult<Account<HumanAddr>> {
        let address = api.human_address(&self.address)?;
        let &Account { amount, cliff, start_at, interval, duration, .. } = self;
        let name = self.name.clone();
        Ok(Account { name, amount, cliff, start_at, interval, duration, address })
    }
}

impl Canonize<Schedule<CanonicalAddr>> for Schedule<HumanAddr> {
    fn canonize <A:Api> (&self, api: &A) -> StdResult<Schedule<CanonicalAddr>> {
        Ok(Schedule { total: self.total, pools: self.pools.canonize(api)? })
    }
}
impl Canonize<Pool<CanonicalAddr>> for Pool<HumanAddr> {
    fn canonize <A:Api> (&self, api: &A) -> StdResult<Pool<CanonicalAddr>> {
        let accounts = self.accounts.canonize(api)?;
        let &Pool { total, partial, .. } = self;
        Ok(Pool { name: self.name.clone(), total, partial, accounts })
    }
}
impl Canonize<Account<CanonicalAddr>> for Account<HumanAddr> {
    fn canonize <A: Api> (&self, api: &A) -> StdResult<Account<CanonicalAddr>> {
        let address = api.canonical_address(&self.address)?;
        let &Account { amount, cliff, start_at, interval, duration, .. } = self;
        let name = self.name.clone();
        Ok(Account { name, amount, cliff, start_at, interval, duration, address })
    }
}
