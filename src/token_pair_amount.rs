use cosmwasm_std::{Uint128, Env, StdResult};
use crate::{token_type::TokenType, token_pair::TokenPair};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenPairAmount {
    pub pair:     TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128
}

pub struct TokenPairAmountIterator<'a> {
    pair: &'a TokenPairAmount,
    index: u8
}

impl fmt::Display for TokenPairAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token 1: {} {} \n Token 2: {} {}",
            self.pair.0, self.amount_0, self.pair.1, self.amount_1
        )
    }
}

impl TokenPairAmount {
    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        self.pair.0.assert_sent_native_token_balance(env, self.amount_0)?;
        self.pair.1.assert_sent_native_token_balance(env, self.amount_1)?;

        Ok(())
    }
}

impl<'a> IntoIterator for &'a TokenPairAmount {
    type Item = (Uint128, &'a TokenType);
    type IntoIter = TokenPairAmountIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokenPairAmountIterator {
            pair: self,
            index: 0
        }
    }
}

impl<'a> Iterator for TokenPairAmountIterator<'a> {
    type Item = (Uint128, &'a TokenType);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some((self.pair.amount_0, &self.pair.pair.0)),
            1 => Some((self.pair.amount_1, &self.pair.pair.1)),
            _ => None
        };

        self.index += 1;

        result
    }
}
