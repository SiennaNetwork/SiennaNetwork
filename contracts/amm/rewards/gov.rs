use crate::{algo::Rewards, auth::Auth};
use core::fmt;
use cosmwasm_std::BlockInfo;
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// feature trait ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub trait Governance<S: Storage, A: Api, Q: Querier>:
    Composable<S, A, Q> // to compose with other modules
    + Auth<S, A, Q>     // to authenticate txs/queries
    + Sized             // to pass mutable self-reference to Total and Account
{
    /// Configure the rewards module
    fn init (&mut self, env: &Env, mut config: GovernanceConfig) -> StdResult<Vec<CosmosMsg>> {
        config.initialize(self, env)
    }
    /// Handle transactions
    fn handle (&mut self, env: Env, msg: GovernanceHandle) -> StdResult<HandleResponse> {
        msg.dispatch_handle(self, env)
    }
    /// Handle queries
    fn query (&self, msg: GovernanceQuery) -> StdResult<GovernanceResponse> {
        msg.dispatch_query(self)
    }
}

// init config api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Governance configuration
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
    pub const QUORUM: &'static [u8] = b"gov/quorum";
    pub const DEADLINE: &'static [u8] = b"gov/deadline";
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
        if let Some(deadine) = deadline {
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

// handle tx api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceHandle {
    CreatePoll { meta: PollMetadata },
    EndPoll { poll_id: u64 },
    Vote { variant: VoteType, poll_id: u64 },
    Unvote { poll_id: u64 },
    ChangeVote { variant: VoteType, poll_id: u64 },
    UpdateConfig { config: GovernanceConfig },
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for GovernanceHandle
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn dispatch_handle(self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        match self {
            GovernanceHandle::CreatePoll { meta } => {
                let poll = Poll {
                    creator: core.canonize(env.message.sender.clone())?,
                    expiration: Expiration::AtTime(30),
                    id: 1,
                    metadata: meta,
                };

                Ok(HandleResponse::default())
            }
            GovernanceHandle::EndPoll { poll_id } => Ok(HandleResponse::default()),
            GovernanceHandle::Vote { variant, poll_id } => Ok(HandleResponse::default()),
            GovernanceHandle::ChangeVote { variant, poll_id } => Ok(HandleResponse::default()),
            GovernanceHandle::Unvote { poll_id } => Ok(HandleResponse::default()),
            _ => {
                Auth::assert_admin(core, &env)?;
                match self {
                    GovernanceHandle::UpdateConfig { config } => Ok(HandleResponse {
                        messages: config.store(core)?,
                        log: vec![],
                        data: None,
                    }),
                    _ => unreachable!(),
                }
            }
        }
    }
}

// query api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceQuery {
    Polls {},
    Poll {},
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
            GovernanceQuery::Poll {} => GovernanceResponse::poll(core),
        }
    }
}

// response api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
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
    fn poll(core: &C) -> StdResult<Self>;
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
    fn poll(core: &C) -> StdResult<GovernanceResponse> {
        Ok(GovernanceResponse::Poll(Poll {
            creator: core.canonize(HumanAddr::from("test"))?,
            id: 1,
            metadata: PollMetadata {
                description: "test".to_string(),
                poll_type: PollType::Other,
                title: "test".to_string(),
            },
            expiration: Expiration::AtTime(42),
        }))
    }
}

// custom logic ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub metadata: PollMetadata,
    pub expiration: Expiration,
}
impl Poll {
    pub const TOTAL: &'static [u8] = b"/polls/total";
    pub const POLLS: &'static [u8] = b"/polls";
}

pub trait IPoll<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn create_id(core: &C) -> StdResult<u64>;
}

impl<S, A, Q, C> IPoll<S, A, Q, C> for Poll
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn create_id(core: &C) -> StdResult<u64> {
        Ok(32)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PollMetadata {
    pub title: String,
    pub description: String,
    pub poll_type: PollType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    SiennaRewards,
    SiennaSwapParameters,
    Other,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Active,
    Failed,
    Passed,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
    Yes,
    No,
}

//Expiration utility

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Expiration {
    AtTime(u64),
}

impl fmt::Display for Expiration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expiration::AtTime(time) => write!(f, "Expiration time {}", time),
        }
    }
}

impl Expiration {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        match self {
            Expiration::AtTime(time) => block.time >= *time,
        }
    }
}
