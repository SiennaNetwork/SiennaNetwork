use cosmwasm_std::{HumanAddr, CanonicalAddr, Api, StdResult, Querier, Uint128, StdError, Env};
use fadroma_scrt_addr::{Canonize, Humanize};
use schemars::JsonSchema;
use secret_toolkit::snip20;
use serde::{Deserialize, Serialize};

const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType<A> {
    CustomToken {
        contract_addr: A,
        token_code_hash: String,
        //viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}
impl Canonize<TokenType<CanonicalAddr>> for TokenType<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<TokenType<CanonicalAddr>> {
        Ok(match self {
            Self::CustomToken { contract_addr, token_code_hash } => TokenType::CustomToken {
                contract_addr: contract_addr.canonize(api)?,
                token_code_hash: token_code_hash.clone()
            },
            Self::NativeToken { denom } => TokenType::NativeToken {
                denom: denom.clone()
            }
        })
    }
}
impl Humanize<TokenType<HumanAddr>> for TokenType<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<TokenType<HumanAddr>> {
        Ok(match self {
            Self::CustomToken { contract_addr, token_code_hash } => TokenType::CustomToken {
                contract_addr: contract_addr.humanize(api)?,
                token_code_hash: token_code_hash.clone()
            },
            Self::NativeToken { denom } => TokenType::NativeToken {
                denom: denom.clone()
            }
        })
    }
}

#[deprecated(note="please use TokenType<CanonicalAddr> instead")]
pub type TokenTypeStored = TokenType<CanonicalAddr>;

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

impl TokenType<HumanAddr> {
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
}
