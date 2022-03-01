use std::borrow::Borrow;

use amm_shared::fadroma::{
    platform::{
        Api, CanonicalAddr, Canonize, ContractLink, Extern, HumanAddr, Humanize, Querier, StdError,
        StdResult, Storage, Uint128,
    },
    storage::{load, save, ns_load, ns_save},
    ViewingKey,
};
use amm_shared::{msg::ido::SaleType, TokenType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const KEY_CONTRACT_ADDR: &[u8] = b"this_contract";
const KEY_VIEWING_KEY: &[u8] = b"viewing_key";
const TOTAL_PRE_LOCK_AMOUNT: &[u8] = b"total_pre_lock_amount";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config<A> {
    /// The token that is used to buy the sold SNIP20.
    pub input_token: TokenType<A>,
    /// The token that is being sold.
    pub sold_token: ContractLink<A>,
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
    /// Configuration of sale type options (PreLockOnly, SwapOnly, PreLockAndSwap)
    pub sale_type: SaleType,
    /// Info of the launchpad contract that will post whitelisted
    /// addresses to IDO contract after it has been initialized
    pub launchpad: Option<ContractLink<A>>,
    /// The Option<> lets us know if this contract is active,
    /// contract only becomes active once the sold_token funds
    /// are sent to it:
    /// Amount has to be exact to max_seats * max_allocation
    ///
    /// This also means that the sold_token cannot be minted directly to
    /// this contract, it will have to be minted to the owner and then
    /// the owner will have to send funds to IDO contract. This limitation
    /// is due the mint message not having the means to sent the receive
    /// callback to IDO contract.
    pub schedule: Option<SaleSchedule>,
}

pub(crate) fn load_contract_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<HumanAddr> {
    let address: CanonicalAddr = load(&deps.storage, KEY_CONTRACT_ADDR)?.unwrap();

    address.humanize(&deps.api)
}

pub(crate) fn save_contract_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr,
) -> StdResult<()> {
    let address = address.canonize(&deps.api)?;

    save(&mut deps.storage, KEY_CONTRACT_ADDR, &address)
}

pub(crate) fn increment_total_pre_lock_amount<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    add_amount: u128,
) -> StdResult<()> {
    let mut amount: Uint128 = load_total_pre_lock_amount(&deps)?;

    amount = amount
        .u128()
        .checked_add(add_amount)
        .ok_or_else(|| StdError::generic_err("Upper bound overflow detected."))?
        .into();

    save(&mut deps.storage, TOTAL_PRE_LOCK_AMOUNT, &amount)
}

pub(crate) fn load_total_pre_lock_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint128> {
    let amount: Uint128 = load(&deps.storage, TOTAL_PRE_LOCK_AMOUNT)?.unwrap_or_default();

    Ok(amount)
}

pub(crate) fn load_viewing_key(storage: &impl Storage) -> StdResult<ViewingKey> {
    let vk: ViewingKey = load(storage, KEY_VIEWING_KEY)?.unwrap();

    Ok(vk)
}

pub(crate) fn save_viewing_key(storage: &mut impl Storage, vk: &ViewingKey) -> StdResult<()> {
    save(storage, KEY_VIEWING_KEY, vk)
}

impl<A> Config<A> {
    const KEY: &'static[u8] = b"config";

    pub fn load_self<S: Storage, T: Api, Q: Querier>(
        deps: &Extern<S, T, Q>,
    ) -> StdResult<Config<HumanAddr>> {
        let result = load(&deps.storage, Self::KEY)?;
        let result =
            result.ok_or_else(|| StdError::generic_err("Config doesn't exist in storage."))?;

        Ok(result)
    }

    /// Check if the contract is active
    pub fn is_active(&self) -> bool {
        self.schedule.is_some()
    }

    /// Check if tokens can be swaped
    pub fn is_swapable(&self, time: u64) -> StdResult<()> {
        if let Some(schedule) = self.schedule {
            if !schedule.has_started(time) {
                return Err(StdError::generic_err(format!(
                    "Sale hasn't started yet, come back in {} seconds",
                    schedule.start - time
                )));
            }
            if schedule.has_ended(time) {
                return Err(StdError::generic_err("Sale has ended"));
            }

            return Ok(());
        }

        Err(StdError::generic_err("Contract is not yet active"))
    }

    /// Check if the contract can be refunded
    pub fn is_refundable(&self, time: u64) -> StdResult<()> {
        if let Some(schedule) = self.schedule {
            if !schedule.has_ended(time) {
                return Err(StdError::generic_err(format!(
                    "Sale hasn't finished yet, come back in {} seconds",
                    schedule.end - time
                )));
            }

            return Ok(());
        }

        Err(StdError::generic_err("Contract is not yet active"))
    }

    /// Returns the total amount required for the sale to take place.
    /// Overflow checking needs to be performed at init time.
    pub fn total_allocation(&self) -> Uint128 {
        Uint128(self.max_allocation.u128() * self.max_seats as u128)
    }
}

impl Config<HumanAddr> {
    pub fn save<S: Storage, T: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, T, Q>
    ) -> StdResult<()> {
        save(&mut deps.storage, Self::KEY, self)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, Debug)]
pub(crate) struct SaleSchedule {
    /// Time when the sale will start
    pub start: u64,
    /// Time when the sale will end
    pub end: u64,
}

impl SaleSchedule {
    pub fn new(now: u64, start: Option<u64>, end: u64) -> StdResult<Self> {
        let start = start.unwrap_or(now);
        if start >= end {
            return Err(StdError::generic_err(format!(
                "End time of the sale has to be after {}.",
                start
            )));
        }
        if end <= now {
            return Err(StdError::generic_err(
                "End time of the sale must be any time after now.",
            ));
        }

        Ok(Self { start, end })
    }

    /// Check if the contract has started
    pub fn has_started(&self, time: u64) -> bool {
        self.start <= time
    }

    /// Check if the contract has ended
    pub fn has_ended(&self, time: u64) -> bool {
        time >= self.end
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
pub struct Account {
    pub owner: HumanAddr,
    pub total_bought: Uint128,
    pub pre_lock_amount: Uint128,
}

impl Account {
    const NS: &'static[u8] = b"accounts";

    pub fn new(owner: HumanAddr) -> Account {
        Account {
            owner,
            total_bought: Uint128::zero(),
            pre_lock_amount: Uint128::zero(),
        }
    }

    /// Load the account if its whitelisted
    pub fn load_self<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr,
    ) -> StdResult<Account> {
        let address = address.canonize(&deps.api)?;

        ns_load(&deps.storage, Self::NS, address.as_slice())?
            .ok_or_else(|| StdError::generic_err("This address is not whitelisted."))
    }

    pub fn save<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>
    ) -> StdResult<()> {
        let address = self.owner.borrow().canonize(&deps.api)?;
        ns_save(&mut deps.storage, Self::NS, address.as_slice(), self)
    }
}

impl Canonize for Config<HumanAddr> {
    type Output = Config<CanonicalAddr>;

    fn canonize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Config {
            input_token: self.input_token.canonize(api)?,
            sold_token: self.sold_token.canonize(api)?,
            swap_constants: self.swap_constants,
            taken_seats: self.taken_seats,
            max_seats: self.max_seats,
            max_allocation: self.max_allocation,
            min_allocation: self.min_allocation,
            sale_type: self.sale_type,
            launchpad: self.launchpad.canonize(api)?,
            schedule: self.schedule,
        })
    }
}

impl Humanize for Config<CanonicalAddr> {
    type Output = Config<HumanAddr>;

    fn humanize(self, api: &impl Api) -> StdResult<Self::Output> {
        Ok(Config {
            input_token: self.input_token.humanize(api)?,
            sold_token: self.sold_token.humanize(api)?,
            swap_constants: self.swap_constants,
            taken_seats: self.taken_seats,
            max_seats: self.max_seats,
            max_allocation: self.max_allocation,
            min_allocation: self.min_allocation,
            sale_type: self.sale_type,
            launchpad: self.launchpad.humanize(api)?,
            schedule: self.schedule,
        })
    }
}
