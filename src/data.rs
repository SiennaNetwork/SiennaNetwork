use cosmwasm_std::{
    Api, CanonicalAddr, HumanAddr, Querier, StdResult, Env,
    Uint128, CosmosMsg, WasmMsg, BankMsg, Coin, to_binary, StdError
};
use cosmwasm_utils::{ContractInfo, ContractInfoStored};
use crate::token_type::TokenType;
use schemars::JsonSchema;
use secret_toolkit::snip20;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::fmt;

const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8
}

pub fn create_send_msg <A> (
    token:     &TokenType<A>,
    sender:    HumanAddr,
    recipient: HumanAddr,
    amount:    Uint128
) -> StdResult<CosmosMsg> {
    let msg = match token {
        TokenType::CustomToken { contract_addr, token_code_hash } => {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone(),
                callback_code_hash: token_code_hash.to_string(),
                msg: to_binary(&snip20::HandleMsg::Send {
                    recipient,
                    amount,
                    padding: None,
                    msg: None,
                })?,
                send: vec![]
            })
        },
        TokenType::NativeToken { denom } => {            
            CosmosMsg::Bank(BankMsg::Send {
                from_address: sender,
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.to_string(),
                    amount: amount
                }],
            })
        }
    };

    Ok(msg)
}
