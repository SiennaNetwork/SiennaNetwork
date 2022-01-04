use amm_shared::fadroma::{
    platform::{
        from_binary, log, Api, Binary, Env, Extern, HandleResponse, HumanAddr, Querier,
        StdError, StdResult, Storage, Uint128,
        secret_toolkit::snip20,
        BLOCK_SIZE,
    },
    admin::assert_admin,
    auth_proc::require_admin,
    storage::traits1::Storable
};
use amm_shared::TokenType;
use amm_shared::msg::launchpad::{ReceiverCallbackMsg, TokenSettings};

use crate::data::{
    load_all_accounts, load_or_create_account, load_viewing_key, save_account, Account, Config,
    TokenConfig,
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
