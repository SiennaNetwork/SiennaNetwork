use cosmwasm_std::{Api, StdResult, Querier, HumanAddr, Uint128};
use crate::token_type::{TokenType, TokenTypeStored};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::fmt;

#[derive(Clone, Debug, JsonSchema)]
pub struct TokenPair(pub TokenType, pub TokenType);

#[derive(Clone, Debug, JsonSchema)]
pub struct TokenPairStored(pub TokenTypeStored, pub TokenTypeStored);

pub struct TokenPairIterator<'a> {
    pair: &'a TokenPair,
    index: u8
}

impl fmt::Display for TokenPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token 1: {} \n Token 2: {}",
            self.0, self.1
        )
    }
}

impl TokenPair {
    pub fn to_stored(&self, api: &impl Api) -> StdResult<TokenPairStored> {
        Ok(TokenPairStored(self.0.to_stored(api)?, self.1.to_stored(api)?))
    }

    /// Returns the balance for each token in the pair. The order of the balances in returned array
    /// correspond to the token order in the pair i.e `[ self.0 balance, self.1 balance ]`.
    pub fn query_balances(
        &self,
        querier: &impl Querier,
        exchange_addr: HumanAddr,
        viewing_key: String
    ) -> StdResult<[Uint128; 2]> {
        let amount_0 = self.0.query_balance(querier, exchange_addr.clone(), viewing_key.clone())?;
        let amount_1 = self.1.query_balance(querier, exchange_addr, viewing_key)?;

        // order is important
        Ok([amount_0, amount_1])
    }

    /// Returns `true` if one of the token types in the pair is the same as the argument.
    pub fn contains(&self, token: &TokenType) -> bool {
        self.0 == *token || self.1 == *token
    }

    /// Returns the index of the stored token type (0 or 1) that matches the argument.
    /// Returns `None` if there are no matches.
    pub fn get_token_index(&self, token: &TokenType) -> Option<usize> {
        if self.0 == *token {
            return Some(0);
        } else if self.1 == *token {
            return Some(1);
        }

        None
    }

    pub fn get_token(&self, index: usize) -> Option<&TokenType> {
        match index {
            0 => Some(&self.0),
            1 => Some(&self.1),
            _ => None
        }
    }
}

impl PartialEq for TokenPair {
    fn eq(&self, other: &TokenPair) -> bool {
        (self.0 == other.0 || self.0 == other.1) && (self.1 == other.0 || self.1 == other.1)
    }
}

impl TokenPairStored {
    pub fn to_normal(self, api: &impl Api) -> StdResult<TokenPair> {
        Ok(TokenPair(self.0.to_normal(api)?, self.1.to_normal(api)?))
    }
}

impl<'a> IntoIterator for &'a TokenPair {
    type Item = &'a TokenType;
    type IntoIter = TokenPairIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokenPairIterator {
            pair: self,
            index: 0
        }
    }
}

impl<'a> Iterator for TokenPairIterator<'a> {
    type Item = &'a TokenType;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(&self.pair.0),
            1 => Some(&self.pair.1),
            _ => None
        };

        self.index += 1;

        result
    }
}

// These are only used for serde, because it doesn't work with struct tuples.
#[derive(Serialize, Deserialize)]
struct TokenPairSerde {
    token_0: TokenType,
    token_1: TokenType,
}

impl Serialize for TokenPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TokenPairSerde { token_0: self.0.clone(), token_1: self.1.clone() }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TokenPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|TokenPairSerde { token_0, token_1 }| TokenPair(token_0, token_1))
    }
}

#[derive(Serialize, Deserialize)]
struct TokenPairStoredSerde {
    token_0: TokenTypeStored,
    token_1: TokenTypeStored,
}

impl Serialize for TokenPairStored {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TokenPairStoredSerde { token_0: self.0.clone(), token_1: self.1.clone() }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TokenPairStored {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
            .map(|TokenPairStoredSerde { token_0, token_1 }| TokenPairStored(token_0, token_1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_pair_equality() {
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: "address".into(),
                token_code_hash: "hash".into()
            },
            TokenType::NativeToken {
                denom: "denom".into()
            }
        );

        let pair2 = TokenPair(pair.1.clone(), pair.0.clone());

        assert_eq!(pair, pair.clone());
        assert_eq!(pair2, pair2.clone());
        assert_eq!(pair, pair2);
    }
}
