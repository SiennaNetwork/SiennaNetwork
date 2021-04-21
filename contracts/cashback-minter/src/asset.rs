/// This file is based on https://github.com/enigmampc/SecretSwap/blob/5bb072d01b21899344b018ab86bb9a4f3fa1d848/packages/secretswap/src/asset.rs
use cosmwasm_std::{
    to_binary, Api, BankMsg, Coin, CosmosMsg, Env, Extern, HumanAddr, Querier, StdError, StdResult,
    Storage, Uint128, WasmMsg,
};
use schemars::JsonSchema;
use secret_toolkit::snip20::HandleMsg;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

impl Asset {
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    pub fn into_msg<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
        sender: HumanAddr,
        recipient: HumanAddr,
    ) -> StdResult<CosmosMsg> {
        let amount = self.amount;

        match &self.info {
            AssetInfo::Token {
                contract_addr,
                token_code_hash,
                ..
            } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone(),

                callback_code_hash: token_code_hash.clone(),

                msg: to_binary(&HandleMsg::Send {
                    recipient,
                    amount,
                    padding: None,
                    msg: None,
                })?,
                send: vec![],
            })),
            AssetInfo::NativeToken { .. } => Ok(CosmosMsg::Bank(BankMsg::Send {
                from_address: sender,
                to_address: recipient,
                amount: vec![self.deduct_tax(deps)?],
            })),
        }
    }

    pub fn compute_tax<S: Storage, A: Api, Q: Querier>(
        &self,
        _deps: &Extern<S, A, Q>,
    ) -> StdResult<Uint128> {
        Ok(Uint128::zero())
    }

    pub fn deduct_tax<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<Coin> {
        let amount = self.amount;
        if let AssetInfo::NativeToken { denom } = &self.info {
            Ok(Coin {
                denom: denom.to_string(),
                amount: (amount - self.compute_tax(deps)?)?,
            })
        } else {
            Err(StdError::generic_err("cannot deduct tax from token asset"))
        }
    }

    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        if let AssetInfo::NativeToken { denom } = &self.info {
            match env.message.sent_funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if self.amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
                None => {
                    if self.amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token {
        contract_addr: HumanAddr,
        token_code_hash: String,
        viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfo::NativeToken { denom } => write!(f, "{}", denom),
            AssetInfo::Token { contract_addr, .. } => write!(f, "{}", contract_addr),
        }
    }
}

impl AssetInfo {
    pub fn is_native_token(&self) -> bool {
        match self {
            AssetInfo::NativeToken { .. } => true,
            AssetInfo::Token { .. } => false,
        }
    }

    pub fn equal(&self, asset: &AssetInfo) -> bool {
        match self {
            AssetInfo::Token { contract_addr, .. } => {
                let self_contract_addr = contract_addr;
                match asset {
                    AssetInfo::Token { contract_addr, .. } => self_contract_addr == contract_addr,
                    AssetInfo::NativeToken { .. } => false,
                }
            }
            AssetInfo::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfo::Token { .. } => false,
                    AssetInfo::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }
}
