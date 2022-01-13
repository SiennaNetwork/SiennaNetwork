use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Binary, Storage, Api, Querier, Extern,
            Uint128,StdResult, StdError, HumanAddr,
            HandleResponse
        },
        secret_toolkit::snip20,
        storage::{ns_load, ns_save},
        Decimal256, Uint256, BLOCK_SIZE
    },
    interfaces::market::VIEWING_KEY
};
use crate::{GlobalData, Config, Contracts};

// TODO: Move to state.rs
// *********************************************************************************************
pub struct Account(Binary);

impl Account {
    const NS_BALANCES: &'static [u8] = b"balances";

    pub fn get_balance(&self, storage: &impl Storage) -> StdResult<Uint128> {
        let result: Option<Uint128> = ns_load(
            storage,
            Self::NS_BALANCES,
            self.0.as_slice()
        )?;

        Ok(result.unwrap_or_default())
    }

    pub fn add_balance(&self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        let account_balance = self.get_balance(storage)?;

        if let Some(new_balance) = account_balance.0.checked_add(amount.0) {
            self.set_balance(storage, Uint128(new_balance))
        } else {
            Err(StdError::generic_err(
                "This deposit would overflow your balance",
            ))
        }
    }

    pub fn subtract_balance(&self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        let account_balance = self.get_balance(storage)?;

        if let Some(new_balance) = account_balance.0.checked_sub(amount.0) {
            self.set_balance(storage, Uint128(new_balance))
        } else {
            Err(StdError::generic_err(format!(
                "insufficient funds: balance={}, required={}",
                account_balance, amount
            )))
        }
    }

    #[inline]
    fn set_balance(&self, storage: &mut impl Storage, amount: Uint128) -> StdResult<()> {
        ns_save(storage, Self::NS_BALANCES, self.0.as_slice(), &amount)
    }
}
// *********************************************************************************************

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S,A,Q>,
    self_addr: HumanAddr,
    depositor: HumanAddr,
    amount: Uint128
) -> StdResult<HandleResponse> {
    // TODO: accrue_interest
    // TODO: https://github.com/compound-finance/compound-protocol/blob/4a8648ec0364d24c4ecfc7d6cae254f55030d65f/contracts/CToken.sol#L505-L507

    let exchange_rate = calc_exchange_rate(deps, self_addr)?;
    let mint_amount: Uint128 = Uint256::from(amount)
        .decimal_div(exchange_rate)?
        .clamp_u128()?
        .into();

    GlobalData::increase_total_supply(&mut deps.storage, mint_amount);

    // TODO: increase account balance.

    Ok(HandleResponse::default())
}

pub fn calc_exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    self_addr: HumanAddr
) -> StdResult<Decimal256> {
    let total_supply = GlobalData::load_total_supply(&deps.storage)?;

    if total_supply.is_zero() {
        let config = Config::load(deps)?;

        return Ok(config.initial_exchange_rate);
    }

    let underlying_asset = Contracts::load_underlying(deps)?;

    let balance = snip20::balance_query(
        &deps.querier,
        self_addr,
        VIEWING_KEY.to_string(),
        BLOCK_SIZE,
        underlying_asset.code_hash,
        underlying_asset.address,
    )?.amount;

    let total_borrows = GlobalData::load_total_borrows(&deps.storage)?.0;
    let total_reserves = GlobalData::load_total_reserves(&deps.storage)?.0;

    let total_minus_reserves = balance.0.checked_add(total_borrows)
        .and_then(|x|
            x.checked_sub(total_reserves)
        )
        .ok_or_else(||
            StdError::generic_err("Math overflow while calculating exchange rate.")
        )?;

    Decimal256::from_ratio(total_minus_reserves, total_supply.0)
}
