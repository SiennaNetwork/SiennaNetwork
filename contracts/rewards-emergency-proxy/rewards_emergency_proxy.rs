#[cfg(test)] mod rewards_emergency_proxy_test;

use fadroma::scrt::{
    BLOCK_SIZE,
    cosmwasm_std::{to_vec, from_slice, QueryRequest, WasmQuery},
    callback::{ContractInstance as ContractLink},
    contract::*,
    snip20_api::ISnip20
};

use sienna_rewards::msg::{
    Query    as RewardsQuery,
    Response as RewardsResponse,
    Handle   as RewardsHandle
};

pub const ADMIN:        &[u8] = b"/admin";
pub const COLLECTOR:    &[u8] = b"/collector";
pub const REWARD_TOKEN: &[u8] = b"/reward_token";

contract! {

    [NoGlobalState] {}

    [Init] (deps, env, msg: {
        collector:    HumanAddr,
        reward_token: ContractLink<HumanAddr>
    }) {
        deps.storage.set(ADMIN,        &to_vec(&env.message.sender)?);
        deps.storage.set(COLLECTOR,    &to_vec(&collector)?);
        deps.storage.set(REWARD_TOKEN, &to_vec(&reward_token)?);
        save_state!(NoGlobalState {});
        InitResponse { messages: vec![], log: vec![] }
    }

    [Query] (_deps, _state, msg) -> Response {
        Status () {
            Ok(Response::Status{})
        }
    }

    [Response] {
        Status {}
    }

    [Handle] (deps, env, _state, msg) -> Response {
        Claim (pool: ContractLink<HumanAddr>, key: String) {
            let claimable = get_claimable(
                &deps.querier, &pool,
                env.block.height, &env.message.sender, &key
            )?;
            if claimable == Uint128::zero() {
                return Err(StdError::generic_err("nothing to claim right now"))
            }
            let mut messages = vec![claim(&pool)?];
            let mut log = vec![];
            match deps.storage.get(&to_vec(&env.message.sender)?) {
                Some(_) => { /* already returned extra rewards */ },
                None => {
                    let reward_token_link: ContractLink<HumanAddr> =
                        from_slice(&deps.storage.get(REWARD_TOKEN).unwrap())?;
                    let collector: HumanAddr =
                        from_slice(&deps.storage.get(COLLECTOR).unwrap())?;
                    let reward_token = ISnip20::attach(&reward_token_link);
                    let receivable = claimable.multiply_ratio(  1u128, 159u128);
                    let returnable = claimable.multiply_ratio(158u128, 159u128);
                    deps.storage.set(&to_vec(&env.message.sender)?, &to_vec(&true)?);
                    messages.push(reward_token.transfer_from(
                        &env.message.sender,
                        &collector,
                        returnable
                    )?);
                    log.push(LogAttribute { key: "received".to_string(), value: claimable.into()  });
                    log.push(LogAttribute { key: "eligible".to_string(), value: receivable.into() });
                    log.push(LogAttribute { key: "returned".to_string(), value: returnable.into() });
                }
            }
            Ok(HandleResponse { messages, log, data: None })
        }
    }
}

pub fn get_claimable (
    querier: &impl Querier, pool: &ContractLink<HumanAddr>,
    at: u64, address: &HumanAddr, key: &str
) -> StdResult<Uint128> {
    let mut msg = to_binary(&RewardsQuery::UserInfo {
        at,
        address: address.clone(),
        key:     key.to_string()
    })?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr:      pool.address.clone(),
        callback_code_hash: pool.code_hash.clone(),
        msg,
    }))?;
    match result {
        RewardsResponse::UserInfo { user_claimable, .. } => Ok(user_claimable),
        _ => Err(StdError::generic_err(""))
    }
}

pub fn claim (
    pool: &ContractLink<HumanAddr>
) -> StdResult<CosmosMsg> {
    let mut msg = to_binary(&RewardsHandle::Claim {})?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    let execute = WasmMsg::Execute {
        contract_addr:      pool.address.clone(),
        callback_code_hash: pool.code_hash.clone(),
        msg,
        send: vec![],
    };
    Ok(execute.into())
}

pub fn space_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}
