use fadroma::*;
use crate::{*, Auth};

pub trait Drain <S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
    + Rewards<S, A, Q>
{
    fn drain (
        &mut self,
        env:       Env,
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    ) -> StdResult<HandleResponse> {
        Auth::assert_admin(self, &env)?;
        let recipient = recipient.unwrap_or(env.message.sender.clone());
        // Update the viewing key if the supplied
        // token info for is the reward token
        let reward_token = RewardsConfig::reward_token(self)?;
        if reward_token.link == snip20 {
            self.set(RewardsConfig::REWARD_VK, key.clone())?
        }
        let allowance = Uint128(u128::MAX);
        let duration  = Some(env.block.time + DAY * 10000);
        let snip20    = ISnip20::attach(snip20);
        HandleResponse::default()
            .msg(snip20.increase_allowance(&recipient, allowance, duration)?)?
            .msg(snip20.set_viewing_key(&key)?)
    }
}
