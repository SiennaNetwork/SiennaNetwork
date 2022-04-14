use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::time_utils::{Duration, Moment};

use super::poll::Poll;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Structure containing all the configuration for governance.
/// Set as Option in order to handle partial updates. Can never be None
/// Default values are:
///     - threshold = 35000
///     - quorum = 0.3 (33%)
///     - deadline = 7 * 24 * 60 * 60 (7 days)
pub struct GovernanceConfig {
    /// Minimum amount of tokens staked needed to create a poll
    pub threshold: Option<Uint128>,
    /// Value from 0 to 1, stands for the minimum % needed for the poll to be valid
    pub quorum: Option<Decimal>,
    /// The time polls last, in seconds
    pub deadline: Option<Duration>,
}
impl GovernanceConfig {
    /// Constant values used for validating metadata
    pub const MIN_TITLE_LENGTH: usize = 8;
    pub const MAX_TITLE_LENGTH: usize = 64;

    pub const MIN_DESC_LENGTH: usize = 8;
    pub const MAX_DESC_LENGTH: usize = 1024;

    pub const MIN_POLL_TYPE_LENGTH: usize = 8;
    pub const MAX_POLL_TYPE_LENGTH: usize = 24;

    /// Constant values used as default for config
    pub const DEFAULT_QUORUM_PERCENT: u64 = 33;
    pub const DEFAULT_TRESHOLD: Uint128 = Uint128(3500);
    pub const DEFAULT_DEADLINE: u64 = 7 * 24 * 60 * 60;

    pub const MIN_STAKED_FOR_VOTE: Uint128 = Uint128(1);

    /// Keys where config data is stored
    pub const THRESHOLD: &'static [u8] = b"/gov/threshold";
    pub const QUORUM: &'static [u8] = b"/gov/quorum";
    pub const DEADLINE: &'static [u8] = b"/gov/deadline";

    pub const CLOSED: &'static [u8] = b"/gov/closed";
}
pub trait IGovernanceConfig<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    /// Commit initial contract configuration to storage.
    fn initialize(&mut self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    fn get(core: &C) -> StdResult<Self>;
    fn threshold(core: &C) -> StdResult<Uint128>;
    fn quorum(core: &C) -> StdResult<Decimal>;
    fn deadline(core: &C) -> StdResult<u64>;
}
impl<S, A, Q, C> IGovernanceConfig<S, A, Q, C> for GovernanceConfig
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn initialize(&mut self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        core.set(Poll::TOTAL, 0)?;
        self.store(core)
    }
    fn get(core: &C) -> StdResult<Self> {
        Ok(Self {
            deadline: Some(Self::deadline(core)?),
            quorum: Some(Self::quorum(core)?),
            threshold: Some(Self::threshold(core)?),
        })
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

    fn threshold(core: &C) -> StdResult<Uint128> {
        Ok(core.get::<Uint128>(Self::THRESHOLD)?.unwrap())
    }

    fn quorum(core: &C) -> StdResult<Decimal> {
        Ok(core.get::<Decimal>(Self::QUORUM)?.unwrap())
    }

    fn deadline(core: &C) -> StdResult<u64> {
        Ok(core.get::<u64>(Self::DEADLINE)?.unwrap())
    }
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            threshold: Some(Self::DEFAULT_TRESHOLD),
            quorum: Some(Decimal::percent(Self::DEFAULT_QUORUM_PERCENT)),
            deadline: Some(Self::DEFAULT_DEADLINE),
        }
    }
}

pub type CloseSeal = (Moment, String);
