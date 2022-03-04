use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PollMetadata {
    pub title: String,
    pub description: String,
    pub poll_type: String,
}

impl PollMetadata {
    pub const SELF: &'static [u8] = b"/poll/meta/";
}
pub trait IPollMetaData<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn store(&self, core: &mut C, poll_id: u64) -> StdResult<()>;
    fn get(core: &C, poll_id: u64) -> StdResult<Self>;
}
impl<S, A, Q, C> IPollMetaData<S, A, Q, C> for PollMetadata
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn store(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        core.set_ns(Self::SELF, &poll_id.to_be_bytes(), &self)?;

        Ok(())
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        Ok(core.get_ns(Self::SELF, &poll_id.to_be_bytes())?.unwrap())
    }
}
