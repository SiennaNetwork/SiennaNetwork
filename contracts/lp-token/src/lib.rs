use std::ops::RangeInclusive;
use cosmwasm_std::{
    Extern, Storage, Api, Querier, StdResult, HumanAddr,
    InitResponse, Env, HandleResponse, Binary, Uint128,
    StdError, to_binary
};

use amm_shared::snip20_impl as composable_snip20;

use composable_snip20::{Snip20, snip20_init, snip20_handle, snip20_query, check_if_admin};
use composable_snip20::state::{Config, Balances};
use composable_snip20::transaction_history::store_burn;
use composable_snip20::msg::{InitMsg, HandleMsg, QueryMsg, ResponseStatus, HandleAnswer};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    snip20_init(deps, env, msg, LpTokenImpl)
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    snip20_handle(deps, env, msg, LpTokenImpl)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg
) -> StdResult<Binary> {
    snip20_query(deps, msg, LpTokenImpl)
}

struct LpTokenImpl;

impl Snip20 for LpTokenImpl {
    fn symbol_range(&self) -> RangeInclusive<usize> {
        3..=12
    }

    fn name_range(&self) -> RangeInclusive<usize> {
        3..=200
    }

    fn burn_from<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
        env: Env,
        owner: HumanAddr,
        amount: Uint128,
        memo: Option<String>
    ) -> StdResult<HandleResponse> {
        let mut config = Config::from_storage(&mut deps.storage);

        check_if_admin(&config, &env.message.sender)?;

        let symbol = config.constants()?.symbol;
        let raw_amount = amount.u128();

        // remove from supply
        let mut total_supply = config.total_supply();
        if let Some(new_total_supply) = total_supply.checked_sub(raw_amount) {
            total_supply = new_total_supply;
        } else {
            return Err(StdError::generic_err(
                "You're trying to burn more than is available in the total supply",
            ));
        }

        config.set_total_supply(total_supply);
    
        // subtract from owner account
        let owner = deps.api.canonical_address(&owner)?;

        let mut balances = Balances::from_storage(&mut deps.storage);
        let mut account_balance = balances.balance(&owner);
    
        if let Some(new_balance) = account_balance.checked_sub(raw_amount) {
            account_balance = new_balance;
        } else {
            return Err(StdError::generic_err(format!(
                "insufficient funds to burn: balance={}, required={}",
                account_balance, raw_amount
            )));
        }

        balances.set_account_balance(&owner, account_balance);
    
        store_burn(
            &mut deps.storage,
            &owner,
            &deps.api.canonical_address(&env.message.sender)?,
            amount,
            symbol,
            memo,
            &env.block,
        )?;
    
        let res = HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::BurnFrom { status: ResponseStatus::Success })?),
        };
    
        Ok(res)
    }
}


#[cfg(target_arch = "wasm32")]
mod wasm {
    use cosmwasm_std::{
        do_handle, do_init, do_query, ExternalApi, ExternalQuerier, ExternalStorage,
    };

    #[no_mangle]
    extern "C" fn init(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_init(
            &super::init::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn handle(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_handle(
            &super::handle::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn query(msg_ptr: u32) -> u32 {
        do_query(
            &super::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
            msg_ptr,
        )
    }

    // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
    // automatically because we `use cosmwasm_std`.
}
