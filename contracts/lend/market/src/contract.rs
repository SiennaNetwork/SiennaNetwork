mod checks;
mod state;
mod token;
mod ops;

use std::convert::TryFrom;

use lend_shared::{
    fadroma::{
        admin,
        admin::{assert_admin, Admin},
        cosmwasm_std,
        cosmwasm_std::{
            Api, Binary, Extern, HandleResponse, HumanAddr, Env,
            InitResponse, Querier, StdError, StdResult, Storage,
            log
        },
        derive_contract::*,
        require_admin,
        secret_toolkit::snip20,
        snip20_impl::msg as snip20_msg,
        Uint256, Decimal256, Permit,
        Uint128, BLOCK_SIZE, ContractLink,
        from_binary, to_binary
    },
    interfaces::{
        interest_model::query_borrow_rate,
        market::{
            ReceiverCallbackMsg, AccountInfo,
            StateResponse, ConfigResponse
        },
        overseer::{OverseerPermissions, query_can_transfer},
    },
};

pub const VIEWING_KEY: &str = "SiennaLend"; // TODO: This shouldn't be hardcoded.
pub const MAX_RESERVE_FACTOR: Decimal256 = Decimal256::one();
// TODO: proper value here
pub const MAX_BORROW_RATE: Decimal256 = Decimal256::one();

use state::{
    Config, Contracts, Global,
    Account, TotalBorrows
};

#[contract_impl(path = "lend_shared::interfaces::market", component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        initial_exchange_rate: Decimal256,
        reserve_factor: Decimal256,
        underlying_asset: ContractLink<HumanAddr>,
        overseer_contract: ContractLink<HumanAddr>,
        interest_model_contract: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse> {
        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };

        Config::save(
            deps,
            &Config {
                initial_exchange_rate,
                reserve_factor,
            },
        )?;

        Contracts::save_overseer(deps, &overseer_contract)?;
        Contracts::save_interest_model(deps, &interest_model_contract)?;
        Contracts::save_underlying(deps, &underlying_asset)?;

        admin::DefaultImpl.new(admin, deps, env)?;

        Ok(InitResponse {
            messages: vec![
                snip20::set_viewing_key_msg(
                    VIEWING_KEY.into(),
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash.clone(),
                    underlying_asset.address.clone(),
                )?,
                snip20::register_receive_msg(
                    self_ref.code_hash.clone(),
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash,
                    underlying_asset.address,
                )?
            ],
            log: vec![],
        })
    }

    #[handle]
    fn receive(from: HumanAddr, msg: Option<Binary>, amount: Uint128) -> StdResult<HandleResponse> {
        if msg.is_none() {
            return Err(StdError::generic_err("\"msg\" parameter cannot be empty."));
        }
        match from_binary(&msg.unwrap())? {
            ReceiverCallbackMsg::Deposit => {
                let underlying = Contracts::load_underlying(deps)?;

                if env.message.sender != underlying.address {
                    return Err(StdError::unauthorized());
                }

                token::deposit(
                    deps,
                    env,
                    underlying,
                    from,
                    amount.into()
                )
            },
            ReceiverCallbackMsg::Repay { borrower } => {
                repay(
                    deps,
                    env,
                    if let Some(borrower) = borrower {
                        // TODO: Is a wrong/fake ID dangerous?
                        Account::try_from(borrower)?
                    } else {
                        Account::new(deps, &from)?
                    },
                    amount.into()
                )
            }
        }
    }

    #[handle]
    fn redeem_token(
        permit: Permit<OverseerPermissions>,
        burn_amount: Uint256
    ) -> StdResult<HandleResponse> {
        token::redeem(
            deps,
            env,
            permit,
            burn_amount,
            Uint256::zero()
        )
    }

    #[handle]
    fn redeem_underlying(
        permit: Permit<OverseerPermissions>,
        receive_amount: Uint256
    ) -> StdResult<HandleResponse> {
        token::redeem(
            deps,
            env,
            permit,
            Uint256::zero(),
            receive_amount
        )
    }

    #[handle]
    fn borrow(
        permit: Permit<OverseerPermissions>,
        amount: Uint256
    ) -> StdResult<HandleResponse> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address.clone(),
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        checks::assert_can_withdraw(balance.into(), amount)?;
    
        accrue_interest(deps, env.block.height, balance)?;

        checks::assert_borrow_allowed(
            deps,
            permit,
            env.contract.address,
            amount
        )?;

        let borrow_index = Global::load_borrow_index(&deps.storage)?;

        let account = Account::new(deps, &env.message.sender)?;

        let mut snapshot = account.get_borrow_snapshot(&deps.storage)?;
        snapshot.add_balance(borrow_index, amount)?;
        account.save_borrow_snapshot(&mut deps.storage, &snapshot)?;

        TotalBorrows::increase(&mut deps.storage, amount)?;

        Ok(HandleResponse {
            messages: vec![
                snip20::transfer_msg(
                    env.message.sender,
                    amount.clamp_u128()?.into(),
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash,
                    underlying_asset.address
                )?
            ],
            log: vec![
                log("action", "borrow"),
                log("borrow_info", snapshot.0)
            ],
            data: None
        })
    }

    #[handle]
    fn transfer(
        recipient: HumanAddr,
        amount: Uint256
    ) -> StdResult<HandleResponse> {
        let can_transfer = query_can_transfer(
            &deps.querier,
            Contracts::load_overseer(deps)?,
            todo!(),
            env.contract.address,
            Uint128(amount.low_u128())
        )?;

        if !can_transfer {
            return Err(StdError::generic_err("Account has negative liquidity and cannot transfer."));
        }

        let sender = Account::new(deps, &env.message.sender)?;
        sender.subtract_balance(&mut deps.storage, amount)?;

        let recipient = Account::new(deps, &recipient)?;
        recipient.add_balance(&mut deps.storage, amount)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            // SNIP-20 spec compliance.
            data: Some(to_binary(&snip20_msg::HandleAnswer::Transfer {
                status: snip20_msg::ResponseStatus::Success
            })?)
        })
    }

    #[handle]
    #[require_admin]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse> {
        let mut config = Config::load(deps)?;
        if let Some(interest_model) = interest_model {
            Contracts::save_interest_model(deps, &interest_model)?;
        }

        if let Some(reserve_factor) = reserve_factor {
            config.reserve_factor = reserve_factor;
            Config::save(deps, &config)?;
        }

        Ok(HandleResponse::default())
    }

    #[handle]
    fn reduce_reserves(amount: Uint128) -> StdResult<HandleResponse> {
        unimplemented!()
    }

    #[query("config")]
    fn config() -> StdResult<ConfigResponse> {
        unimplemented!()
    }

    #[query("state")]
    fn state() -> StdResult<StateResponse> {
        unimplemented!()
    }

    #[query("borrow_rate_per_block")]
    fn borrow_rate() -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("supply_rate_per_block")]
    fn supply_rate() -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("exchange_rate")]
    fn exchange_rate() -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("borrow_balance")]
    fn borrow_balance(id: Binary) -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("account_snapshot")]
    fn account_snapshot(id: Binary) -> StdResult<AccountInfo> {
        unimplemented!()
    }
}

fn repay<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    borrower: Account,
    amount: Uint256
) -> StdResult<HandleResponse> {
    let underlying_asset = Contracts::load_underlying(deps)?;

    let balance = snip20::balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.to_string(),
        BLOCK_SIZE,
        underlying_asset.code_hash.clone(),
        underlying_asset.address.clone(),
    )?.amount;

    accrue_interest(deps, env.block.height, balance)?;

    let borrow_index = Global::load_borrow_index(&deps.storage)?;

    let mut snapshot = borrower.get_borrow_snapshot(&deps.storage)?;
    snapshot.subtract_balance(borrow_index, amount)?;
    borrower.save_borrow_snapshot(&mut deps.storage, &snapshot)?;

    TotalBorrows::decrease(&mut deps.storage, amount)?;

    Ok(HandleResponse::default())
}

fn accrue_interest<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    current_block: u64,
    balance_prior: Uint128
) -> StdResult<()> {
    let config = Config::load(deps)?;
    // Initial block number
    let last_accrual_block = Global::load_accrual_block_number(&deps.storage)?;

    if last_accrual_block == current_block {
        return Ok(());
    }

    // Previous values from storage
    let borrows_prior = TotalBorrows::load(&deps.storage)?;
    let reserves_prior = Global::load_interest_reserve(&deps.storage)?;
    let borrow_index_prior = Global::load_borrow_index(&deps.storage)?;

    // Current borrow interest rate
    let interest_model = Contracts::load_interest_model(deps)?;
    let borrow_rate = query_borrow_rate(
        &deps.querier,
        interest_model,
        Decimal256::from_uint256(balance_prior)?,
        Decimal256::from_uint256(borrows_prior)?,
        Decimal256::from_uint256(reserves_prior)?,
    )?;

    if borrow_rate >= MAX_BORROW_RATE {
        return Err(StdError::generic_err("Borrow rate is absurdly high"));
    }

    // Calculate the number of blocks elapsed since last accrual
    let block_delta = current_block
        .checked_sub(last_accrual_block)
        .ok_or_else(|| StdError::generic_err("Could not calculate block delta"))?;

    let simple_interest_factor = (borrow_rate * Decimal256::from_uint256(block_delta)?)?;
    let interest_accumulated = borrows_prior.decimal_mul(simple_interest_factor)?;

    let total_borrows_new = (interest_accumulated + borrows_prior)?;
    TotalBorrows::save(&mut deps.storage, &total_borrows_new)?;

    let total_reserves_new = (interest_accumulated.decimal_mul(config.reserve_factor)? + reserves_prior)?;
    Global::save_interest_reserve(&mut deps.storage, &total_reserves_new)?;

    let borrow_index_new = ((borrow_index_prior * simple_interest_factor)? + borrow_index_prior)?;
    Global::save_borrow_index(&mut deps.storage, &borrow_index_new)?;

    Global::save_accrual_block_number(&mut deps.storage, current_block)?;

    Ok(())
}
