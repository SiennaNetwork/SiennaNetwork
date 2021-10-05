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

use crate::data::{
    load_account, load_all_accounts, load_contract_address, load_viewing_key, AccounTokenEntry,
    Config, TokenConfig,
};
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
        })
        .collect::<Vec<QueryAccountToken>>();

    to_binary(&QueryResponse::UserInfo(tokens))
}

/// Query that will return the list of addresses, this won't mark the addresses as drawn
/// but can be used without a cost and it won't be used in an IDO.
pub(crate) fn draw_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    tokens: Vec<Option<HumanAddr>>,
    number: u32,
    timestamp: u64,
) -> StdResult<Binary> {
    let config = Config::load_self(deps)?;

    let mut token_configs: Vec<TokenConfig> = vec![];
    for token in &tokens {
        token_configs.push(config.get_token_config(token.clone())?);
    }

    let accounts = load_all_accounts(deps)?;
    let mut entries: Vec<(HumanAddr, AccounTokenEntry)> = vec![];

    for account in &accounts {
        let account_entries = account.get_entries(&token_configs, timestamp);
        for account_entry in account_entries {
            entries.push(account_entry);
        }
    }
    // Sort entries based on the timestamp they were locked,
    // this can be used as a weighted rand select where we will use biased
    // random number generation when picking entries.
    // Bias can be towards to begining of the list making the entries
    // locked longer more likely to be drawn.
    entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut addresses: Vec<HumanAddr> = vec![];

    // Run the loop while we don't fill the whitelist with addresses
    // or while we don't run out of entries to pick from
    while addresses.len() < number as usize && entries.len() > 0 {
        let index: usize = gen_rand_range(0, (entries.len() - 1) as u64, timestamp) as usize;

        match &entries.get(index as usize) {
            Some((address, _)) => {
                let addr = address.clone();
                // After the address is picked, we will remove it from the list of entries
                // so we make sure we are creating a whitelist of unique addresses, thats
                // why we are cloning it above.
                entries = entries.into_iter().filter(|(a, _)| a != &addr).collect();
                addresses.push(addr);
            }
            None => (),
        };
    }

    to_binary(&QueryResponse::DrawnAddresses(addresses))
}
