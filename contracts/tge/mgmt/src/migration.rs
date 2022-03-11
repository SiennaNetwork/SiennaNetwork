use fadroma::{
    derive_contract::handle,
    killswitch::{Killswitch, ContractStatusLevel, set_status},
    admin::load_admin,
    cosmwasm_std,
    cosmwasm_std::{
        StdResult, HandleResponse, HumanAddr,
        CosmosMsg, WasmMsg, log, to_binary
    },
    snip20_impl::msg::HandleMsg as Snip20HandleMsg
};

use crate::state::Config;

pub struct MgmtKillswitch;

impl Killswitch for MgmtKillswitch {
    #[handle]
    fn set_status(
        level: ContractStatusLevel,
        reason: String,
        new_address: Option<HumanAddr>
    ) -> StdResult<HandleResponse> {
        // This checks for admin.
        set_status(deps, env, level, reason, new_address)?;

        let messages = if Config::is_prefunded(&deps.storage)? {
            vec![]
        } else {
            match level {
                // upon entering migration mode,
                // token admin is changed from "MGMT" to "MGMT's admin"
                // so that the token can be administrated manually
                ContractStatusLevel::Migrating => {
                    let token = Config::load_token(deps)?;
                    let admin = load_admin(deps)?;

                    // This is potentially dangerous because it assumes a function that is not a part of the SNIP-20 spec.
                    vec![CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: token.address,
                        callback_code_hash: token.code_hash,
                        send: vec![],
                        msg: to_binary(&Snip20HandleMsg::ChangeAdmin {
                            address: admin,
                            padding: None
                        })?
                    })]
                },
                _ => vec![]
            }
        };

        Ok(HandleResponse {
            messages,
            log: vec![
                log("action", "set_status"),
                log("level", level)
            ],
            data: None
        })
    }
}
