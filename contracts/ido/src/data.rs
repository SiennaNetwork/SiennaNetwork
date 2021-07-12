use amm_shared::TokenType;
use fadroma::scrt::addr::{Canonize, Humanize};
use fadroma::scrt::callback::ContractInstance;
use fadroma::scrt::cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};
use fadroma::scrt::storage::Storable;
use fadroma::scrt::utils::viewing_key::ViewingKey;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config<A> {
    /// The token that is used to buy the sold SNIP20.
    pub input_token: TokenType<A>,
    /// The token that is being sold.
    pub sold_token: ContractInstance<A>,
    /// Token constants
    pub swap_constants: SwapConstants,
    /// Number of participants currently
    pub taken_seats: u32,
    /// The maximum number of participants allowed.
    pub max_seats: u32,
    /// The total amount that each participant is allowed to buy.
    pub max_allocation: Uint128,
    /// The minimum amount that each participant is allowed to buy.
    pub min_allocation: Uint128,
    /// Time when the sale will start (if None, it will start immediately)
    pub start_time: u64,
    /// Time when the sale will end
    pub end_time: Option<u64>,
    /// Viewkey for sold token
    pub viewing_key: ViewingKey,
}

impl<A> Storable for Config<A>
where
    A: Serialize + serde::de::DeserializeOwned,
{
    fn namespace() -> Vec<u8> {
        b"config".to_vec()
    }
    /// Setting the empty key because config is only one
    fn key(&self) -> StdResult<Vec<u8>> {
        Ok(Vec::new())
    }
}

impl<A> Config<A> {
    pub fn load_self<S: Storage, T: Api, Q: Querier>(
        deps: &Extern<S, T, Q>,
    ) -> StdResult<Config<HumanAddr>> {
        let result = Config::<HumanAddr>::load(deps, b"")?;
        let result = result.ok_or(StdError::generic_err("Config doesn't exist in storage."))?;

        Ok(result)
    }

    /// Check if the contract has started
    pub fn has_started(&self, time: u64) -> bool {
        self.start_time <= time
    }

    /// Check if the contract has ended
    pub fn has_ended(&self, time: u64) -> bool {
        if let Some(end) = self.end_time {
            time >= end
        } else {
            false
        }
    }

    /// Check if tokens can be swaped
    pub fn is_swapable(&self, time: u64) -> StdResult<()> {
        if !self.has_started(time) {
            return Err(StdError::generic_err(format!(
                "Sale hasn't started yet, come back in {} seconds",
                self.start_time - time
            )));
        }

        if self.has_ended(time) {
            return Err(StdError::generic_err("Sale has ended"));
        }

        Ok(())
    }

    /// Check if the contract can be refunded
    pub fn is_refundable(&self, time: u64) -> StdResult<()> {
        if let Some(end) = self.end_time {
            if time <= end {
                Err(StdError::generic_err(format!(
                    "Sale hasn't finished yet, come back in {} seconds",
                    end - time
                )))
            } else {
                Ok(())
            }
        } else {
            Err(StdError::generic_err(
                "Cannot refund, sale is still active and will last until all the funds are swapped",
            ))
        }
    }
}

/// Used when calculating the swap. These do not change
/// throughout the lifetime of the contract.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct SwapConstants {
    pub rate: Uint128,
    pub input_token_decimals: u8,
    pub sold_token_decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Account<A> {
    pub owner: A,
    pub total_bought: Uint128,
}

impl Storable for Account<CanonicalAddr> {
    /// Global accounts namespace
    fn namespace() -> Vec<u8> {
        b"accounts".to_vec()
    }

    /// Individual account key
    fn key(&self) -> StdResult<Vec<u8>> {
        Ok(self.owner.as_slice().to_vec())
    }
}

impl Account<CanonicalAddr> {
    pub fn new(address: &CanonicalAddr) -> Account<CanonicalAddr> {
        Account {
            owner: address.clone(),
            total_bought: 0_u128.into(),
        }
    }

    /// Load the account if its whitelisted
    pub fn load_self<S: Storage, T: Api, Q: Querier>(
        deps: &Extern<S, T, Q>,
        address: &HumanAddr,
    ) -> StdResult<Account<CanonicalAddr>> {
        let canonical_address = address.canonize(&deps.api)?;

        Self::load(&deps, canonical_address.as_slice())?
            .ok_or(StdError::generic_err("This address is not whitelisted."))
    }
}

impl Humanize<Account<HumanAddr>> for Account<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Account<HumanAddr>> {
        Ok(Account {
            owner: self.owner.humanize(api)?,
            total_bought: self.total_bought,
        })
    }
}

impl Canonize<Account<CanonicalAddr>> for Account<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Account<CanonicalAddr>> {
        Ok(Account {
            owner: self.owner.canonize(api)?,
            total_bought: self.total_bought,
        })
    }
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            input_token: self.input_token.canonize(api)?,
            sold_token: self.sold_token.canonize(api)?,
            swap_constants: self.swap_constants.clone(),
            taken_seats: self.taken_seats,
            max_seats: self.max_seats,
            max_allocation: self.max_allocation,
            min_allocation: self.min_allocation,
            start_time: self.start_time,
            end_time: self.end_time,
            viewing_key: self.viewing_key.clone(),
        })
    }
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            input_token: self.input_token.humanize(api)?,
            sold_token: self.sold_token.humanize(api)?,
            swap_constants: self.swap_constants.clone(),
            taken_seats: self.taken_seats,
            max_seats: self.max_seats,
            max_allocation: self.max_allocation,
            min_allocation: self.min_allocation,
            start_time: self.start_time,
            end_time: self.end_time,
            viewing_key: self.viewing_key.clone(),
        })
    }
}
