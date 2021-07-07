use fadroma::scrt::{
    BLOCK_SIZE,
    contract::*,
    addr::Canonize,
    callback::ContractInstance,
    toolkit::snip20,
    utils::viewing_key::ViewingKey
};
use sienna_reward_schedule::RewardPool;

contract! {

    [State] {
        provided_token: Option<ContractInstance<CanonicalAddr>>,
        rewarded_token: ContractInstance<CanonicalAddr>,
        viewing_key:    ViewingKey,
        pool:           RewardPool
    }

    [Init] (deps, env, msg: {
        provided_token: Option<ContractInstance<HumanAddr>>,
        rewarded_token: ContractInstance<HumanAddr>,
        viewing_key:    ViewingKey
    }) {
        save_state!(State {
            provided_token: match provided_token {
                None => None,
                Some(provided_token) => Some(provided_token.canonize(&deps.api)?)
            },
            rewarded_token: rewarded_token.canonize(&deps.api)?,
            pool: RewardPool::new(env.block.time),
            viewing_key: viewing_key.clone()
        });

        InitResponse {
            messages: vec![
                snip20::set_viewing_key_msg(
                    viewing_key.0,
                    None,
                    BLOCK_SIZE,
                    rewarded_token.code_hash,
                    rewarded_token.address
                )?,
            ],
            log: vec![]
        }
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
                        snip20::transfer_msg(
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
            let balance = snip20::balance_query(
                &deps.querier,
                env.contract.address.clone(),
                state.viewing_key.0.clone(),
                BLOCK_SIZE,
                state.rewarded_token.code_hash.clone(),
                deps.api.human_address(&state.rewarded_token.address)?
            )?;
            let address = deps.api.canonical_address(&env.message.sender)?;
            let claimed = state.pool.claim(balance.amount, address)?;
            save_state!();
            Ok(HandleResponse {
                messages: vec![
                    snip20::transfer_msg(
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
