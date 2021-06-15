use serde::{ Deserialize, Serialize };
use cosmwasm_std::{ 
    StdResult, Extern, Storage, Querier, Api, Uint128, CanonicalAddr,
    HumanAddr, StdError
};
use amm_shared::fadroma::callback::{ContractInstance, Callback};
use amm_shared::fadroma::address::{Canonize, Humanize};
use amm_shared::fadroma::storage::{save, load};
use amm_shared::TokenType;

pub(crate) static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config<A> {
    /// The token that is used to buy the instantiated SNIP20
    pub input_token: TokenType<A>,
    /// The token that this contract swaps to and instantiates
    pub swap_token: ContractInstance<A>,
    pub swap_constants: SwapConstants,
    pub callback: Option<Callback<A>>
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
/// Used when calculating the swap. These do not change
/// throughout the lifetime of the contract.
pub(crate) struct SwapConstants {
    /// The amount needed to represent 1 whole swap_token
    pub whole_swap_token: Uint128,
    pub rate: Uint128,
    pub input_token_decimals: u8,
    pub swap_token_decimals: u8
}

pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Config<HumanAddr>> {
    let result: Option<Config<CanonicalAddr>> = load(&deps.storage, CONFIG_KEY)?;
    result.ok_or(StdError::generic_err("Config doesn't exist in storage."))?.humanize(&deps.api)
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config{
            input_token: self.input_token.canonize(api)?,
            swap_token: self.swap_token.canonize(api)?,
            swap_constants: self.swap_constants.clone(),
            callback: if let Some(c) = self.callback.clone() {
                Some(c.canonize(api)?)
            } else {
                None
            }
        })
    }
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config{
            input_token: self.input_token.humanize(api)?,
            swap_token: self.swap_token.humanize(api)?,
            swap_constants: self.swap_constants.clone(),
            callback: if let Some(c) = self.callback.clone() {
                Some(c.humanize(api)?)
            } else {
                None
            }
        })
    }
}
