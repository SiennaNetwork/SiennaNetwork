use cosmwasm_std::StdResult;
use crate::*;

impl Pool {
    pub fn add_channel (&mut self, ch: Channel) -> StdResult<()> {
        ch.validate()?;
        self.validate()?;
        let allocated = self.channels_total()?;
        let unallocated = self.total.u128() - allocated;
        if ch.amount.u128() > unallocated {
            return Self::err_too_big(
                &self.name, ch.amount.u128(), unallocated, self.total.u128()
            )
        }
        self.channels.push(ch);
        Ok(())
    }
    define_errors!{
        err_too_big (name: &str, amount: u128, unallocated: u128, total: u128) ->
            ("pool {}: tried to add channel with size {}, which is more than the remaining {} of this pool's total {}",
                name, amount, unallocated, total)}
}

impl Channel {
    /// Allocations can be changed on the fly without affecting past vestings.
    /// FIXME: Allocations are timestamped with real time
    ///        but schedule measures time from `t_launch=0`
    ///        and allocations are realized in `Periodic`s
    ///        which only start measuring time at `start_at`
    ///        For now, reallocations should be forbidden
    ///        before the launch because trying to timestamp
    ///        an allocation would cause an underflow?
    pub fn reallocate (&mut self, t: Seconds, allocations: Vec<Allocation>) -> StdResult<()> {
        match &self.periodic {
            None => {},
            Some(Periodic{cliff,..}) => if (*cliff).u128() > 0 {
                return Self::err_realloc_cliff(&self.name)
            }
        };
        let t_max = self.allocations.iter().fold(0, |x,y|Seconds::max(x,y.0));
        if t < t_max {
            return Self::err_realloc_time_travel(&self.name, t, t_max)
        }
        self.allocations.push((t, allocations));
        self.validate()
    }
    define_errors!{
        err_realloc_cliff (name: &str) ->
            ("channel {}: reallocations for channels with cliffs are not supported",
                name)
        err_realloc_time_travel (name: &str, t: Seconds, t_max: Seconds) ->
            ("channel {}: can not reallocate in the past ({} < {})",
                name, t, t_max)}
}
