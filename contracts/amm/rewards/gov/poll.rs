use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    expiration::Expiration,
    governance::Governance,
    poll_metadata::{IPollMetaData, PollMetadata},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub metadata: PollMetadata,
    pub expiration: Expiration,
    pub status: PollStatus,
    pub reveal_approvals: Vec<HumanAddr>
}

impl Poll {
    pub const TOTAL: &'static [u8] = b"/gov/polls/total";
    pub const CREATOR: &'static [u8] = b"/gov/poll/creator/";
    pub const EXPIRATION: &'static [u8] = b"/gov/poll/expiration";
    pub const STATUS: &'static [u8] = b"/gov/poll/status";
    pub const REVEAL_APPROVALS: &'static [u8] = b"/gov/poll/reveal_approvals";
}

pub trait IPoll<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn create_id(core: &mut C) -> StdResult<u64>;

    fn store(&self, core: &mut C) -> StdResult<()>;
    fn get(core: &C, poll_id: u64) -> StdResult<Self>;

    fn creator(core: &C, poll_id: u64) -> StdResult<CanonicalAddr>;

    fn metadata(core: &C, poll_id: u64) -> StdResult<PollMetadata>;
    fn expiration(core: &C, poll_id: u64) -> StdResult<Expiration>;
    fn status(core: &C, poll_id: u64) -> StdResult<PollStatus>;

    fn commit_status(&self, core: &mut C) -> StdResult<()>;

    fn approve_reveal(core: &mut C, poll_id:u64, sender: &HumanAddr) -> StdResult<()>;
    fn reveal_approvals( core: &C, poll_id: u64) -> StdResult<Vec<HumanAddr>>;
    fn commit_approvals(core: &mut C, poll_id: u64, approvals: &Vec<HumanAddr>) -> StdResult<()>;
}

impl<S, A, Q, C> IPoll<S, A, Q, C> for Poll
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn create_id(core: &mut C) -> StdResult<u64> {

        let total = core
            .get::<u64>(Self::TOTAL)?
            .ok_or(StdError::generic_err("can't find total id count"))?;
        let total = total
            .checked_add(1)
            .ok_or(StdError::generic_err("total integer overflow"))?;

        core.set(Self::TOTAL, total)?;

        Ok(total)
    }

    fn store(&self, core: &mut C) -> StdResult<()> {
        let Poll {
            creator,
            expiration,
            id,
            metadata,
            status,
            reveal_approvals,
        } = self;

        core.set_ns(Self::CREATOR, &self.id.to_be_bytes(), creator)?;
        core.set_ns(Self::EXPIRATION, &self.id.to_be_bytes(), expiration)?;
        core.set_ns(Self::STATUS, &self.id.to_be_bytes(), status)?;
        Self::commit_approvals(core, self.id, reveal_approvals)?;
        
        metadata.store(core, id.clone())?;

        Ok(())
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        let creator = Self::creator(core, poll_id)?;
        let expiration = Self::expiration(core, poll_id)?;
        let status = Self::status(core, poll_id)?;
        let metadata = Self::metadata(core, poll_id)?;
        let reveal_approvals = Self::reveal_approvals(core, poll_id)?;
        Ok(Self {
            id: poll_id,
            creator,
            expiration,
            metadata,
            status,
            reveal_approvals,
        })
    }

    fn creator(core: &C, poll_id: u64) -> StdResult<CanonicalAddr> {
        Ok(core
            .get_ns::<CanonicalAddr>(Self::CREATOR, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll creator"))?)
    }
    fn metadata(core: &C, poll_id: u64) -> StdResult<PollMetadata> {
        PollMetadata::get(core, poll_id)
    }

    fn expiration(core: &C, poll_id: u64) -> StdResult<Expiration> {
        Ok(core
            .get_ns::<Expiration>(Self::EXPIRATION, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll expiration"))?)
    }

    fn status(core: &C, poll_id: u64) -> StdResult<PollStatus> {
        Ok(core
            .get_ns::<PollStatus>(Self::STATUS, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll expiration"))?)
    }

    fn commit_status(&self, core: &mut C) -> StdResult<()> {
        core.set_ns(Self::STATUS, &self.id.to_be_bytes(), self.status.clone())?;

        Ok(())
    }


    fn approve_reveal(core: &mut C, poll_id:u64, sender: &HumanAddr) -> StdResult<()> {
        let mut current_approvals = Self::reveal_approvals(core, poll_id)?;
        if current_approvals.iter().any(|addr| addr == sender) {
            return Err(StdError::generic_err("Member already approved"));
        }
        current_approvals.insert(0, sender.clone());
        Self::commit_approvals(core, poll_id, &current_approvals)?;
        Ok(())
    }

    fn commit_approvals(core: &mut C, poll_id: u64, reveal_approvals: &Vec<HumanAddr>) -> StdResult<()> {
        let cannonized_approvals: Vec<CanonicalAddr> = reveal_approvals
            .iter()
            .map(|addr| addr.canonize(core.api()).unwrap() )
            .collect();  
        core.set_ns(Self::REVEAL_APPROVALS, &poll_id.to_be_bytes(), &cannonized_approvals)?;
        Ok(())
    }

    fn reveal_approvals( core: &C, poll_id: u64) -> StdResult<Vec<HumanAddr>> {
        let approvals: Vec<CanonicalAddr> = core.get_ns(Self::REVEAL_APPROVALS, &poll_id.to_be_bytes())?.expect("Error reading the storage");
        let approval_addresses: Vec<HumanAddr> = approvals
            .iter()
            .map(|addr| addr.humanize(core.api()).unwrap() )
            .collect();  
        Ok(approval_addresses)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Active,
    Failed,
    Passed,
}
