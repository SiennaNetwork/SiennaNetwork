#[macro_use] extern crate fadroma;

use secret_toolkit::{snip20, storage::{TypedStore, TypedStoreMut}};

pub use scrt_finance::{
    lp_staking_msg::LPStakingHandleMsg,
    master_msg::{
        MasterHandleAnswer, MasterInitMsg, MasterQueryMsg,
        MasterHandleMsg, MasterQueryAnswer
    },
    types::{sort_schedule, Schedule, SpySettings, WeightInfo}
};

pub type ContractLink<A> = (A, String);

contract!(

    [State] {
        /// Address that can control the contract.
        admin:            HumanAddr,
        /// Address of governance token
        gov_token:        ContractLink<HumanAddr>,
        /// TODO document
        total_weight:     u64,
        /// TODO document
        minting_schedule: Schedule
    }

    [Init] (deps, env, msg: MasterInitMsg) {
        // The impl. later on relies on the schedule being sorted
        let mut minting_schedule = msg.minting_schedule;
        sort_schedule(&mut minting_schedule);
        State {
            admin:            env.message.sender,
            gov_token:        msg.gov_token,
            total_weight:     0,
            minting_schedule: minting_schedule,
        }
    }

    [Query] (deps, state, msg: MasterQueryMsg) -> MasterQueryAnswer {
        /// Get the address of the current admin
        Admin () {
            Ok(MasterQueryAnswer::Admin { address: state.admin })
        }
        GovToken () {
            Ok(MasterQueryAnswer::GovToken {
                token_addr: state.gov_token.0,
                token_hash: state.gov_token.1,
            })
        }
        Schedule () {
            Ok(MasterQueryAnswer::Schedule { schedule: state.minting_schedule })
        }
        SpyWeight (addr: HumanAddr) {
            let spy = get_spy_settings(&deps.storage, addr);
            Ok(MasterQueryAnswer::SpyWeight { weight: spy.weight })
        }
        Pending (spy_addr: HumanAddr, block: u64) {
            let spy = get_spy_settings(&deps.storage, spy_addr);
            let amount = get_spy_rewards(block, state.total_weight, &state.minting_schedule, spy);
            Ok(MasterQueryAnswer::Pending { amount: Uint128(amount) })
        }
    }

    [Response] {}

    [Handle] (deps, env, state, msg: MasterHandleMsg) -> MasterHandleAnswer {
        UpdateAllocation (spy_addr: HumanAddr, spy_hash: String, hook: Option<Binary>) {
            let state = config_read(&deps.storage).load()?;

            let mut rs = TypedStoreMut::attach(&mut deps.storage);
            let mut spy_settings = rs.load(spy_addr.0.as_bytes()).unwrap_or(SpySettings {
                weight: 0,
                last_update_block: env.block.height,
            });

            let mut rewards = 0;
            let mut messages = vec![];
            if spy_settings.last_update_block < env.block.height && spy_settings.weight > 0 {
                // Calc amount to mint for this spy contract and push to messages
                rewards = get_spy_rewards(
                    env.block.height,
                    state.total_weight,
                    &state.minting_schedule,
                    spy_settings.clone(),
                );
                messages.push(snip20::mint_msg(
                    spy_addr.clone(),
                    Uint128(rewards),
                    None,
                    1,
                    state.gov_token_hash.clone(),
                    state.gov_token_addr,
                )?);

                spy_settings.last_update_block = env.block.height;
                rs.store(spy_addr.0.as_bytes(), &spy_settings)?;
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
        SetWeights (weights: Vec<WeightInfo>) {
            is_admin(&deps.api, &state, &env)?;
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
        SetSchedule (schedule) {
            is_admin(&deps.api, &state, &env)?;
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
        SetGovToken (addr, hash) {
            is_admin(&deps.api, &state, &env)?;
            state.gov_token_addr = gov_addr.clone();
            state.gov_token_hash = gov_hash;
            config(&mut deps.storage).save(&state)?;
            Ok(HandleResponse {
                messages: vec![],
                log: vec![log("set_gov_token", gov_addr.0)],
                data: Some(to_binary(&MasterHandleAnswer::Success)?),
            })
        }
        ChangeAdmin (addr) {
            is_admin(&deps.api, &state, &env)?;
            state.admin = admin_addr;
            config(&mut deps.storage).save(&state)?;
            Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&MasterHandleAnswer::Success)?),
            })
        }
    }
);

fn get_spy_settings <S: Storage> (
    storage: &S,
    spy_address: HumanAddr
) -> SpySettings {
    TypedStore::attach(storage)
        .load(spy_address.0.as_bytes())
        .unwrap_or(SpySettings { weight: 0, last_update_block: 0, })
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

fn is_admin <A:Api> (api: &A, state: &State, env: &Env) -> StdResult<()> {
    //let sender = api.canonical_address(&env.message.sender)?;
    if state.admin == env.message.sender { return Ok(()) }
    Err(StdError::Unauthorized { backtrace: None })
}
