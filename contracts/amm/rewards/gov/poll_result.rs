use fadroma::{Api, Composable, Querier, StdError, StdResult, Storage, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::vote::VoteType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PollResult {
    pub poll_id: u64,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
}
impl PollResult {
    pub const SELF: &'static [u8] = b"/gov/result";

    pub fn append_votes(&mut self, amount: i128, choice: VoteType) -> StdResult<()> {
        let try_add = |vote: &mut Uint128, amount: i128| {
            if amount > 0 {
                *vote = Uint128((*vote).u128() + amount as u128);
            } else {
                let new_vote = vote
                    .u128()
                    .checked_sub(amount.abs() as u128)
                    .ok_or(StdError::generic_err("Not enough voting power available"))
                    .unwrap();
                *vote = Uint128(new_vote);
            }
        };

        if let VoteType::Yes = choice {
            try_add(&mut self.yes_votes, amount);
        } else {
            try_add(&mut self.no_votes, amount);
        }
        Ok(())
    }

    pub fn total(&self) -> u128 {
        self.no_votes.u128() + self.yes_votes.u128()
    }

    pub fn change_vote_power(
        &mut self,
        choice: VoteType,
        power_diff: i128,
    ) -> StdResult<&mut Self> {
        self.append_votes(power_diff, choice)?;
        Ok(self)
    }

    pub fn transfer_vote(&mut self, target_choice: VoteType, power: u128) -> StdResult<&mut Self> {
        let power = power as i128;
        if target_choice == VoteType::Yes {
            self.append_votes(power, VoteType::Yes)?;
            self.append_votes(-power, VoteType::No)?;
        } else {
            self.append_votes(-power, VoteType::Yes)?;
            self.append_votes(power, VoteType::No)?;
        }
        Ok(self)
    }
}

pub trait IPollResult<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn new(core: &C, poll_id: u64) -> Self;
    fn store(&self, core: &mut C) -> StdResult<()>;
    fn get(core: &C, poll_id: u64) -> StdResult<Self>;
}

impl<S, A, Q, C> IPollResult<S, A, Q, C> for PollResult
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn store(&self, core: &mut C) -> StdResult<()> {
        core.set_ns(Self::SELF, &self.poll_id.to_be_bytes(), &self)
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        core.get_ns::<Self>(Self::SELF, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll result"))
    }
    fn new(_: &C, poll_id: u64) -> Self {
        Self {
            poll_id,
            no_votes: Uint128(0),
            yes_votes: Uint128(0),
        }
    }
}
