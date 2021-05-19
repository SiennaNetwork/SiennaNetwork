use cosmwasm_std::{Uint128, Env, StdResult};
use crate::token_type::TokenType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenTypeAmount<A> {
    pub token: TokenType<A>,
    pub amount: Uint128
}

impl Display for TokenTypeAmount<HumanAddr> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Token type: {} \n Amount: {}", self.token, self.amount)
    }
}

impl<A> TokenTypeAmount<A> {
    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        self.token.assert_sent_native_token_balance(env, self.amount)
    }
}
