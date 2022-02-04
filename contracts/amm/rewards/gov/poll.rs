use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use fadroma::*;

use super::{poll_metadata::{PollMetadata, IPollMetaData}, expiration::Expiration, governance::Governance};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub metadata: PollMetadata,
    pub expiration: Expiration,
    pub status: PollStatus,
}

impl Poll {
    pub const TOTAL: &'static [u8] = b"/gov/polls/total";

    pub const CREATOR: &'static [u8] = b"/gov/poll/creator/";
    pub const EXPIRATION: &'static [u8] = b"/gov/poll/expiration";
    pub const STATUS: &'static [u8] = b"/gov/poll/status";
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
        } = self;

        core.set_ns(Self::CREATOR, &self.id.to_be_bytes(), creator)?;
        core.set_ns(Self::EXPIRATION, &self.id.to_be_bytes(), expiration)?;
        core.set_ns(Self::STATUS, &self.id.to_be_bytes(), status)?;
        metadata.store(core, *id)?;

        Ok(())
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        let creator = Self::creator(core, poll_id)?;
        let expiration = Self::expiration(core, poll_id)?;
        let status = Self::status(core, poll_id)?;
        let metadata = Self::metadata(core, poll_id)?;
        Ok(Self {
            id: poll_id,
            creator,
            expiration,
            metadata,
            status,
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
}




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Active,
    Failed,
    Passed,
}

