use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{governance::Governance, poll::Poll};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GovernanceConfig {
    pub threshold: Option<u64>,
    pub quorum: Option<Decimal>,
    pub deadline: Option<u64>,
    pub reveal_committee: Option<Vec<HumanAddr>>,
}
impl GovernanceConfig {
    //metadata configuration
    pub const MIN_TITLE_LENGTH: usize = 8;
    pub const MAX_TITLE_LENGTH: usize = 64;
    pub const COMMITTEE_CAPACITY: usize = 3;

    pub const MIN_DESC_LENGTH: usize = 8;
    pub const MAX_DESC_LENGTH: usize = 1024;

    pub const DEFAULT_QUORUM_PERCENT: u64 = 33;
    pub const DEFAULT_TRESHOLD: u64 = 3500;
    pub const DEFAULT_DEADLINE: u64 = 7 * 24 * 60 * 60;

    //storage keys
    pub const THRESHOLD: &'static [u8] = b"/gov/threshold";
    pub const QUORUM: &'static [u8] = b"/gov/quorum";
    pub const DEADLINE: &'static [u8] = b"/gov/deadline";
    pub const VOTES: &'static [u8] = b"/gov/votes";
    pub const COMMITTEE: &'static [u8] = b"/gov/committee";
}
pub trait IGovernanceConfig<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    /// Commit initial contract configuration to storage.
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    fn get(core: &C) -> StdResult<Self>;
    fn threshold(core: &C) -> StdResult<u64>;
    fn quorum(core: &C) -> StdResult<Decimal>;
    fn deadline(core: &C) -> StdResult<u64>;
    fn commit_committee(committee: &Vec<HumanAddr>, core: &mut C) -> StdResult<()>;
    fn committee(core: &C) -> StdResult<Vec<HumanAddr>>;
    fn add_committee_member(core: &mut C, member: HumanAddr) -> StdResult<()>;
    fn remove_committee_member(core: &mut C, member: HumanAddr) -> StdResult<()>;

}
impl<S, A, Q, C> IGovernanceConfig<S, A, Q, C> for GovernanceConfig
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn initialize(&mut self, core: &mut C, _env: &Env) -> StdResult<Vec<CosmosMsg>> {
        core.set(Poll::TOTAL, 0)?;
        if self.reveal_committee.is_none() {
            self.reveal_committee = Some(vec![]);
        };
        self.store(core)
        // TODO: check for uninitialized fields after store
    }
    fn get(core: &C) -> StdResult<Self> {
        Ok(Self {
            deadline: Some(Self::deadline(core)?),
            quorum: Some(Self::quorum(core)?),
            threshold: Some(Self::threshold(core)?),
            reveal_committee: Some(Self::committee(core)?),
        })
    }

    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        let GovernanceConfig {
            deadline,
            threshold,
            quorum,
            reveal_committee,
        } = self;
        if let Some(deadline) = deadline {
            core.set(Self::DEADLINE, deadline)?;
        }
        if let Some(threshold) = threshold {
            core.set(Self::THRESHOLD, threshold)?;
        }
        if let Some(quorum) = quorum {
            core.set(Self::QUORUM, quorum)?;
        }
        if let Some(reveal_committee) = reveal_committee {
            Self::commit_committee(reveal_committee, core)?
        }
        Ok(vec![])
    }

    fn threshold(core: &C) -> StdResult<u64> {
        core.get::<u64>(Self::THRESHOLD)?
            .ok_or(StdError::generic_err("threshold not set"))
    }

    fn quorum(core: &C) -> StdResult<Decimal> {
        core.get::<Decimal>(Self::QUORUM)?
            .ok_or(StdError::generic_err("quorum not set"))
    }

    fn deadline(core: &C) -> StdResult<u64> {
        core.get::<u64>(Self::DEADLINE)?
            .ok_or(StdError::generic_err("deadline not set"))
    }

    fn commit_committee(reveal_committee: &Vec<HumanAddr>, core: &mut C) -> StdResult<()> {
        let canonized_committee: Vec<CanonicalAddr> = reveal_committee
            .iter()
            .map(|member| member.canonize(core.api()).unwrap())
            .collect();
        core.set(Self::COMMITTEE, canonized_committee)?;
        Ok(())
    }

    fn committee(core: &C) -> StdResult<Vec<HumanAddr>> {
        let canonized_committee =
            core.get::<Vec<CanonicalAddr>>(Self::COMMITTEE)?
                .ok_or(StdError::generic_err(
                    "failed to parse reveal committee from storage",
                ))?;
        let committee: Vec<HumanAddr> = canonized_committee
            .iter()
            .map(|member| member.humanize(core.api()).unwrap())
            .collect();
        Ok(committee)
    }

    fn add_committee_member(core: &mut C, member: HumanAddr) -> StdResult<()> {
        let mut committee = Self::committee(core)?;
        if committee.len() == Self::COMMITTEE_CAPACITY {
            return Err(StdError::generic_err("Committee already at maximum capacity"));
        }
        committee.push(member);
        Self::commit_committee(&committee, core)?;
        Ok(())
    }

    fn remove_committee_member(core: &mut C, member: HumanAddr) -> StdResult<()> {
        let committee = Self::committee(core)?
            .into_iter()
            .filter(|addr| addr != &member)
            .collect();
        Self::commit_committee(&committee, core)?;
        Ok(())
    }
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            threshold: Some(Self::DEFAULT_TRESHOLD),
            quorum: Some(Decimal::percent(Self::DEFAULT_QUORUM_PERCENT)),
            deadline: Some(Self::DEFAULT_DEADLINE),
            reveal_committee: Some(vec![]),
        }
    }
}
