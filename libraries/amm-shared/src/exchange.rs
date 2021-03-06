use fadroma::platform::{HumanAddr, StdResult, Api, CanonicalAddr, Canonize, Humanize, ContractLink};
use crate::token_pair::TokenPair;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct Exchange<A: Clone> {
    /// The pair that the contract manages.
    pub pair: TokenPair<A>,
    /// The contract that manages the exchange.
    pub contract: ContractLink<A>,
}

impl Canonize for Exchange<HumanAddr> {
    type Output = Exchange<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Exchange {
            pair: self.pair.canonize(api)?,
            contract: self.contract.canonize(api)?
        })
    }
}

impl Humanize for Exchange<CanonicalAddr> {
    type Output = Exchange<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Exchange {
            pair: self.pair.humanize(api)?,
            contract: self.contract.humanize(api)?
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct ExchangeSettings<A> {
    pub swap_fee: Fee,
    pub sienna_fee: Fee,
    pub sienna_burner: Option<A>,
}

impl ExchangeSettings<HumanAddr> {
    pub fn canonize(&self, api: &impl Api) -> StdResult<ExchangeSettings<CanonicalAddr>> {
        Ok(ExchangeSettings {
            swap_fee: self.swap_fee,
            sienna_fee: self.sienna_fee,
            sienna_burner: if let Some(info) = &self.sienna_burner {
                Some(info.canonize(api)?)
            } else {
                None
            },
        })
    }
}

impl ExchangeSettings<CanonicalAddr> {
    pub fn humanize(self, api: &impl Api) -> StdResult<ExchangeSettings<HumanAddr>> {
        Ok(ExchangeSettings {
            swap_fee: self.swap_fee,
            sienna_fee: self.sienna_fee,
            sienna_burner: if let Some(info) = self.sienna_burner {
                Some(info.humanize(api)?)
            } else {
                None
            },
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Copy, Debug)]
pub struct Fee {
    pub nom: u8,
    pub denom: u16,
}

impl Fee {
    pub fn new(nom: u8, denom: u16) -> Self {
        Self { nom, denom }
    }
}
