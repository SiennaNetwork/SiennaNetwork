use crate::*;

/// Log of executed claims
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct History {
    pub history: Vec<ClaimedPortion>
}
impl History {
    pub fn new () -> Self { Self { history: vec![] } }
    /// Takes list of portions, returns the ones which aren't marked as claimed
    pub fn unclaimed (&mut self, claimable: Portions) -> Portions {
        // TODO throw if monotonicity of time is violated in eiter collection
        let claimed_portions: Portions =
            self.history.iter().map(|claimed| claimed.portion.clone()).collect();
        claimable.into_iter()
            .filter(|portion| {!claimed_portions.contains(portion)}).collect()
    }
    /// Marks a portion as claimed
    pub fn claim (&mut self, claimed: Seconds, portions: Portions) {
        for portion in portions.iter() {
            self.history.push(ClaimedPortion {
                claimed,
                portion: portion.clone()
            })
        }
    }
}

/// History entry
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimedPortion {
    portion: Portion,
    claimed: Seconds
}

impl Pool {
    pub fn add_channel (&mut self, ch: Channel) -> UsuallyOk {
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
    pub fn reallocate (&mut self, a: AllocationSet) -> UsuallyOk {
        match &self.periodic {
            None => {},
            Some(Periodic{cliff,..}) => if (*cliff).u128() > 0 {
                return Self::err_realloc_cliff(&self.name)
            }
        };
        for allocations in self.allocations.iter() {
            if allocations.t > a.t {
                return Self::err_realloc_time_travel(&self.name, a.t, allocations.t)
            }
        }
        self.allocations.push(a);
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
