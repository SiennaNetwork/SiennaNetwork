use std::fmt::{Display, Formatter, Result};
use crate::{TokenType, TokenTypeAmount, TokenPair, TokenPairAmount};

impl<A: Display> Display for TokenType<A> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            TokenType::NativeToken { denom } => write!(f, "{}", denom),
            TokenType::CustomToken { contract_addr, .. } => write!(f, "{}", contract_addr),
        }
    }
}

impl<A: Display> Display for TokenTypeAmount<A> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Token type: {} \n Amount: {}", self.token, self.amount)
    }
}

impl<A: Display> Display for TokenPair<A> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Token 1: {} \n Token 2: {}", self.0, self.1)
    }
}

impl<A: Display + Clone> Display for TokenPairAmount<A> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f, "Token 1: {} {} \n Token 2: {} {}",
            self.pair.0, self.amount_0, self.pair.1, self.amount_1
        )
    }
}
