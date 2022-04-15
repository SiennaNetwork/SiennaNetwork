use fadroma::ISnip20;

use crate::config::IRewardsConfig;
use crate::{
    time_utils::{Duration, DAY},
    Auth, *,
};

/// Number of seconds to wait after closing pool
/// before it is possible to drain remaining LP or reward tokens.
pub const WAIT_PERIOD: Duration = 604800;

pub trait Drain<S: Storage, A: Api, Q: Querier>:
    Composable<S, A, Q> + Auth<S, A, Q> + Rewards<S, A, Q>
{
    /// Give access (maximum allowance + specified viewing key)
    /// over this contract's holdings in a particular SNIP20 token
    /// to a particular user. Intended for emergency recovery only.
    /// Can be called no sooner than `WAIT_PERIOD` seconds after
    /// permantntly closing the pool.
    fn drain(
        &mut self,
        env: Env,
        snip20: ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key: String,
    ) -> StdResult<HandleResponse> {
        // If not the admin, can't drain.
        Auth::assert_admin(self, &env)?;

        // If closed for less than the wait period, can't drain yet.
        if RewardsConfig::assert_closed(self, &env)? < WAIT_PERIOD {
            return Err(StdError::unauthorized());
        }

        // If no recipient, default to admin.
        let recipient = recipient.unwrap_or_else(|| env.message.sender.clone());

        // If draining reward token, update the stored reward VK.
        if RewardsConfig::reward_token(self)?.link == snip20 {
            self.set(RewardsConfig::REWARD_VK, key.clone())?
        }

        // Call methods of drained token.
        let allowance = Uint128(u128::MAX);
        let duration = Some(env.block.time + DAY * 10000);
        let snip20 = ISnip20::attach(snip20);
        HandleResponse::default()
            .msg(snip20.increase_allowance(&recipient, allowance, duration)?)?
            .msg(snip20.set_viewing_key(&key)?)
    }
}
