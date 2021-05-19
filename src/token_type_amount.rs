use cosmwasm_std::{Uint128, Env, StdResult};
use crate::token_type::TokenType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenTypeAmount<A> {
    pub token: TokenType<A>,
    pub amount: Uint128
}

impl<A: Clone> TokenTypeAmount<A> {
    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        self.token.assert_sent_native_token_balance(env, self.amount)
    }
}
