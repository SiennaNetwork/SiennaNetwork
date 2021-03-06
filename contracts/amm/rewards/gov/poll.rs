use amm_shared::Sender;
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    time_utils::Moment,
    total::{ITotal, Total},
};

use super::{
    expiration::Expiration,
    poll_metadata::{IPollMetaData, PollMetadata},
    poll_result::{IPollResult, PollResult},
    vote::VoteType,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Poll {
    /// Unique, auto increment id for the poll
    pub id: u64,
    /// Creator of the poll, shown as HumanAddr but stored as CanonicalAddr
    pub creator: HumanAddr,
    /// Meta information about the poll, set as user input
    pub metadata: PollMetadata,
    /// When the poll expires.
    /// Set as current_time + deadline
    pub expiration: Expiration,
    /// Up to date status of the poll
    pub status: PollStatus,
    /// The quroum upon time of creation
    pub current_quorum: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
/// Helper data structure used for returning the full information about a given poll
pub struct PollInfo {
    pub instance: Poll,
    pub result: PollResult,
}

impl Poll {
    /// Keys used for storing the data
    pub const TOTAL: &'static [u8] = b"/gov/polls/total";
    pub const CREATOR: &'static [u8] = b"/gov/poll/creator/";
    pub const EXPIRATION: &'static [u8] = b"/gov/poll/expiration";
    pub const STATUS: &'static [u8] = b"/gov/poll/status";
    pub const CURRENT_QUORUM: &'static [u8] = b"/gov/poll/current_quorum";
}

pub trait IPoll<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn create_id(core: &mut C) -> StdResult<u64>;

    fn new(
        core: &mut C,
        creator: &Sender,
        expiration: Expiration,
        metadata: PollMetadata,
        current_quorum: Decimal,
    ) -> StdResult<Self>;

    fn store(&self, core: &mut C) -> StdResult<()>;
    fn get(core: &C, poll_id: u64, now: Moment) -> StdResult<Self>;

    fn creator(core: &C, poll_id: u64) -> StdResult<Sender>;
    fn metadata(core: &C, poll_id: u64) -> StdResult<PollMetadata>;
    fn expiration(core: &C, poll_id: u64) -> StdResult<Expiration>;
    fn status(core: &C, poll_id: u64) -> StdResult<PollStatus>;
    fn current_quorum(core: &C, poll_id: u64) -> StdResult<Decimal>;
    fn count(core: &C) -> StdResult<u64>;

    fn commit_status(core: &mut C, poll_id: u64, status: PollStatus) -> StdResult<()>;
    fn update_result(
        core: &mut C,
        poll_id: u64,
        now: Moment,
        update: UpdateResultReason,
    ) -> StdResult<PollResult>;
}

impl<S, A, Q, C> IPoll<S, A, Q, C> for Poll
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    /// Handles the auto increment ID
    fn create_id(core: &mut C) -> StdResult<u64> {
        let total = Self::count(core)?;
        let total = total.checked_add(1).unwrap();

        core.set(Self::TOTAL, total)?;

        Ok(total)
    }
    /// Returns the up to date auto increment ID. Can also stand for the total amount of polls created
    fn count(core: &C) -> StdResult<u64> {
        Ok(core.get::<u64>(Self::TOTAL)?.unwrap())
    }

    fn store(&self, core: &mut C) -> StdResult<()> {
        let Poll {
            creator,
            expiration,
            id,
            metadata,
            status,
            current_quorum,
        } = self;

        core.set_ns(
            Self::CREATOR,
            &self.id.to_be_bytes(),
            creator.canonize(core.api())?,
        )?;
        core.set_ns(Self::EXPIRATION, &self.id.to_be_bytes(), expiration)?;
        core.set_ns(Self::STATUS, &self.id.to_be_bytes(), status)?;
        core.set_ns(Self::CURRENT_QUORUM, &self.id.to_be_bytes(), current_quorum)?;

        metadata.store(core, *id)?;

        Ok(())
    }

    fn get(core: &C, poll_id: u64, now: Moment) -> StdResult<Self> {
        let creator = Self::creator(core, poll_id)?.human;
        let expiration = Self::expiration(core, poll_id)?;
        let mut status = Self::status(core, poll_id)?;
        if !expiration.is_expired(now) {
            status = PollStatus::Active;
        }
        let metadata = Self::metadata(core, poll_id)?;
        let current_quorum = Self::current_quorum(core, poll_id)?;
        Ok(Self {
            id: poll_id,
            creator,
            expiration,
            metadata,
            status,
            current_quorum,
        })
    }

    fn creator(core: &C, poll_id: u64) -> StdResult<Sender> {
        let canonical_addr = core
            .get_ns::<CanonicalAddr>(Self::CREATOR, &poll_id.to_be_bytes())?
            .unwrap();
        Ok(Sender::from_canonical(&canonical_addr, core.api())?)
    }
    fn metadata(core: &C, poll_id: u64) -> StdResult<PollMetadata> {
        PollMetadata::get(core, poll_id)
    }

    fn expiration(core: &C, poll_id: u64) -> StdResult<Expiration> {
        Ok(core
            .get_ns::<Expiration>(Self::EXPIRATION, &poll_id.to_be_bytes())?
            .unwrap())
    }

    fn status(core: &C, poll_id: u64) -> StdResult<PollStatus> {
        Ok(core
            .get_ns::<PollStatus>(Self::STATUS, &poll_id.to_be_bytes())?
            .unwrap())
    }

    fn commit_status(core: &mut C, poll_id: u64, status: PollStatus) -> StdResult<()> {
        core.set_ns(Self::STATUS, &poll_id.to_be_bytes(), status)?;
        Ok(())
    }

    fn current_quorum(core: &C, poll_id: u64) -> StdResult<Decimal> {
        Ok(core
            .get_ns::<Decimal>(Self::CURRENT_QUORUM, &poll_id.to_be_bytes())?
            .unwrap())
    }

    fn new(
        core: &mut C,
        creator: &Sender,
        expiration: Expiration,
        metadata: PollMetadata,
        current_quorum: Decimal,
    ) -> StdResult<Self> {
        let id = Self::create_id(core)?;
        Ok(Self {
            creator: creator.human.clone(),
            id,
            current_quorum,
            expiration,
            metadata,
            status: PollStatus::Failed,
        })
    }

    /// Main entry point for any update on a poll.
    /// Uses the update enum to define all possible updates on a poll
    fn update_result(
        core: &mut C,
        poll_id: u64,
        now: Moment,
        update: UpdateResultReason,
    ) -> StdResult<PollResult> {
        let mut result = PollResult::get(core, poll_id)?;

        match update {
            UpdateResultReason::ChangeVoteChoice { choice, power } => {
                result.transfer_vote(choice, power)?;
            }
            UpdateResultReason::ChangeVotePower { choice, power_diff } => {
                result.change_vote_power(choice, power_diff)?;
            }
        }
        result.store(core)?;

        let total = Total::from_time(core, now)?;
        let current_quorum = Poll::current_quorum(core, poll_id)?;
        let participation = Decimal::from_ratio(result.total(), total.staked.u128());
        let is_quorum_met = participation > current_quorum;

        if is_quorum_met && result.yes_votes > result.no_votes {
            Poll::commit_status(core, poll_id, PollStatus::Passed)?;
        } else {
            Poll::commit_status(core, poll_id, PollStatus::Failed)?;
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Active,
    Failed,
    Passed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UpdateResultReason {
    ChangeVotePower { choice: VoteType, power_diff: i128 },
    ChangeVoteChoice { choice: VoteType, power: u128 },
}
