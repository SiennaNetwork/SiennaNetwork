use amm_shared::{
    auth::authenticate,
    fadroma::scrt::{
        addr::Canonize,
        callback::ContractInstance,
        cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage},
        utils::viewing_key::ViewingKey,
    },
    msg::launchpad::{QueryAccountToken, QueryResponse, QueryTokenConfig},
    TokenType,
};

use crate::data::{load_account, load_contract_address, load_viewing_key, Config};
use crate::helpers::*;

/// Display the configured tokens to be accepted into the launchpad and their terms
pub fn launchpad_info<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = Config::load_self(&deps)?;
    let address = load_contract_address(deps)?;
    let viewing_key = load_viewing_key(&deps.storage)?;

    let mut tokens = vec![];

    for token in config.tokens.into_iter() {
        let locked_balance = match &token.token_type {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => get_token_balance(
                deps,
                address.clone(),
                ContractInstance {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
                viewing_key.clone(),
            )?,
            TokenType::NativeToken { denom } => {
                deps.querier.query_balance(address.clone(), &denom)?.amount
            }
        };

        tokens.push(QueryTokenConfig {
            token_type: token.token_type.clone(),
            segment: token.segment,
            bounding_period: token.bounding_period,
            active: token.active,
            token_decimals: token.token_decimals,
            locked_balance,
        });
    }

    to_binary(&QueryResponse::LaunchpadInfo(tokens))
}

/// Query for users information about locked tokens and current balance
pub fn user_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    key: String,
) -> StdResult<Binary> {
    let canonical_address = address.canonize(&deps.api)?;

    authenticate(
        &deps.storage,
        &ViewingKey(key),
        canonical_address.as_slice(),
    )?;

    let account = load_account(deps, &address)?;

    let tokens = account
        .tokens
        .into_iter()
        .map(|t| QueryAccountToken {
            token_type: t.token_type,
            balance: t.balance,
            entries: t.entries,
            last_draw: t.last_draw,
        })
        .collect::<Vec<QueryAccountToken>>();

    to_binary(&QueryResponse::UserInfo(tokens))
}
