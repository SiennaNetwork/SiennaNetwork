use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{ StdResult, Extern, Storage, Querier, Api, Uint128};

use shared::{ContractInfo, ContractInfoStored, TokenType, TokenTypeStored};
use utils::storage::{load, save};

pub(crate) static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub(crate) struct Config {
    /// The token that is used to buy the instantiated SNIP20
    pub input_token: TokenType,
    pub rate: Uint128,
    /// The token that this contract swaps to and instantiates
    pub swapped_token: ContractInfo
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigStored {
    pub input_token: TokenTypeStored,
    pub rate: Uint128,
    pub swapped_token: ContractInfoStored
}

pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, config: &Config) -> StdResult<()> {
    let config = ConfigStored {
        input_token: config.input_token.to_stored(&deps.api)?,
        rate: config.rate,
        swapped_token: config.swapped_token.to_stored(&deps.api)?,
    };

    save(&mut deps.storage, CONFIG_KEY, &config)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Config> {
    let result: ConfigStored = load(&deps.storage, CONFIG_KEY)?;

    Ok(Config {
        input_token: result.input_token.to_normal(&deps.api)?,
        rate: result.rate,
        swapped_token: result.swapped_token.to_normal(&deps.api)?,
    })
}
