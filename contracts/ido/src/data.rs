use cosmwasm_std::{HumanAddr, CanonicalAddr, Uint128, Api, StdResult};
use amm_shared::fadroma::address::{Canonize, Humanize};
use amm_shared::fadroma::callback::ContractInstance;
use amm_shared::TokenType;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config<A> {
    /// The token that is used to buy the sold SNIP20.
    pub input_token: TokenType<A>,
    /// The token that is being sold.
    pub sold_token: ContractInstance<A>,
    pub swap_constants: SwapConstants,
    /// The maximum number of participants allowed.
    pub max_seats: u32,
    /// The total amount that each participant is allowed to buy.
    pub max_allocation: Uint128,
    /// The minimum amount that each participant is allowed to buy.
    pub min_allocation: Uint128
}

/// Used when calculating the swap. These do not change
/// throughout the lifetime of the contract.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct SwapConstants {
    pub rate: Uint128,
    pub input_token_decimals: u8,
    pub sold_token_decimals: u8
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Account<A> {
    pub owner: A,
    pub total_bought: Uint128
}

impl Humanize<Account<HumanAddr>> for Account<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Account<HumanAddr>> {
        Ok(Account {
            owner: self.owner.humanize(api)?,
            total_bought: self.total_bought
        })
    }
}

impl Canonize<Account<CanonicalAddr>> for Account<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Account<CanonicalAddr>> {
        Ok(Account {
            owner: self.owner.canonize(api)?,
            total_bought: self.total_bought
        })
    }
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config{
            input_token: self.input_token.canonize(api)?,
            sold_token: self.sold_token.canonize(api)?,
            swap_constants: self.swap_constants.clone(),
            max_seats: self.max_seats,
            max_allocation: self.max_allocation,
            min_allocation: self.min_allocation
        })
    }
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config{
            input_token: self.input_token.humanize(api)?,
            sold_token: self.sold_token.humanize(api)?,
            swap_constants: self.swap_constants.clone(),
            max_seats: self.max_seats,
            max_allocation: self.max_allocation,
            min_allocation: self.min_allocation
        })
    }
}
