use crate::*;

impl Schedule {
    pub fn validate (&self) -> StdResult<()> {
        let mut total = 0u128;
        let mut pools: Vec<String> = vec![];
        for pool in self.pools.iter() {
            pools.push(pool.name.clone());
            match pool.validate() {
                Ok(_)  => { total += pool.total.u128() },
                Err(e) => return Err(e)
            }
        }
        if total != self.total.u128() { return Self::err_total(total, self.total.u128()) }
        Ok(())
    }
    define_errors!{
        err_total (actual: u128, expected: u128) ->
            ("schedule: pools add up to {}, expected {}",
                actual, expected)}
}

impl Pool {
    pub fn validate (&self) -> StdResult<()> {
        let total = self.channels_total()?;
        let invalid_total = if self.partial {
            total > self.total.u128()
        } else {
            total != self.total.u128()
        };
        if invalid_total { return Self::err_total(&self.name, total, self.total.u128()) }
        Ok(())
    }
    define_errors!{
        err_total (name: &str, actual: u128, expected: u128) ->
            ("pool {}: channels add up to {}, expected {}",
                name, actual, expected)}
}

impl Channel {
    pub fn validate (&self) -> StdResult<()> {
        match &self.periodic {
            None => {},
            Some(periodic) => periodic.validate(&self)?
        }
        for (_, allocations) in self.allocations.iter() {
            let mut total_portion = 0u128;
            for Allocation { amount, .. } in allocations.iter() {
                total_portion += amount.u128()
            }
            let portion_size = self.portion_size()?;
            if total_portion > portion_size {
                return Self::err_total(&self.name, total_portion, portion_size);
            }
        }
        Ok(())
    }
    define_errors!{
        err_total (name: &str, total: u128, portion: u128) -> 
            ("channel {}: allocations add up to {}, expected {}",
                name, total, portion)}
}

impl Periodic {
    pub fn validate (&self, ch: &Channel) -> StdResult<()> {
        let Periodic{cliff,duration,interval,..} = self;
        if *duration < 1u64 { return Self::err_zero_duration(&ch.name) }
        if *interval < 1u64 { return Self::err_zero_interval(&ch.name) }
        if *cliff > ch.amount { return Self::err_cliff_gt_total(&ch.name, cliff.u128(), ch.amount.u128()) }
        for (_, allocations) in ch.allocations.iter() {
            if allocations.len() > 1 && cliff.u128() > 0 { return Self::err_periodic_cliff_multiple(&ch.name) }
        }
        self.portion_count(&ch.name)?;
        self.portion_size(&ch.name, ch.amount.u128())?;
        Ok(())
    }
    define_errors!{
        err_zero_duration (name: &str) ->
            ("channel {}: periodic vesting's duration can't be 0",
                name)
        err_zero_interval (name: &str) ->
            ("channel {}: periodic vesting's interval can't be 0",
                name)
        err_cliff_gt_total (name: &str, cliff: u128, amount: u128) ->
            ("channel {}: cliff {} can't be larger than total amount {}",
                name, cliff, amount)
        err_periodic_cliff_multiple (name: &str) ->
            ("channel {}: cliffs not supported alongside split allocations",
                name)}
}
