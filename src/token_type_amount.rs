use cosmwasm_std::{Uint128, Env, StdResult};
use crate::token_type::TokenType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenTypeAmount {
    pub token: TokenType,
    pub amount: Uint128
}

impl fmt::Display for TokenTypeAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token type: {} \n Amount: {}",
            self.token, self.amount
        )
    }
}

impl TokenTypeAmount {
    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        self.token.assert_sent_native_token_balance(env, self.amount)
    }
}
