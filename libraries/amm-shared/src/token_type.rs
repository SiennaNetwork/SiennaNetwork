use std::convert::TryInto;
use fadroma::platform::{
    HumanAddr, CanonicalAddr, Api, StdResult,
    Querier, Uint128, StdError, Env, CosmosMsg,
    WasmMsg, BankMsg, Coin, ContractLink, to_binary,
    secret_toolkit::snip20,
    Canonize, Humanize
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType<A> {
    CustomToken {
        contract_addr: A,
        token_code_hash: String
    },
    NativeToken {
        denom: String
    },
}

impl<A: PartialEq> PartialEq for TokenType<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::CustomToken { contract_addr: l_contract_addr, .. },
                Self::CustomToken { contract_addr: r_contract_addr, .. }
            ) => l_contract_addr == r_contract_addr,
            (
                Self::NativeToken { denom: l_denom },
                Self::NativeToken { denom: r_denom }
            ) => l_denom == r_denom,
            _ => false
        }
    }
}

impl Canonize<TokenType<CanonicalAddr>> for TokenType<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<TokenType<CanonicalAddr>> {
        Ok(match self {
            Self::CustomToken {
                contract_addr,
                token_code_hash,
            } => TokenType::CustomToken {
                contract_addr: contract_addr.canonize(api)?,
                token_code_hash: token_code_hash.clone(),
            },
            Self::NativeToken { denom } => TokenType::NativeToken {
                denom: denom.clone(),
            },
        })
    }
}
impl Humanize<TokenType<HumanAddr>> for TokenType<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<TokenType<HumanAddr>> {
        Ok(match self {
            Self::CustomToken {
                contract_addr,
                token_code_hash,
            } => TokenType::CustomToken {
                contract_addr: contract_addr.humanize(api)?,
                token_code_hash: token_code_hash.clone(),
            },
            Self::NativeToken { denom } => TokenType::NativeToken {
                denom: denom.clone(),
            },
        })
    }
}

impl<A: Clone> TokenType<A> {
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
    pub fn assert_sent_native_token_balance(&self, env: &Env, amount: Uint128) -> StdResult<()> {
        if let TokenType::NativeToken { denom } = &self {
            return match env.message.sent_funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
                None => {
                    if amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
            };
        }

        Ok(())
    }
}

impl TokenType<HumanAddr> {
    pub fn query_balance(
        &self,
        querier: &impl Querier,
        exchange_addr: HumanAddr,
        viewing_key: String,
    ) -> StdResult<Uint128> {
        match self {
            TokenType::NativeToken { denom } => {
                let result = querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                let result = snip20::balance_query(
                    querier,
                    exchange_addr,
                    viewing_key,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?;
                Ok(result.amount)
            }
        }
    }

    pub fn create_send_msg (
        &self,
        sender: HumanAddr,
        recipient: HumanAddr,
        amount: Uint128
    ) -> StdResult<CosmosMsg> {
        let msg = match self {
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
                        denom: denom.clone(),
                        amount
                    }],
                })
            }
        };
    
        Ok(msg)
    }
}

impl From<ContractLink<HumanAddr>> for TokenType<HumanAddr> {
    fn from(contract: ContractLink<HumanAddr>) -> Self {
        TokenType::CustomToken {
            contract_addr: contract.address,
            token_code_hash: contract.code_hash
        }
    }
}

impl TryInto<ContractLink<HumanAddr>> for &TokenType<HumanAddr> {
    type Error = StdError;

    fn try_into(self) -> Result<ContractLink<HumanAddr>, Self::Error> {
        match self {
            TokenType::CustomToken { contract_addr, token_code_hash } => {
                Ok(ContractLink {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone()
                })
            },
            _ => Err(StdError::generic_err("Cannot convert TokenType::NativeToken to ContractLink."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let token_0 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_0"),
            token_code_hash: "code_hash_0".into()
        };

        let token_1: TokenType<HumanAddr> = TokenType::NativeToken {
            denom: "denom_0".into()
        };

        assert!(token_0 != token_1);
        assert!(token_1 != token_0);

        let token_0 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_0"),
            token_code_hash: "code_hash_0".into()
        };

        let token_1 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_0"),
            token_code_hash: "code_hash_1".into()
        };

        assert!(token_0 == token_1);
        assert!(token_1 == token_0);

        let token_0 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_0"),
            token_code_hash: "code_hash_0".into()
        };

        let token_1 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_0"),
            token_code_hash: "code_hash_0".into()
        };

        assert!(token_0 == token_1);
        assert!(token_1 == token_0);

        let token_0 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_1"),
            token_code_hash: "code_hash_0".into()
        };

        let token_1 = TokenType::CustomToken {
            contract_addr: HumanAddr::from("addr_0"),
            token_code_hash: "code_hash_0".into()
        };

        assert!(token_0 != token_1);
        assert!(token_1 != token_0);
    }
}
