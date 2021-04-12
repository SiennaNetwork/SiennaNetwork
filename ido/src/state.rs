use serde::{Deserialize, Serialize};
use cosmwasm_std::{ StdResult, Extern, Storage, Querier, Api, Uint128};

use shared::{TokenType, TokenTypeStored};
use cosmwasm_utils::{ContractInfo, ContractInfoStored, Callback};
use cosmwasm_utils::storage::{load, save};

pub(crate) static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    /// The token that is used to buy the instantiated SNIP20
    pub input_token: TokenType,
    /// The token that this contract swaps to and instantiates
    pub swap_token: ContractInfo,
    pub swap_constants: SwapConstants,
    pub callback: Option<Callback>
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
/// Used when calculating the swap. These do not change
/// throughout the lifetime of the contract.
pub(crate) struct SwapConstants {
    /// The amount needed to represent 1 whole swap_token
    pub whole_swap_token: Uint128,
    pub rate: Uint128,
    pub input_token_decimals: u8,
    pub swap_token_decimals: u8
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigStored {
    pub input_token: TokenTypeStored,
    pub swap_token: ContractInfoStored,
    pub swap_constants: SwapConstants,
    pub callback: Option<Callback>
}

pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, config: &Config) -> StdResult<()> {
    let config = ConfigStored {
        input_token: config.input_token.to_stored(&deps.api)?,
        swap_token: config.swap_token.to_stored(&deps.api)?,
        swap_constants: SwapConstants {
            whole_swap_token: config.swap_constants.whole_swap_token,
            rate: config.swap_constants.rate,
            input_token_decimals: config.swap_constants.input_token_decimals,
            swap_token_decimals: config.swap_constants.swap_token_decimals
        },
        callback: config.callback.clone()
    };

    save(&mut deps.storage, CONFIG_KEY, &config)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Config> {
    let result: ConfigStored = load(&deps.storage, CONFIG_KEY)?;

    Ok(Config {
        input_token: result.input_token.to_normal(&deps.api)?,
        swap_token: result.swap_token.to_normal(&deps.api)?,
        swap_constants: SwapConstants {
            whole_swap_token: result.swap_constants.whole_swap_token,
            rate: result.swap_constants.rate,
            input_token_decimals: result.swap_constants.input_token_decimals,
            swap_token_decimals: result.swap_constants.swap_token_decimals
        },
        callback: result.callback
    })
}
