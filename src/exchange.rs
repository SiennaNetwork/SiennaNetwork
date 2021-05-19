use cosmwasm_std::{HumanAddr, StdResult, Api, CanonicalAddr};
use crate::token_pair::{TokenPair, TokenPairStored};
use fadroma_scrt_addr::{Humanize, Canonize};
use fadroma_scrt_callback::ContractInstance;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct Exchange {
    /// The pair that the contract manages.
    pub pair:    TokenPair,
    /// Address of the contract that manages the exchange.
    pub address: HumanAddr
}
impl Exchange {
    pub fn canonize (&self, api: &impl Api) -> StdResult<ExchangeStored> {
        Ok(ExchangeStored {
            pair: self.pair.to_stored(api)?,
            address: api.canonical_address(&self.address)?
        })
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ExchangeStored {
    pub pair:    TokenPairStored,
    pub address: CanonicalAddr
}
impl ExchangeStored {
    pub fn humanize (self, api: &impl Api) -> StdResult<Exchange> {
        Ok(Exchange {
            pair: self.pair.to_normal(api)?,
            address: api.human_address(&self.address)?
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug, Clone)]
pub struct ExchangeSettings<A> {
    pub swap_fee:      Fee,
    pub sienna_fee:    Fee,
    pub sienna_burner: Option<ContractInstance<A>>
}

#[deprecated(note="please use ExchangeSettings<CanonicalAddr> instead")]
pub type ExchangeSettingsStored = ExchangeSettings<CanonicalAddr>;

impl ExchangeSettings<HumanAddr> {
    pub fn canonize (&self, api: &impl Api) -> StdResult<ExchangeSettings<CanonicalAddr>> {
        Ok(ExchangeSettings {
            swap_fee:   self.swap_fee,
            sienna_fee: self.sienna_fee,
            sienna_burner: if let Some(info) = &self.sienna_burner { 
                Some(info.canonize(api)?) 
            } else {
                None
            }
        })
    }
}
impl ExchangeSettings<CanonicalAddr> {
    pub fn humanize (self, api: &impl Api) -> StdResult<ExchangeSettings<HumanAddr>> {
        Ok(ExchangeSettings {
            swap_fee:   self.swap_fee,
            sienna_fee: self.sienna_fee,
            sienna_burner: if let Some(info) = self.sienna_burner { 
                Some(info.humanize(api)?)
            } else {
                None
            }
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Copy, Debug)]
pub struct Fee {
    pub nom: u8,
    pub denom: u16
}

impl Fee {
    pub fn new(nom: u8, denom: u16) -> Self {
        Self {
            nom,
            denom
        }
    }
}
