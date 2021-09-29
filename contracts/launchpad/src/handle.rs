use amm_shared::admin::{admin::assert_admin, require_admin};
use amm_shared::{
    fadroma::scrt::{
        callback::ContractInstance,
        cosmwasm_std::{
            from_binary, log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
            HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
        },
        storage::Storable,
        toolkit::snip20,
        BLOCK_SIZE,
    },
    msg::ido::HandleMsg as IDOHandleMsg,
    msg::launchpad::{ReceiverCallbackMsg, TokenSettings},
    TokenType,
};

use crate::data::{
    load_all_accounts, load_or_create_account, load_viewing_key, save_account, AccounTokenEntry,
    Account, Config, TokenConfig,
};

use crate::helpers::*;

/// Handler that will receive calls from snip20 interface and it will handle it accordingly
pub(crate) fn receive_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let token_config =
        Config::load_self(deps)?.get_token_config(Some(env.message.sender.clone()))?;

    match from_binary(&msg)? {
        ReceiverCallbackMsg::Lock {} => lock(deps, env, Some(from), token_config, amount),
        ReceiverCallbackMsg::Unlock { entries } => {
            unlock(deps, env, Some(from), token_config, entries)
        }
    }
}

/// Perform locking of the funds into the launchpad account
pub(crate) fn lock<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: Option<HumanAddr>,
    token_config: TokenConfig,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let from = from.unwrap_or_else(|| env.message.sender.clone());

    let mut account: Account = load_or_create_account(deps, &from)?;
    let mut messages = vec![];

    let (change_amount, number_of_entry) = account.lock(env.block.time, &token_config, amount)?;

    // If the amount sent is more then a round number of segments we return the rest back
    // to the sender. We won't lock partial amounts of tokens in the launchpad
    if !change_amount.is_zero() {
        create_transfer_message(
            &token_config,
            &mut messages,
            env.contract.address,
            from,
            change_amount,
        )?;
    }

    save_account(deps, account)?;

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "lock"),
            log("locked_amount", (amount - change_amount)?),
            log("change_amount", change_amount),
            log("number_of_entry", number_of_entry),
        ],
        data: None,
    })
}

/// Perform unlocking of the funds from the launchpad, user needs to tell the contract
/// how many of the entries he would like to unlock and then the amount is calculated
/// and sent to user. Segment price for entry/token is decided in the token_config
pub(crate) fn unlock<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: Option<HumanAddr>,
    token_config: TokenConfig,
    entries: u32,
) -> StdResult<HandleResponse> {
    let from = from.unwrap_or_else(|| env.message.sender.clone());

    let mut account: Account = load_or_create_account(deps, &from)?;
    let mut messages = vec![];

    let (amount, remaining_number_of_entry) = account.unlock(&token_config, entries)?;

    if !amount.is_zero() {
        create_transfer_message(
            &token_config,
            &mut messages,
            env.contract.address,
            from,
            amount,
        )?;
    }

    save_account(deps, account)?;

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "unlock"),
            log("unlocked_entries", entries),
            log("unlocked_amount", amount),
            log("remaining_number_of_entry", remaining_number_of_entry),
        ],
        data: None,
    })
}

/// Handler that will send message to callback IDO contract and will fill it up with drawn addresses
pub(crate) fn draw_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    callback: ContractInstance<HumanAddr>,
    tokens: Vec<Option<HumanAddr>>,
    number: u32,
) -> StdResult<HandleResponse> {
    let config = Config::load_self(deps)?;

    let mut token_configs: Vec<TokenConfig> = vec![];
    for token in tokens {
        token_configs.push(config.get_token_config(token)?);
    }

    let accounts = load_all_accounts(deps)?;
    let mut entries: Vec<(HumanAddr, AccounTokenEntry)> = vec![];

    for account in &accounts {
        let account_entries = account.get_entries(&token_configs, env.block.time);
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
        // Randomly generate index to get from entry list
        let index: usize = gen_rand_range(0, (entries.len() - 1) as u64, env.block.time) as usize;

        match &entries.get(index) {
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

    // Loop through the accounts and update the drawn accounts so they are marked with
    // last drawn timestamp. This is the actual reason we are doing this as a handle, and not query
    for mut account in accounts.into_iter() {
        if addresses.iter().position(|a| a == &account.owner).is_some() {
            account.mark_as_drawn(&token_configs, env.block.time);

            save_account(deps, account)?;
        }
    }

    let mut messages = vec![];
    let addresses_len = addresses.len();

    // If we have drawn any addresses we will create a callback message for the IDO
    // that will deliver them there and fill the empty seats
    if addresses_len > 0 {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: callback.address,
            callback_code_hash: callback.code_hash,
            msg: to_binary(&IDOHandleMsg::AdminAddAddresses { addresses })?,
            send: vec![],
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "draw"),
            log("number", number),
            log("addresses_len", addresses_len),
        ],
        data: None,
    })
}

/// Admin can add new token to the list of tokens that can be locked in the launchpad contract
#[require_admin]
pub(crate) fn admin_add_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token: TokenSettings,
) -> StdResult<HandleResponse> {
    let mut config = Config::load_self(deps)?;
    let viewing_key = load_viewing_key(&deps.storage)?;
    let mut messages = vec![];

    match &token.token_type {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            // Set created viewing key onto the contract so we can check the balance later
            messages.push(snip20::set_viewing_key_msg(
                viewing_key.to_string(),
                None,
                BLOCK_SIZE,
                token_code_hash.clone(),
                contract_addr.clone(),
            )?);

            // Register this contract as a receiver of the callback messages
            // from the custom input token. This will allow us to receive
            // message after the tokens have been sent and will make the lock
            // happen in a single transaction
            messages.push(snip20::register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                BLOCK_SIZE,
                token_code_hash.clone(),
                contract_addr.clone(),
            )?);
        }
        _ => (),
    };

    config.add_token(&deps.querier, token)?;
    config.save(deps)?;

    Ok(HandleResponse {
        messages,
        log: vec![log("action", "admin_add_token")],
        data: None,
    })
}

/// Admin can remove the token from the lanchpad. This will disable its feature to be locked.
/// It will also send all the locked funds from all the accounts back to their owners.
#[require_admin]
pub(crate) fn admin_remove_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    index: u32,
) -> StdResult<HandleResponse> {
    let mut config = Config::load_self(deps)?;

    let token_config = config.remove_token(index)?;
    let accounts = load_all_accounts(deps)?;
    let mut messages = vec![];
    let mut total_amount = Uint128::zero();
    let mut total_entries = 0;

    for mut account in accounts {
        let (amount, entries) = account.unlock_all(&token_config)?;

        if amount.u128() > 0_u128 && entries > 0 {
            total_amount += amount;
            total_entries += entries;

            create_transfer_message(
                &token_config,
                &mut messages,
                env.contract.address.clone(),
                account.owner.clone(),
                amount,
            )?;

            save_account(deps, account)?;
        }
    }

    config.save(deps)?;

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "admin_remove_token"),
            log("total_amount", total_amount),
            log("total_entries", total_entries),
        ],
        data: None,
    })
}
