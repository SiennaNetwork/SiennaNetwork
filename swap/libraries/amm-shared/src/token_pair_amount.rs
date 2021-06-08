use cosmwasm_std::{Uint128, Env, StdResult};
use crate::{token_type::TokenType, token_pair::TokenPair};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenPairAmount<A: Clone> {
    pub pair:     TokenPair<A>,
    pub amount_0: Uint128,
    pub amount_1: Uint128
}

impl<A: Clone> TokenPairAmount<A> {
    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        self.pair.0.assert_sent_native_token_balance(env, self.amount_0)?;
        self.pair.1.assert_sent_native_token_balance(env, self.amount_1)?;

        Ok(())
    }
}

impl<'a, A: Clone> IntoIterator for &'a TokenPairAmount<A> {
    type Item = (Uint128, &'a TokenType<A>);
    type IntoIter = TokenPairAmountIterator<'a, A>;
    fn into_iter(self) -> Self::IntoIter {
        TokenPairAmountIterator {
            pair: self,
            index: 0
        }
    }
}

pub struct TokenPairAmountIterator<'a, A: Clone> {
    pair: &'a TokenPairAmount<A>,
    index: u8
}

impl<'a, A: Clone> Iterator for TokenPairAmountIterator<'a, A> {
    type Item = (Uint128, &'a TokenType<A>);
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
