use cosmwasm_std::{
    log, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::state::{config, config_read, State};
use scrt_finance::lp_staking_msg::LPStakingHandleMsg;
use scrt_finance::master_msg::{MasterHandleAnswer, MasterInitMsg, MasterQueryMsg};
use scrt_finance::master_msg::{MasterHandleMsg, MasterQueryAnswer};
use scrt_finance::types::{sort_schedule, Schedule, SpySettings, WeightInfo};
use secret_toolkit::snip20;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: MasterInitMsg,
) -> StdResult<InitResponse> {
    // The impl. later on relies on the schedule being sorted
    let mut mint_schedule = msg.minting_schedule;
    sort_schedule(&mut mint_schedule);

    let state = State {
        admin: env.message.sender,
        gov_token_addr: msg.gov_token_addr,
        gov_token_hash: msg.gov_token_hash,
        total_weight: 0,
        minting_schedule: mint_schedule,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: MasterHandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        MasterHandleMsg::UpdateAllocation {
            spy_addr,
            spy_hash,
            hook,
        } => update_allocation(deps, env, spy_addr, spy_hash, hook),
        MasterHandleMsg::SetWeights { weights } => set_weights(deps, env, weights),
        MasterHandleMsg::SetSchedule { schedule } => set_schedule(deps, env, schedule),
        MasterHandleMsg::SetGovToken { addr, hash } => set_gov_token(deps, env, addr, hash),
        MasterHandleMsg::ChangeAdmin { addr } => change_admin(deps, env, addr),
    }
}

fn set_schedule<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    schedule: Schedule,
) -> StdResult<HandleResponse> {
    let mut st = config(&mut deps.storage);
    let mut state = st.load()?;

    enforce_admin(state.clone(), env)?;

    let mut s = schedule;
    sort_schedule(&mut s);

    state.minting_schedule = s;
    st.save(&state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&MasterHandleAnswer::Success)?),
    })
}

fn set_weights<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    weights: Vec<WeightInfo>,
) -> StdResult<HandleResponse> {
    let mut state = config_read(&deps.storage).load()?;

    enforce_admin(state.clone(), env.clone())?;

    let mut messages = vec![];
    let mut logs = vec![];
    let mut new_weight_counter = 0;
    let mut old_weight_counter = 0;

    // Update reward contracts one by one
    for to_update in weights {
        let mut rs = TypedStoreMut::attach(&mut deps.storage);
        let mut spy_settings =
            rs.load(to_update.address.clone().0.as_bytes())
                .unwrap_or(SpySettings {
                    weight: 0,
                    last_update_block: env.block.height,
                });

        // There is no need to update a SPY twice in a block, and there is no need to update a SPY
        // that had 0 weight until now
        if spy_settings.last_update_block < env.block.height && spy_settings.weight > 0 {
            // Calc amount to mint for this spy contract and push to messages
            let rewards = get_spy_rewards(
                env.block.height,
                state.total_weight,
                &state.minting_schedule,
                spy_settings.clone(),
            );
            messages.push(snip20::mint_msg(
                to_update.address.clone(),
                Uint128(rewards),
                None,
                1,
                state.gov_token_hash.clone(),
                state.gov_token_addr.clone(),
            )?);

            // Notify to the spy contract on the new allocation
            messages.push(
                WasmMsg::Execute {
                    contract_addr: to_update.address.clone(),
                    callback_code_hash: to_update.hash,
                    msg: to_binary(&LPStakingHandleMsg::NotifyAllocation {
                        amount: Uint128(rewards),
                        hook: None,
                    })?,
                    send: vec![],
                }
                .into(),
            );
        }

        let old_weight = spy_settings.weight;
        let new_weight = to_update.weight;

        // Set new weight and update total counter
        spy_settings.weight = new_weight;
        spy_settings.last_update_block = env.block.height;
        rs.store(to_update.address.0.as_bytes(), &spy_settings)?;

        // Update counters to batch update after the loop
        new_weight_counter += new_weight;
        old_weight_counter += old_weight;

        logs.push(log("weight_update", to_update.address.0))
    }

    state.total_weight = state.total_weight - old_weight_counter + new_weight_counter;
    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages,
        log: logs,
        data: Some(to_binary(&MasterHandleAnswer::Success)?),
    })
}

fn update_allocation<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    spy_address: HumanAddr,
    spy_hash: String,
    hook: Option<Binary>,
) -> StdResult<HandleResponse> {
    let state = config_read(&deps.storage).load()?;

    let mut rs = TypedStoreMut::attach(&mut deps.storage);
    let mut spy_settings = rs.load(spy_address.0.as_bytes()).unwrap_or(SpySettings {
        weight: 0,
        last_update_block: env.block.height,
    });

    let mut rewards = 0;
    let mut messages = vec![];
    if spy_settings.last_update_block < env.block.height && spy_settings.weight > 0 {
        // Calc amount to minLPStakingHandleMsgt for this spy contract and push to messages
        rewards = get_spy_rewards(
            env.block.height,
            state.total_weight,
            &state.minting_schedule,
            spy_settings.clone(),
        );
        messages.push(snip20::mint_msg(
            spy_address.clone(),
            Uint128(rewards),
            None,
            1,
            state.gov_token_hash.clone(),
            state.gov_token_addr,
        )?);

        spy_settings.last_update_block = env.block.height;
        rs.store(spy_address.0.as_bytes(), &spy_settings)?;
    }

    // Notify to the spy contract on the new allocation
    messages.push(
        WasmMsg::Execute {
            contract_addr: spy_address.clone(),
            callback_code_hash: spy_hash,
            msg: to_binary(&LPStakingHandleMsg::NotifyAllocation {
                amount: Uint128(rewards),
                hook,
            })?,
            send: vec![],
        }
        .into(),
    );

    Ok(HandleResponse {
        messages,
        log: vec![log("update_allocation", spy_address.0)],
        data: Some(to_binary(&MasterHandleAnswer::Success)?),
    })
}

fn set_gov_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    gov_addr: HumanAddr,
    gov_hash: String,
) -> StdResult<HandleResponse> {
    let mut state = config_read(&deps.storage).load()?;

    enforce_admin(state.clone(), env)?;

    state.gov_token_addr = gov_addr.clone();
    state.gov_token_hash = gov_hash;

    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("set_gov_token", gov_addr.0)],
        data: Some(to_binary(&MasterHandleAnswer::Success)?),
    })
}

fn change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin_addr: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut state = config_read(&deps.storage).load()?;

    enforce_admin(state.clone(), env)?;

    state.admin = admin_addr;

    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&MasterHandleAnswer::Success)?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: MasterQueryMsg,
) -> StdResult<Binary> {
    match msg {
        MasterQueryMsg::Admin {} => to_binary(&query_admin(deps)?),
        MasterQueryMsg::GovToken {} => to_binary(&query_gov_token(deps)?),
        MasterQueryMsg::Schedule {} => to_binary(&query_schedule(deps)?),
        MasterQueryMsg::SpyWeight { addr } => to_binary(&query_spy_weight(deps, addr)?),
        MasterQueryMsg::Pending { spy_addr, block } => {
            to_binary(&query_pending_rewards(deps, spy_addr, block)?)
        }
    }
}

fn query_admin<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<MasterQueryAnswer> {
    let state = config_read(&deps.storage).load()?;

    Ok(MasterQueryAnswer::Admin {
        address: state.admin,
    })
}

fn query_gov_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<MasterQueryAnswer> {
    let state = config_read(&deps.storage).load()?;

    Ok(MasterQueryAnswer::GovToken {
        token_addr: state.gov_token_addr,
        token_hash: state.gov_token_hash,
    })
}

fn query_schedule<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<MasterQueryAnswer> {
    let state = config_read(&deps.storage).load()?;

    Ok(MasterQueryAnswer::Schedule {
        schedule: state.minting_schedule,
    })
}

fn query_spy_weight<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    spy_address: HumanAddr,
) -> StdResult<MasterQueryAnswer> {
    let spy = TypedStore::attach(&deps.storage)
        .load(spy_address.0.as_bytes())
        .unwrap_or(SpySettings {
            weight: 0,
            last_update_block: 0,
        });

    Ok(MasterQueryAnswer::SpyWeight { weight: spy.weight })
}

fn query_pending_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    spy_addr: HumanAddr,
    block: u64,
) -> StdResult<MasterQueryAnswer> {
    let state = config_read(&deps.storage).load()?;
    let spy = TypedStore::attach(&deps.storage)
        .load(spy_addr.0.as_bytes())
        .unwrap_or(SpySettings {
            weight: 0,
            last_update_block: block,
        });

    let amount = get_spy_rewards(block, state.total_weight, &state.minting_schedule, spy);

    Ok(MasterQueryAnswer::Pending {
        amount: Uint128(amount),
    })
}

fn get_spy_rewards(
    current_block: u64,
    total_weight: u64,
    schedule: &Schedule,
    spy_settings: SpySettings,
) -> u128 {
    let mut last_update_block = spy_settings.last_update_block;

    let mut multiplier = 0;
    // Going serially assuming that schedule is not a big vector
    for u in schedule.to_owned() {
        if last_update_block < u.end_block {
            if current_block > u.end_block {
                multiplier += (u.end_block - last_update_block) as u128 * u.mint_per_block.u128();
                last_update_block = u.end_block;
            } else {
                multiplier += (current_block - last_update_block) as u128 * u.mint_per_block.u128();
                // last_update_block = current_block;
                break; // No need to go further up the schedule
            }
        }
    }

    (multiplier * spy_settings.weight as u128) / total_weight as u128
}

fn enforce_admin(config: State, env: Env) -> StdResult<()> {
    if config.admin != env.message.sender {
        return Err(StdError::generic_err(format!(
            "not an admin: {}",
            env.message.sender
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
}
