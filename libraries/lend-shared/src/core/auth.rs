use fadroma::{
    schemars,
    permit::{Permit, Permission},
    auth::ViewingKey,
    vk_auth::authenticate,
    cosmwasm_std::{
        Extern, Storage, Querier,
        Api, StdResult, HumanAddr,
        CanonicalAddr
    },
    Canonize, ContractLink
};

use serde::{Serialize, Deserialize};

use crate::core::MasterKey;

#[derive(Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod<P: Permission> {
    Permit(Permit<P>),
    ViewingKey {
        address: HumanAddr,
        key: String
    },
    Internal {
        address: HumanAddr,
        key: MasterKey
    }
}

pub trait AuthenticatedUser: Sized {
    fn from_canonical<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: CanonicalAddr
    ) -> StdResult<Self>;

    #[inline]
    fn from_human<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: HumanAddr
    ) -> StdResult<Self> {
        let address = address.canonize(&deps.api)?;

        Self::from_canonical(deps, address)
    }

    fn authenticate<S: Storage, A: Api, Q: Querier, P: Permission, F>(
        deps: &Extern<S, A, Q>,
        method: AuthMethod<P>,
        permission: P,
        load_self_ref: F
    ) -> StdResult<Self>
        where F: FnOnce(&Extern<S, A, Q>) -> StdResult<ContractLink<HumanAddr>>
    {
        match method {
            AuthMethod::Permit(permit) => {
                let self_ref = load_self_ref(deps)?;
    
                let address = permit.validate_with_permissions(
                    deps,
                    self_ref.address,
                    vec![ permission ]
                )?;
    
                Self::from_human(deps, address)
            },
            AuthMethod::ViewingKey { key, address } => {
                Self::auth_viewing_key(deps, key, &address)
            },
            AuthMethod::Internal { key, address } => {
                MasterKey::check(&deps.storage, &key)?;
    
                Self::from_human(deps, address)
            }
        }
    }

    #[inline]
    fn auth_viewing_key<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        key: String,
        address: &HumanAddr
    ) -> StdResult<Self> {
        let canonical = address.canonize(&deps.api)?;
        authenticate(&deps.storage, &ViewingKey(key), canonical.as_slice())?;

        Self::from_canonical(deps, canonical)
    }
}

impl<P: Permission> From<Permit<P>> for AuthMethod<P> {
    #[inline]
    fn from(permit: Permit<P>) -> Self {
        Self::Permit(permit)
    }
}
