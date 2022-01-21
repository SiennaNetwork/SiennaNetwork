use lend_shared::{
    fadroma::{
        auth::ViewingKey,
        vk_auth::authenticate,
        cosmwasm_std::{
            Extern, Storage, Querier,
            Api, StdResult, HumanAddr
        },
        Canonize
    },
    interfaces::market::{AuthMethod, MarketPermissions},
    core::MasterKey
};

use crate::state::{Account, Contracts};

pub fn auth<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    method: AuthMethod,
    permission: MarketPermissions
) -> StdResult<Account> {
    match method {
        AuthMethod::Permit(permit) => {
            let self_ref = Contracts::load_self_ref(deps)?;

            let address = permit.validate_with_permissions(
                deps,
                self_ref.address,
                vec![ permission ]
            )?;

            Account::of(deps, &address)
        },
        AuthMethod::ViewingKey { key, address } => {
            auth_user_key(deps, key, address)
        },
        AuthMethod::Internal { key, address } => {
            MasterKey::check(&deps.storage, &key)?;

            Account::of(deps, &address)
        }
    }
}

#[inline]
pub fn auth_user_key<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
    address: HumanAddr
) -> StdResult<Account> {
    let canonical = address.canonize(&deps.api)?;
    authenticate(&deps.storage, &ViewingKey(key), canonical.as_slice())?;

    Ok(Account::from(canonical))
}
