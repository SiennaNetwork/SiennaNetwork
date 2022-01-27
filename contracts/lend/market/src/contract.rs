mod checks;
mod state;
mod token;
mod ops;

use lend_shared::{
    fadroma::{
        admin,
        admin::{assert_admin, Admin},
        cosmwasm_std,
        cosmwasm_std::{
            Api, Binary, Extern, HandleResponse, HumanAddr, Env,
            InitResponse, Querier, StdError, StdResult, Storage,
            CosmosMsg, WasmMsg, log
        },
        auth::{
            vk_auth::{
                DefaultImpl as AuthImpl, Auth
            }
        },
        derive_contract::*,
        require_admin,
        secret_toolkit::snip20,
        snip20_impl::msg as snip20_msg,
        Uint256, Decimal256, Uint128,
        BLOCK_SIZE, ContractLink, Callback,
        from_binary, to_binary
    },
    interfaces::{
        interest_model::{query_borrow_rate, query_supply_rate},
        market::{
            ReceiverCallbackMsg, State, HandleMsg,
            MarketPermissions, AccountInfo, Config,
            MarketAuth, Borrower, query_balance, 
        },
        overseer::{
            query_can_transfer,
            query_seize_amount,
            query_market,
            query_account_liquidity,
            query_entered_markets
        },
    },
    core::{MasterKey, AuthenticatedUser}
};

pub const VIEWING_KEY: &str = "SiennaLend"; // TODO: This shouldn't be hardcoded.
pub const MAX_RESERVE_FACTOR: Decimal256 = Decimal256::one();
// TODO: proper value here
pub const MAX_BORROW_RATE: Decimal256 = Decimal256::one();

use state::{
    Constants, Contracts, Global, BorrowerId,
    Account, TotalBorrows, TotalSupply, load_borrowers
};
use token::calc_exchange_rate;
use ops::{accrue_interest, accrued_interest_at};

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::market",
    component(path = "admin")
)]
pub trait Market {
    #[init]
    fn new(
        admin: HumanAddr,
        prng_seed: Binary,
        key: MasterKey,
        underlying_asset: ContractLink<HumanAddr>,
        interest_model_contract: ContractLink<HumanAddr>,
        config: Config,
        callback: Callback<HumanAddr>
    ) -> StdResult<InitResponse> {
        key.save(&mut deps.storage)?;

        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };

        Constants::save(&mut deps.storage, &config)?;

        BorrowerId::set_prng_seed(&mut deps.storage, &prng_seed)?;
        Contracts::save_overseer(deps, &callback.contract)?;
        Contracts::save_interest_model(deps, &interest_model_contract)?;
        Contracts::save_underlying(deps, &underlying_asset)?;
        Contracts::save_self_ref(deps, &ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        })?;

        Global::save_borrow_index(&mut deps.storage, &Decimal256::one())?;
        Global::save_accrual_block_number(&mut deps.storage, env.block.height)?;

        admin::DefaultImpl.new(Some(admin), deps, env)?;

        Ok(InitResponse {
            messages: vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    send: vec![],
                    callback_code_hash: callback.contract.code_hash,
                    contract_addr: callback.contract.address,
                    msg: callback.msg
                }),
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
    fn receive(_sender: HumanAddr, from: HumanAddr, msg: Option<Binary>, amount: Uint128) -> StdResult<HandleResponse> {
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
                let underlying = Contracts::load_underlying(deps)?;

                if env.message.sender != underlying.address {
                    return Err(StdError::unauthorized());
                }

                repay(
                    deps,
                    env,
                    underlying,
                    if let Some(borrower) = borrower {
                        Account::from_id(&deps.storage, &borrower)?
                    } else {
                        Account::of(deps, &from)?
                    },
                    amount.into()
                )
            },
            ReceiverCallbackMsg::Liquidate {
                borrower,
                collateral
            } => {
                let underlying = Contracts::load_underlying(deps)?;

                if env.message.sender != underlying.address {
                    return Err(StdError::unauthorized());
                }

                liquidate(
                    deps,
                    env,
                    underlying,
                    from,
                    Account::from_id(&deps.storage, &borrower)?,
                    collateral,
                    amount.into()
                )
            }
        }
    }

    #[handle]
    fn redeem_token(burn_amount: Uint256) -> StdResult<HandleResponse> {
        token::redeem(
            deps,
            env,
            burn_amount,
            Uint256::zero()
        )
    }

    #[handle]
    fn redeem_underlying(receive_amount: Uint256) -> StdResult<HandleResponse> {
        token::redeem(
            deps,
            env,
            Uint256::zero(),
            receive_amount
        )
    }

    #[handle]
    fn borrow(amount: Uint256) -> StdResult<HandleResponse> {
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
            env.message.sender.clone(),
            env.block.height,
            env.contract.address,
            amount
        )?;

        let borrow_index = Global::load_borrow_index(&deps.storage)?;

        let account = Account::of(deps, &env.message.sender)?;

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
        let sender = Account::of(deps, &env.message.sender)?;

        let can_transfer = query_can_transfer(
            &deps.querier,
            Contracts::load_overseer(deps)?,
            MasterKey::load(&deps.storage)?,
            env.message.sender,
            env.contract.address,
            env.block.height,
            amount
        )?;

        if !can_transfer {
            return Err(StdError::generic_err("Account has negative liquidity and cannot transfer."));
        }

        sender.subtract_balance(&mut deps.storage, amount)?;

        let recipient = Account::of(deps, &recipient)?;
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
    fn accrue_interest() -> StdResult<HandleResponse> {
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

        Ok(HandleResponse::default())
    }

    #[handle]
    fn seize(
        liquidator: HumanAddr,
        borrower: HumanAddr,
        amount: Uint256
    ) -> StdResult<HandleResponse> {
        // Assert that the caller is a market contract.
        query_market(
            &deps.querier,
            Contracts::load_overseer(deps)?,
            env.message.sender.clone()
        )?;

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

        seize(
            deps,
            balance.into(),
            Account::of(deps, &liquidator)?,
            Account::of(deps, &borrower)?,
            amount
        )
    }

    #[handle]
    #[require_admin]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse> {
        let mut config = Constants::load(&deps.storage)?;
        if let Some(interest_model) = interest_model {
            Contracts::save_interest_model(deps, &interest_model)?;
        }

        if let Some(reserve_factor) = reserve_factor {
            config.reserve_factor = reserve_factor;
            Constants::save(&mut deps.storage, &config)?;
        }

        Ok(HandleResponse::default())
    }

    #[handle]
    #[require_admin]
    fn reduce_reserves(amount: Uint128, to: Option<HumanAddr>) -> StdResult<HandleResponse> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address.clone(),
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        if balance < amount {
            return Err(StdError::generic_err(format!(
                "Insufficient underlying balance. Balance: {}, Required: {}",
                balance,
                amount
            )));
        }

        accrue_interest(deps, env.block.height, balance)?;

        // Load after accrue_interest(), because it's updated inside.
        let reserve = Global::load_interest_reserve(&deps.storage)?;
        let amount_256 = Uint256::from(amount);

        if reserve < amount_256 {
            return Err(StdError::generic_err(format!(
                "Insufficient reserve balance. Balance: {}, Required: {}",
                reserve,
                amount
            )));
        }

        let reserve = (reserve - amount_256)?;
        Global::save_interest_reserve(&mut deps.storage, &reserve)?;

        Ok(HandleResponse {
            messages: vec![
                snip20::transfer_msg(
                    to.unwrap_or(env.message.sender),
                    amount,
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash,
                    underlying_asset.address
                )?
            ],
            log: vec![
                log("action", "reduce_reserves"),
                log("new_reserve", reserve)
            ],
            data: None
        })
    }

    #[handle]
    fn create_viewing_key(
        entropy: String,
        padding: Option<String>
    ) -> StdResult<HandleResponse> {
        AuthImpl.create_viewing_key(entropy, padding, deps, env)
    }

    #[handle]
    fn set_viewing_key(
        key: String,
        padding: Option<String>
    ) -> StdResult<HandleResponse> {
        AuthImpl.set_viewing_key(key, padding, deps, env)
    }

    #[query]
    fn balance(
        address: HumanAddr,
        key: String
    ) -> StdResult<Uint128> {
        let account = Account::auth_viewing_key(deps, key, &address)?;

        Ok(account.get_balance(&deps.storage)?
            .low_u128()
            .into()
        )
    }

    #[query]
    fn balance_underlying(
        method: MarketAuth,
        block: Option<u64>
    ) -> StdResult<Uint128> {
        let account = Account::authenticate(
            deps,
            method,
            MarketPermissions::Balance,
            Contracts::load_self_ref
        )?;

        let exchange_rate = self.exchange_rate(block, deps)?;
        let balance = account.get_balance(&deps.storage)?;

        Ok(balance.decimal_mul(exchange_rate)?
            .low_u128()
            .into()
        )
    }

    #[query]
    fn balance_internal(
        address: HumanAddr,
        key: MasterKey
    ) -> StdResult<Uint128> {
        MasterKey::check(&deps.storage, &key)?;

        let account = Account::of(deps, &address)?;

        Ok(account.get_balance(&deps.storage)?
            .low_u128()
            .into()
        )
    }

    #[query]
    fn state(block: Option<u64>) -> StdResult<State> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        let interest = accrued_interest_at(
            deps,
            block,
            balance
        )?;

        Ok(State {
            underlying_balance: balance,
            total_borrows: interest.total_borrows,
            total_reserves: interest.total_reserves,
            borrow_index: interest.borrow_index,
            total_supply: TotalSupply::load(&deps.storage)?,
            accrual_block: Global::load_accrual_block_number(&deps.storage)?,
            config: Constants::load(&deps.storage)?
        })
    }

    #[query]
    fn underlying_asset() -> StdResult<ContractLink<HumanAddr>> {
        Contracts::load_underlying(deps)
    }

    #[query]
    fn borrow_rate(block: Option<u64>) -> StdResult<Decimal256> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        let interest = accrued_interest_at(
            deps,
            block,
            balance
        )?;
    
        query_borrow_rate(
            &deps.querier,
            Contracts::load_interest_model(deps)?,
            Decimal256::from_uint256(balance)?,
            Decimal256::from_uint256(interest.total_borrows)?,
            Decimal256::from_uint256(interest.total_reserves)?,
        )
    }

    #[query]
    fn supply_rate(block: Option<u64>) -> StdResult<Decimal256> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        let interest = accrued_interest_at(
            deps,
            block,
            balance
        )?;

        query_supply_rate(
            &deps.querier,
            Contracts::load_interest_model(deps)?,
            Decimal256::from_uint256(balance)?,
            Decimal256::from_uint256(interest.total_borrows)?,
            Decimal256::from_uint256(interest.total_reserves)?,
            Constants::load(&deps.storage)?.reserve_factor
        )
    }

    #[query]
    fn exchange_rate(block: Option<u64>) -> StdResult<Decimal256> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        let interest = accrued_interest_at(
            deps,
            block,
            balance
        )?;

        calc_exchange_rate(
            deps,
            balance.into(),
            interest.total_borrows,
            interest.total_reserves
        )
    }

    #[query]
    fn account(method: MarketAuth, block: Option<u64>) -> StdResult<AccountInfo> {
        let account = Account::authenticate(
            deps,
            method,
            MarketPermissions::AccountInfo,
            Contracts::load_self_ref
        )?;

        let underlying_asset = Contracts::load_underlying(deps)?;
        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            VIEWING_KEY.to_string(),
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?.amount;

        let interest = accrued_interest_at(
            deps,
            block,
            balance
        )?;

        let snapshot = account.get_borrow_snapshot(&deps.storage)?;

        Ok(AccountInfo {
            sl_token_balance: account.get_balance(&deps.storage)?,
            borrow_balance: snapshot.current_balance(interest.borrow_index)?,
            exchange_rate: calc_exchange_rate(
                deps,
                balance.into(),
                interest.total_borrows,
                interest.total_reserves
            )?
        })
    }

    #[query]
    fn borrowers(
        block: u64,
        start_after: Option<Binary>,
        limit: Option<u8>
    ) -> StdResult<Vec<Borrower>> {
        let borrowers = load_borrowers(deps, start_after, limit)?;
        let mut result = Vec::with_capacity(borrowers.len());

        let overseer = Contracts::load_overseer(deps)?;
        let key = MasterKey::load(&deps.storage)?;
        
        for record in borrowers {
            result.push(Borrower {
                id: record.id,
                info: record.info,
                liquidity: query_account_liquidity(
                    &deps.querier,
                    overseer.clone(),
                    key.clone(),
                    record.address.clone(),
                    None,
                    Some(block),
                    Uint256::zero(),
                    Uint256::zero()
                )?,
                markets: query_entered_markets(
                    &deps.querier,
                    overseer.clone(),
                    key.clone(),
                    record.address
                )?
            });
        }

        Ok(result)
    }
}

fn repay<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    underlying_asset: ContractLink<HumanAddr>,
    borrower: Account,
    amount: Uint256
) -> StdResult<HandleResponse> {
    let balance = snip20::balance_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.to_string(),
        BLOCK_SIZE,
        underlying_asset.code_hash,
        underlying_asset.address,
    )?.amount;

    accrue_interest(deps, env.block.height, balance)?;

    let borrow_index = Global::load_borrow_index(&deps.storage)?;

    let mut snapshot = borrower.get_borrow_snapshot(&deps.storage)?;
    snapshot.subtract_balance(borrow_index, amount)?;
    borrower.save_borrow_snapshot(&mut deps.storage, &snapshot)?;

    TotalBorrows::decrease(&mut deps.storage, amount)?;

    Ok(HandleResponse::default())
}

fn liquidate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    underlying_asset: ContractLink<HumanAddr>,
    liquidator_address: HumanAddr,
    borrower: Account,
    collateral: HumanAddr,
    amount: Uint256
) -> StdResult<HandleResponse> {
    let liquidator = Account::of(deps, &liquidator_address)?;

    if liquidator == borrower {
        return Err(StdError::generic_err("Liquidator and borrower are the same account."));
    }

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

    checks::assert_liquidate_allowed(
        deps,
        HumanAddr::default(), // TODO: get borrower address
        snapshot.current_balance(borrow_index)?,
        env.block.height,
        amount
    )?;

    // Do repay
    snapshot.subtract_balance(borrow_index, amount)?;
    borrower.save_borrow_snapshot(&mut deps.storage, &snapshot)?;

    TotalBorrows::decrease(&mut deps.storage, amount)?;

    let overseer = Contracts::load_overseer(deps)?;
    let borrower_address = borrower.address(&deps.api)?;

    let (borrower_balance, market) = if env.contract.address == collateral {
        (borrower.get_balance(&deps.storage)?, None)
    } else {
        let market = query_market(
            &deps.querier,
            overseer.clone(),
            collateral.clone()
        )?;

        let borrower_balance: Uint256 = query_balance(
            &deps.querier,
            market.contract.clone(),
            MasterKey::load(&deps.storage)?,
            borrower_address.clone()
        )?.into();

        (borrower_balance, Some(market))
    };

    let seize_amount = query_seize_amount(
        &deps.querier,
        overseer,
        env.contract.address,
        collateral,
        amount
    )?;

    if borrower_balance < seize_amount {
        return Err(StdError::generic_err(format!(
            "Borrow collateral balance is less that the seize amount. Shortfall: {}",
            (seize_amount - borrower_balance)?
        )));
    }

    if let Some(market) = market {
        Ok(HandleResponse {
            messages: vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    send: vec![],
                    contract_addr: market.contract.address,
                    callback_code_hash: market.contract.code_hash,
                    msg: to_binary(&HandleMsg::Seize {
                        liquidator: liquidator_address,
                        borrower: borrower_address,
                        amount
                    })?
                })
            ],
            log: vec![],
            data: None
        })
    } else {
        seize(
            deps,
            balance.into(),
            liquidator,
            borrower,
            amount
        )
    }
}

fn seize<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    underlying_balance: Uint256,
    liquidator: Account,
    borrower: Account,
    amount: Uint256
) -> StdResult<HandleResponse> {
    borrower.subtract_balance(&mut deps.storage, amount)?;

    let config = Constants::load(&deps.storage)?;
    // TODO: how are those two next lines correct???
    // https://github.com/compound-finance/compound-protocol/blob/4a8648ec0364d24c4ecfc7d6cae254f55030d65f/contracts/CToken.sol#L1085-L1086
    let protocol_share = amount.decimal_mul(config.seize_factor)?;
    let liquidator_share = (amount - protocol_share)?;

    let interest_reserve = Global::load_interest_reserve(&deps.storage)?;
    let exchange_rate = calc_exchange_rate(
        deps,
        underlying_balance,
        TotalBorrows::load(&deps.storage)?,
        interest_reserve
    )?;

    let protocol_amount = protocol_share.decimal_mul(exchange_rate)?;

    Global::save_interest_reserve(
        &mut deps.storage,
        &(interest_reserve + protocol_amount)?
    )?;

    TotalSupply::decrease(&mut deps.storage, protocol_share)?;

    liquidator.add_balance(&mut deps.storage, liquidator_share)?;

    Ok(HandleResponse::default())
}
