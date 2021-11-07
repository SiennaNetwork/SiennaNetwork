use fadroma::*;
use crate::{*, Auth};

pub trait KeplrCompat <S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    fn token_info (&self) -> StdResult<Response> {
        let link = self.humanize(
            self.get(b"/lp_token")?.ok_or(StdError::generic_err("no lp token"))?
        )?;
        let info = ISnip20::attach(link).query_token_info(self.querier())?;
        Ok(Response::TokenInfo {
            name:         format!("Sienna Rewards: {}", info.name),
            symbol:       "SRW".into(),
            decimals:     1,
            total_supply: None
        })
    }

    fn balance (&self, address: HumanAddr, key: ViewingKey) -> StdResult<Response> {
        let id = self.canonize(address)?;
        Auth::check_vk(self, &key, id.as_slice())?;
        Ok(Response::Balance {
            amount: self.get_ns(crate::algo::Account::STAKED, id.as_slice())?.unwrap_or(Amount::zero())
        })
    }

}
