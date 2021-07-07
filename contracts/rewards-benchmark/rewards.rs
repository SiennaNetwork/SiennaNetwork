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
        provided_token: Option<ContractInstance<CanonicalAddr>>,
        rewarded_token: ContractInstance<CanonicalAddr>,
        pool:           RewardPool
    }

    [Init] (deps, env, msg: {
        provided_token: Option<ContractInstance<HumanAddr>>,
        rewarded_token: ContractInstance<HumanAddr>
    }) {
        save_state!(State {
            provided_token: match provided_token {
                None => None,
                Some(provided_token) => Some(provided_token.canonize(&deps.api)?)
            },
            rewarded_token: rewarded_token.canonize(&deps.api)?,
            pool: RewardPool::new(env.block.time)
        })
    }

    [Query] (_deps, _state, msg) -> Response {
        Status () {
            Ok(Response::Status {})
        }
    }

    [Response] { Status {} }

    [Handle] (deps, env, state, msg) -> Response {

        SetProvidedToken (address: HumanAddr, code_hash: String) {
            let address = deps.api.canonical_address(&address)?;
            state.provided_token = Some(ContractInstance { address, code_hash });
            save_state!();
            Ok(HandleResponse::default())
        }

        Lock (amount: Uint128) {
            let address = deps.api.canonical_address(&env.message.sender)?;
            let locked = state.pool.lock(env.block.time, address, amount);
            save_state!();
            match state.provided_token {
                Some(provided_token) => Ok(HandleResponse {
                    messages: vec![
                        snip20::transfer_from_msg(
                            env.message.sender,
                            env.contract.address,
                            locked,
                            None,
                            BLOCK_SIZE,
                            provided_token.code_hash,
                            deps.api.human_address(&provided_token.address)?
                        )?
                    ],
                    log: vec![],
                    data: None
                }),
                None => panic!()
            }
        }

        Retrieve (amount: Uint128) {
            let address = deps.api.canonical_address(&env.message.sender)?;
            let retrieved = state.pool.retrieve(env.block.time, address, amount)?;
            save_state!();
            match state.provided_token {
                Some(provided_token) => Ok(HandleResponse {
                    messages: vec![
                        snip20::transfer_from_msg(
                            env.contract.address,
                            env.message.sender,
                            retrieved,
                            None,
                            BLOCK_SIZE,
                            provided_token.code_hash,
                            deps.api.human_address(&provided_token.address)?
                        )?
                    ],
                    log: vec![],
                    data: None
                }),
                None => panic!()
            }
        }

        Claim () {
            let address = deps.api.canonical_address(&env.message.sender)?;
            let claimed = state.pool.claim(address)?;
            save_state!();
            Ok(HandleResponse {
                messages: vec![
                    snip20::transfer_from_msg(
                        env.contract.address,
                        env.message.sender,
                        claimed,
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
