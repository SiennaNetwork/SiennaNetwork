
use fadroma::*;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use super::{config::GovernanceConfig, governance::Governance, poll::{Poll, IPoll}};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceResponse {
    Polls {},
    Poll(Poll),
    Config(GovernanceConfig),
}
pub trait IGovernanceResponse<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(core: &C) -> StdResult<Self>;
    fn poll(core: &C, id: u64) -> StdResult<Self>;
}
impl<S, A, Q, C> IGovernanceResponse<S, A, Q, C> for GovernanceResponse
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(_core: &C) -> StdResult<Self> {
        Ok(GovernanceResponse::Polls {})
    }
    fn poll(core: &C, id: u64) -> StdResult<GovernanceResponse> {
        let meta = Poll::metadata(core, id)?;

        let poll = Poll {
            creator: Poll::creator(core, id)?,
            id,
            metadata: meta,
            expiration: Poll::expiration(core, id)?,
            status: Poll::status(core, id)?,
        };
        Ok(GovernanceResponse::Poll(poll))
    }
}
