use cosmwasm_std::{HumanAddr, CanonicalAddr, Api, StdResult, Querier, Uint128, StdError, Env};
use schemars::JsonSchema;
use secret_toolkit::snip20;
use serde::{Deserialize, Serialize};
use std::fmt;

const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    CustomToken {
        contract_addr: HumanAddr,
        token_code_hash: String,
        //viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenTypeStored {
    CustomToken {
        contract_addr: CanonicalAddr,
        token_code_hash: String,
        //viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::NativeToken { denom } => write!(f, "{}", denom),
            TokenType::CustomToken { contract_addr, .. } => write!(f, "{}", contract_addr),
        }
    }
}

impl TokenType {
    pub fn to_stored(&self, api: &impl Api) -> StdResult<TokenTypeStored> {
        Ok(match self {
            TokenType::CustomToken { contract_addr, token_code_hash } => 
                TokenTypeStored::CustomToken { 
                    contract_addr: api.canonical_address(&contract_addr)?,
                    token_code_hash: token_code_hash.clone()
                },
            TokenType::NativeToken { denom } => 
                TokenTypeStored::NativeToken { 
                    denom: denom.clone()
                }
        })
    }

    pub fn is_native_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => true,
            TokenType::CustomToken { .. } => false,
        }
    }

    pub fn is_custom_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => false,
            TokenType::CustomToken { .. } => true,
        }
    }

    pub fn query_balance(
        &self,
        querier: &impl Querier,
        exchange_addr: HumanAddr,
        viewing_key: String
    ) -> StdResult<Uint128> {
        match self {
            TokenType::NativeToken { denom } => {
                let result = querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            },
            TokenType::CustomToken { contract_addr, token_code_hash } => {
                let result = snip20::balance_query(
                    querier,
                    exchange_addr.clone(),
                    viewing_key,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone()
                )?;

                Ok(result.amount)
            }
        }
    }

    pub fn assert_sent_native_token_balance(&self, env: &Env, amount: Uint128) -> StdResult<()> {
        if let TokenType::NativeToken { denom } = &self {
            return match env.message.sent_funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance missmatch between the argument and the transferred"))
                    }
                }
                None => {
                    if amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance missmatch between the argument and the transferred"))
                    }
                }
            }
        }

        Ok(())
    }
}

impl TokenTypeStored {
    pub fn to_normal(self, api: &impl Api) -> StdResult<TokenType> {
        Ok(match self {
            TokenTypeStored::CustomToken { contract_addr, token_code_hash } => 
                TokenType::CustomToken { 
                    contract_addr: api.human_address(&contract_addr)?,
                    token_code_hash
                },
            TokenTypeStored::NativeToken { denom } => 
                TokenType::NativeToken { 
                    denom
                }
        })
    }
}
