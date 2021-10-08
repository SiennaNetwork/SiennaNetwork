use fadroma::scrt::{
    BLOCK_SIZE,
    cosmwasm_std::to_vec,
    callback::{ContractInstance as ContractLink},
    contract::*,
    snip20_api::ISnip20,
    vk::{ViewingKey,
         auth_handle, authenticate, AuthHandleMsg,
         DefaultHandleImpl as AuthHandle},
    admin::{DefaultHandleImpl as AdminHandle,
            admin_handle, AdminHandleMsg, load_admin,
            assert_admin, save_admin}};

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
        admin:        HumanAddr,
        collector:    HumanAddr,
        reward_token: ContractLink<HumanAddr>
    }) {
        deps.storage.set(ADMIN,        &to_vec(&admin)?);
        deps.storage.set(COLLECTOR,    &to_vec(&collector)?);
        deps.storage.set(REWARD_TOKEN, &to_vec(&reward_token)?);
    }

    [Query] (deps, _state, msg) -> Response {}

    [Response] {}

    [Handle] (deps, env, state, msg) -> Response {
        Claim (pool: ContractLink<HumanAddr>, key: String) {
            let reward_token = ISnip20::attach(deps.storage.get(REWARD_TOKEN)?);
            let claimable = deps.querier.query();
            let receivable = claimable.multiply_ratio(  1u128, 159u128);
            let returnable = claimable.multiply_ratio(158u128, 159u128);
            Ok(HandleResponse {
                messages: vec![
                    reward_pool
                ],
                log: vec![],
                data: None
            })
        }
    }
}

pub fn get_claimable (
    at: u64, address: &HumanAddr, key: &str
) -> StdResult<Uint128> {
    let mut msg = to_binary(RewardsQuery::UserInfo {
        at,
        address: address.clone(),
        key:     key.to_string()
    })?;
    space_pad(&mut msg.0, BLOCK_SIZE);
    querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr,
            callback_code_hash,
            msg,
        }))
        .map_err(|err| {
            StdError::generic_err(format!("Error performing {} query: {}", self, err))
        })
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
