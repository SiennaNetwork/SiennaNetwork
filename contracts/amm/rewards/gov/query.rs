use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    governance::Governance,
    response::{GovernanceResponse, IGovernanceResponse},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceQuery {
    Polls {
        // TODO: add pagination
    // take: u16,
    // page: u16
    },
    Poll {
        id: u64,
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
            GovernanceQuery::Polls {} => GovernanceResponse::polls(core),
            GovernanceQuery::Poll { id } => GovernanceResponse::poll(core, id),
            GovernanceQuery::Config {} => GovernanceResponse::config(core),
        }
    }
}
