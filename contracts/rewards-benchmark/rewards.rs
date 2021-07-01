use fadroma::scrt::{
    BLOCK_SIZE,
    contract::*,
    addr::Canonize,
    callback::ContractInstance,
    toolkit::snip20
};
use sienna_reward_schedule::RewardPool;

contract! {

    [State] {
        provided_token: ContractInstance<CanonicalAddr>,
        rewarded_token: ContractInstance<CanonicalAddr>,
        pool:           RewardPool
    }

    [Init] (deps, env, msg: {
        provided_token: ContractInstance<HumanAddr>,
        rewarded_token: ContractInstance<HumanAddr>
    }) {
        save_state!(State {
            provided_token: provided_token.canonize(&deps.api)?,
            rewarded_token: rewarded_token.canonize(&deps.api)?,
            pool:           RewardPool::new(env.block.time)
        })
    }

    [Query] (_deps, _state, msg) -> Response {
        Status () {
            Ok(Response::Status {})
        }
    }

    [Response] { Status {} }

    [Handle] (deps, env, state, msg) -> Response {

        Lock (amount: Uint128) {
            let address = deps.api.canonical_address(&env.message.sender)?;
            Ok(HandleResponse {
                messages: vec![
                    snip20::transfer_from_msg(
                        env.message.sender,
                        env.contract.address,
                        state.pool.lock(env.block.time, address, amount),
                        None,
                        BLOCK_SIZE,
                        state.provided_token.code_hash,
                        deps.api.human_address(&state.provided_token.address)?
                    )?
                ],
                log: vec![],
                data: None
            })
        }

        Retrieve (amount: Uint128) {
            let address = deps.api.canonical_address(&env.message.sender)?;
            Ok(HandleResponse {
                messages: vec![
                    snip20::transfer_from_msg(
                        env.contract.address,
                        env.message.sender,
                        state.pool.retrieve(env.block.time, address, amount)?,
                        None,
                        BLOCK_SIZE,
                        state.provided_token.code_hash,
                        deps.api.human_address(&state.provided_token.address)?
                    )?
                ],
                log: vec![],
                data: None
            })
        }

        Claim () {
            let address = deps.api.canonical_address(&env.message.sender)?;
            Ok(HandleResponse {
                messages: vec![
                    snip20::transfer_from_msg(
                        env.contract.address,
                        env.message.sender,
                        state.pool.claim(address)?,
                        None,
                        BLOCK_SIZE,
                        state.rewarded_token.code_hash,
                        deps.api.human_address(&state.rewarded_token.address)?
                    )?
                ],
                log: vec![],
                data: None
            })
        }

    }
}
