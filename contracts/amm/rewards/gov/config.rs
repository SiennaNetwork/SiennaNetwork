use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use fadroma::*;

use super::governance::Governance;




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GovernanceConfig {
    pub threshold: Option<u8>,
    pub quorum: Option<u8>,
    pub deadline: Option<u64>,
}
impl GovernanceConfig {
    //metadata configuration
    pub const MIN_TITLE_LENGTH: usize = 8;
    pub const MAX_TITLE_LENGTH: usize = 64;

    pub const MIN_DESC_LENGTH: usize = 8;
    pub const MAX_DESC_LENGTH: usize = 1024;

    //storage keys
    pub const THRESHOLD: &'static [u8] = b"/gov/threshold";
    pub const QUORUM: &'static [u8] = b"/gov/quorum";
    pub const DEADLINE: &'static [u8] = b"/gov/deadline";
    pub const VOTES: &'static [u8] = b"/gov/votes";
}
pub trait IGovernanceConfig<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    /// Commit initial contract configuration to storage.
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    fn threshold(core: &C) -> StdResult<u8>;
    fn quorum(core: &C) -> StdResult<u8>;
    fn deadline(core: &C) -> StdResult<u64>;
}
impl<S, A, Q, C> IGovernanceConfig<S, A, Q, C> for GovernanceConfig
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        Ok(vec![])
    }

    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        let GovernanceConfig {
            deadline,
            threshold,
            quorum,
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
        Ok(vec![])
    }

    fn threshold(core: &C) -> StdResult<u8> {
        core.get::<u8>(Self::THRESHOLD)?
            .ok_or(StdError::generic_err("threshold not set"))
    }

    fn quorum(core: &C) -> StdResult<u8> {
        core.get::<u8>(Self::QUORUM)?
            .ok_or(StdError::generic_err("quorum not set"))
    }

    fn deadline(core: &C) -> StdResult<u64> {
        core.get::<u64>(Self::THRESHOLD)?
            .ok_or(StdError::generic_err("deadline not set"))
    }
}
