use std::cmp::min;

use amm_shared::Sender;
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::time_utils::Moment;

use super::{
    config::{GovernanceConfig, IGovernanceConfig},
    governance::Governance,
    poll::{IPoll, Poll, PollInfo},
    poll_result::{IPollResult, PollResult},
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceResponse {
    Polls {
        polls: Vec<Poll>,
        total: u64,
        total_pages: u64,
    },
    Poll(PollInfo),
    VoteStatus {
        choice: VoteType,
        power: Uint128,
    },

    Config(GovernanceConfig),
}
pub trait IGovernanceResponse<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(core: &C, take: u64, page: u64, asc: bool, now: Moment) -> StdResult<Self>;
    fn poll(core: &C, id: u64, now: Moment) -> StdResult<Self>;
    fn vote_status(core: &C, poll_id: u64, snender: &Sender) -> StdResult<Self>;
    fn config(core: &C) -> StdResult<Self>;
}
impl<S, A, Q, C> IGovernanceResponse<S, A, Q, C> for GovernanceResponse
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(core: &C, mut take: u64, page: u64, asc: bool, now: Moment) -> StdResult<Self> {
        let total = Poll::count(core)?;
        let total_pages = (total + take - 1) / take;

        let build_ascended = |take| -> StdResult<Vec<Poll>> {
            let start = (page - 1) * take;
            let end = min(start + take, total);
            let mut polls = vec![];
            for index in start + 1..=end {
                polls.push(Poll::get(core, index, now)?);
            }
            Ok(polls)
        };

        let mut build_descended = || -> StdResult<Vec<Poll>> {
            let end = total.saturating_sub(take * page);
            if end == 0 {
                take = total - take * (page - 1);
            }

            let start = end + take;

            let mut polls = vec![];
            for index in end + 1..=start {
                polls.push(Poll::get(core, index, now)?);
            }
            Ok(polls)
        };

        let polls = if asc {
            build_ascended(take)?
        } else {
            build_descended()?
        };

        Ok(GovernanceResponse::Polls {
            total,
            polls,
            total_pages,
        })
    }
    fn poll(core: &C, id: u64, now: Moment) -> StdResult<GovernanceResponse> {
        let poll = Poll::get(core, id, now)?;
        let poll_res = PollResult::get(core, id)?;
        Ok(GovernanceResponse::Poll(PollInfo {
            instance: poll,
            result: poll_res,
        }))
    }
    fn config(core: &C) -> StdResult<Self> {
        let config = GovernanceConfig::get(core)?;
        Ok(GovernanceResponse::Config(config))
    }

    fn vote_status(core: &C, poll_id: u64, sender: &Sender) -> StdResult<Self> {
        let vote = Vote::get(core, sender, poll_id)?;
        Ok(GovernanceResponse::VoteStatus {
            power: vote.power,
            choice: vote.choice,
        })
    }
}
