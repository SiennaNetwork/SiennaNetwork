use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    governance::Governance,
    response::{GovernanceResponse, IGovernanceResponse},
};
use crate::auth::Auth;
use crate::time_utils::Moment;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceQuery {
    Polls {
        take: u64,
        page: u64,
        asc: bool,
        now: Moment,
    },
    Poll {
        id: u64,
        now: Moment,
    },
    VoteStatus {
        poll_id: u64,
        address: HumanAddr,
        key: String,
    },
    Config {},
}
impl<S, A, Q, C> QueryDispatch<S, A, Q, C, GovernanceResponse> for GovernanceQuery
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn dispatch_query(self, core: &C) -> StdResult<GovernanceResponse> {
        match self {
            GovernanceQuery::Polls {
                take,
                page,
                now,
                asc,
            } => GovernanceResponse::polls(core, take, page, asc, now),
            GovernanceQuery::Poll { id, now } => GovernanceResponse::poll(core, id, now),
            GovernanceQuery::Config {} => GovernanceResponse::config(core),
            GovernanceQuery::VoteStatus {
                poll_id,
                key,
                address,
            } => {
                Auth::check_vk(core, &address, &key.into())?;
                GovernanceResponse::vote_status(core, poll_id, address)
            }
        }
    }
}
