use amm_shared::{
    auth::authenticate,
    fadroma::scrt::{
        addr::Canonize,
        cosmwasm_std::{
            to_binary, Api, CanonicalAddr, Extern, HumanAddr, Querier, QueryResult, Storage,
        },
        toolkit::snip20,
        utils::viewing_key::ViewingKey,
        BLOCK_SIZE,
    },
    msg::ido::QueryResponse,
};

use crate::data::{
    load_contract_address, load_total_pre_lock_amount, load_viewing_key, Account, Config,
};
use std::ops::Sub;

/// Check if the address is eligible to participate in the sale
pub(crate) fn get_eligibility_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> QueryResult {
    let can_participate = Account::load_self(deps, &address).is_ok();

    to_binary(&QueryResponse::Eligibility { can_participate })
}

/// Return info about the token sale
pub(crate) fn get_sale_info<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::<HumanAddr>::load_self(&deps)?;

    let (start, end) = if let Some(schedule) = config.schedule {
        (Some(schedule.start), Some(schedule.end))
    } else {
        (None, None)
    };

    to_binary(&QueryResponse::SaleInfo {
        input_token: config.input_token,
        sold_token: config.sold_token,
        rate: config.swap_constants.rate,
        taken_seats: config.taken_seats,
        max_seats: config.max_seats,
        max_allocation: config.max_allocation,
        min_allocation: config.min_allocation,
        end,
        start,
    })
}

/// Get information about the ongoing sale and its status
pub(crate) fn get_sale_status<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> QueryResult {
    let config = Config::<HumanAddr>::load_self(&deps)?;

    let mut available_for_sale = crate::helpers::get_token_balance(
        &deps,
        load_contract_address(deps)?,
        config.sold_token.clone(),
        load_viewing_key(&deps.storage)?,
    )?;

    let sold_in_pre_lock = load_total_pre_lock_amount(&deps)?;

    available_for_sale = available_for_sale.sub(sold_in_pre_lock)?;

    to_binary(&QueryResponse::Status {
        total_allocation: config.total_allocation(),
        available_for_sale,
        sold_in_pre_lock,
        is_active: config.is_active(),
    })
}

pub(crate) fn get_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    key: String,
) -> QueryResult {
    let canonical = address.canonize(&deps.api)?;
    authenticate(&deps.storage, &ViewingKey(key), canonical.as_slice())?;

    let account = Account::<CanonicalAddr>::load_self(&deps, &address)?;

    to_binary(&QueryResponse::Balance {
        pre_lock_amount: account.pre_lock_amount,
        total_bought: account.total_bought,
    })
}

pub(crate) fn get_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> QueryResult {
    let config = Config::<CanonicalAddr>::load_self(deps)?;

    let info = snip20::token_info_query(
        &deps.querier,
        BLOCK_SIZE,
        config.sold_token.code_hash,
        config.sold_token.address,
    )?;

    to_binary(&QueryResponse::TokenInfo {
        name: format!("IDO for {}", info.name),
        symbol: format!("IDO:{}", info.symbol),
        decimals: 0,
        total_supply: None,
    })
}
