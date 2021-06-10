use cosmwasm_std::{
    Extern, Storage, Api, Querier, Env, HandleResponse, to_binary,
    StdResult
};
use composable_auth::{
    AuthHandle, ViewingKey, save_viewing_key, HandleAnswer
};

use crate::state::load_config;

pub(crate) struct AuthImpl;

impl AuthHandle for AuthImpl {
    fn create_viewing_key<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
        env: Env,
        entropy: String
    ) -> StdResult<HandleResponse> {
        let config = load_config(deps)?;

        let key = ViewingKey::new(&env, config.prng_seed.as_slice(), entropy.as_bytes());
        
        let address = deps.api.canonical_address(&env.message.sender)?;
        save_viewing_key(deps, address.as_slice(), &key)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::CreateViewingKey {
                key
            })?)
        })
    }
}
