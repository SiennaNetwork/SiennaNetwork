mod checks;
mod ops;
mod state;
mod token;

use lend_shared::{
    core::{AuthenticatedUser, MasterKey, Pagination},
    fadroma::{
        admin,
        admin::{assert_admin, Admin},
        auth::{
            vk_auth::{Auth, DefaultImpl as AuthImpl},
            ViewingKey,
        },
        cosmwasm_std,
        cosmwasm_std::{
            Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
            HumanAddr, InitResponse, Querier, StdError, StdResult,
            Storage, WasmMsg, log
        },
        derive_contract::*,
        from_binary, killswitch, require_admin,
        secret_toolkit::snip20,
        snip20_impl::msg as snip20_msg,
        snip20_impl::receiver::Snip20ReceiveMsg,
        to_binary, Callback, ContractLink, Decimal256, Uint128, Uint256, BLOCK_SIZE,
    },
    interfaces::{
        interest_model::{query_borrow_rate, query_supply_rate},
        market::{
            AccountInfo, Borrower, Config, HandleMsg, MarketAuth,
            MarketPermissions, ReceiverCallbackMsg, State, BorrowersResponse,
            SimulateLiquidationResult, query_simulate_seize
        },
        overseer::{
            query_account_liquidity, query_can_transfer, query_entered_markets,
            query_market, query_seize_amount,
        },
    },
};

pub const MAX_RESERVE_FACTOR: Decimal256 = Decimal256::one();
/// 0.0005%
pub const MAX_BORROW_RATE: u64 = 5000000000000;

const TOKEN_PREFIX: &str = "sl-";

use ops::{accrue_interest, accrued_interest_at, LatestInterest};
use state::{
    load_borrowers, Account, BorrowerId, Constants, Contracts,
    Global, TotalBorrows, TotalSupply, ReceiverRegistry
};
use token::calc_exchange_rate;

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::market",
    component(path = "admin"),
    component(path = "killswitch")
)]
pub trait Market {
    #[init]
    fn new(
        admin: HumanAddr,
        prng_seed: Binary,
        entropy: Binary,
        key: MasterKey,
        underlying_asset: ContractLink<HumanAddr>,
        interest_model_contract: ContractLink<HumanAddr>,
        config: Config,
        callback: Callback<HumanAddr>,
    ) -> StdResult<InitResponse> {
        key.save(&mut deps.storage)?;

        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };

        config.validate()?;
        Constants::save_config(&mut deps.storage, &config)?;
        BorrowerId::set_prng_seed(&mut deps.storage, &prng_seed)?;

        Contracts::save_overseer(deps, callback.contract.clone())?;
        Contracts::save_interest_model(deps, interest_model_contract)?;
        Contracts::save_underlying(deps, underlying_asset.clone())?;
        Contracts::save_self_ref(deps, self_ref.clone())?;

        Global::save_borrow_index(&mut deps.storage, &Decimal256::one())?;
        Global::save_accrual_block_number(&mut deps.storage, env.block.height)?;

        let viewing_key = ViewingKey::new(&env, prng_seed.as_slice(), entropy.as_slice()).0;
        Constants::save_vk(&mut deps.storage, &viewing_key)?;

        admin::DefaultImpl.new(Some(admin), deps, env)?;

        Ok(InitResponse {
            messages: vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    send: vec![],
                    callback_code_hash: callback.contract.code_hash,
                    contract_addr: callback.contract.address,
                    msg: callback.msg,
                }),
                snip20::set_viewing_key_msg(
                    viewing_key,
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash.clone(),
                    underlying_asset.address.clone(),
                )?,
                snip20::register_receive_msg(
                    self_ref.code_hash,
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash,
                    underlying_asset.address,
                )?,
            ],
            log: vec![],
        })
    }

    #[handle_guard]
    fn guard(msg: &HandleMsg) -> StdResult<()> {
        let operational = killswitch::is_operational(deps);

        if operational.is_err()
            && matches!(
                msg,
                HandleMsg::UpdateConfig { .. }
                    | HandleMsg::ReduceReserves { .. }
                    | HandleMsg::SetViewingKey { .. }
                    | HandleMsg::CreateViewingKey { .. }
                    | HandleMsg::Killswitch(_)
                    | HandleMsg::Admin(_)
            )
        {
            Ok(())
        } else {
            operational
        }
    }

    #[handle]
    fn receive(
        _sender: HumanAddr,
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128,
    ) -> StdResult<HandleResponse> {
        if msg.is_none() {
            return Err(StdError::generic_err("\"msg\" parameter cannot be empty."));
        }

        let underlying = Contracts::load_underlying(deps)?;

        if env.message.sender != underlying.address {
            return Err(StdError::unauthorized());
        }

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address.clone(),
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying.code_hash,
            underlying.address,
        )?
        .amount;

        // Because balance is already increased, we must subtract the incoming amount
        // in order to get the correct interest/exchange rate up to this point.
        let balance = (balance - amount)?;

        let interest = accrue_interest(deps, env.block.height, balance.into())?;

        match from_binary(&msg.unwrap())? {
            ReceiverCallbackMsg::Deposit => {
                token::deposit(deps, interest, balance.into(), from, amount.into())
            }
            ReceiverCallbackMsg::Repay { borrower } => repay(
                deps,
                interest,
                if let Some(borrower) = borrower {
                    Account::from_id(&deps.storage, &borrower)?
                } else {
                    Account::of(deps, &from)?
                },
                from,
                amount.into(),
            ),
            ReceiverCallbackMsg::Liquidate {
                borrower,
                collateral,
            } => liquidate(
                deps,
                env,
                interest,
                balance.into(),
                from,
                Account::from_id(&deps.storage, &borrower)?,
                collateral,
                amount.into(),
            ),
        }
    }

    #[handle]
    fn redeem_token(burn_amount: Uint256) -> StdResult<HandleResponse> {
        token::redeem(deps, env, burn_amount, Uint256::zero())
    }

    #[handle]
    fn redeem_underlying(receive_amount: Uint256) -> StdResult<HandleResponse> {
        token::redeem(deps, env, Uint256::zero(), receive_amount)
    }

    #[handle]
    fn borrow(amount: Uint256) -> StdResult<HandleResponse> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address.clone(),
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        checks::assert_can_withdraw(balance.into(), amount)?;

        let mut latest = accrue_interest(deps, env.block.height, balance)?;

        checks::assert_borrow_allowed(
            deps,
            env.message.sender.clone(),
            env.block.height,
            env.contract.address,
            amount,
        )?;

        let account = Account::new(deps, &env.message.sender)?;

        let mut snapshot = account.get_borrow_snapshot(&deps.storage)?;
        snapshot.add_balance(latest.borrow_index(&deps.storage)?, amount)?;
        account.save_borrow_snapshot(&mut deps.storage, snapshot)?;

        TotalBorrows::increase(&mut deps.storage, amount)?;

        Ok(HandleResponse {
            messages: vec![snip20::transfer_msg(
                env.message.sender,
                amount.clamp_u128()?.into(),
                None,
                None,
                BLOCK_SIZE,
                underlying_asset.code_hash,
                underlying_asset.address,
            )?],
            log: vec![log("action", "borrow")],
            data: None,
        })
    }

    #[handle]
    fn transfer(recipient: HumanAddr, amount: Uint256) -> StdResult<HandleResponse> {
        do_transfer(deps, env, &recipient, amount)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            // SNIP-20 spec compliance.
            data: Some(to_binary(&snip20_msg::HandleAnswer::Transfer {
                status: snip20_msg::ResponseStatus::Success
            })?),
        })
    }

    #[handle]
    fn send(
        recipient: HumanAddr,
        recipient_code_hash: Option<String>,
        amount: Uint256,
        msg: Option<Binary>,
        memo: Option<String>,
        _padding: Option<String>,
    ) -> StdResult<HandleResponse> {
        let sender = env.message.sender.clone();

        do_transfer(deps, env, &recipient, amount)?;

        let code_hash = if recipient_code_hash.is_some() {
            recipient_code_hash
        } else {
            ReceiverRegistry::get(deps, &recipient)?
        };

        let messages = if let Some(code_hash) = code_hash {
            vec![
                Snip20ReceiveMsg {
                    amount: amount.low_u128().into(),
                    from: sender.clone(),
                    sender,
                    msg,
                    memo
                }.into_cosmos_msg(code_hash, recipient)?
            ]
        } else {
            vec![]
        };

        Ok(HandleResponse {
            messages,
            log: vec![],
            // SNIP-20 spec compliance.
            data: Some(to_binary(&snip20_msg::HandleAnswer::Send {
                status: snip20_msg::ResponseStatus::Success
            })?),
        })
    }

    #[handle]
    fn register_receive(
        code_hash: String, 
        _padding: Option<String>
    ) -> StdResult<HandleResponse> {
        ReceiverRegistry::set(deps, &env.message.sender, &code_hash)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("register_status", "success")],
            // SNIP-20 spec compliance.
            data: Some(to_binary(&snip20_msg::HandleAnswer::RegisterReceive {
                status: snip20_msg::ResponseStatus::Success
            })?),
        })
    }

    #[handle]
    fn accrue_interest() -> StdResult<HandleResponse> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address.clone(),
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        accrue_interest(deps, env.block.height, balance)?;

        Ok(HandleResponse::default())
    }

    #[handle]
    fn seize(
        liquidator: HumanAddr,
        borrower: HumanAddr,
        amount: Uint256,
    ) -> StdResult<HandleResponse> {
        // Assert that the caller is a market contract.
        query_market(
            &deps.querier,
            Contracts::load_overseer(deps)?,
            env.message.sender
        )?;

        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash,
            underlying_asset.address,
        )?
        .amount;

        let latest = accrue_interest(deps, env.block.height, balance)?;

        seize(
            deps,
            latest,
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
        borrow_cap: Option<Uint256>,
    ) -> StdResult<HandleResponse> {
        let underlying_asset = Contracts::load_underlying(deps)?;
        let balance = snip20::balance_query(
            &deps.querier,
            env.contract.address.clone(),
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;
        accrue_interest(deps, env.block.height, balance)?;

        if let Some(interest_model) = interest_model {
            Contracts::save_interest_model(deps, interest_model)?;
        }

        if let Some(reserve_factor) = reserve_factor {
            let mut config = Constants::load_config(&deps.storage)?;
            config.set_reserve_factor(reserve_factor)?;
            Constants::save_config(&mut deps.storage, &config)?;
        }

        if let Some(borrow_cap) = borrow_cap {
            Global::save_borrow_cap(&mut deps.storage, &borrow_cap)?;
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
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        if balance < amount {
            return Err(StdError::generic_err(format!(
                "Insufficient underlying balance. Balance: {}, Required: {}",
                balance, amount
            )));
        }

        let mut latest = accrue_interest(deps, env.block.height, balance)?;

        // Load after accrue_interest(), because it's updated inside.
        let reserve = latest.total_reserves(&deps.storage)?;
        let amount_256 = Uint256::from(amount);

        if reserve < amount_256 {
            return Err(StdError::generic_err(format!(
                "Insufficient reserve balance. Balance: {}, Required: {}",
                reserve, amount
            )));
        }

        let reserve = (reserve - amount_256)?;
        Global::save_interest_reserve(&mut deps.storage, &reserve)?;

        Ok(HandleResponse {
            messages: vec![snip20::transfer_msg(
                to.unwrap_or(env.message.sender),
                amount,
                None,
                None,
                BLOCK_SIZE,
                underlying_asset.code_hash,
                underlying_asset.address,
            )?],
            log: vec![
                log("action", "reduce_reserves"),
                log("new_reserve", reserve),
            ],
            data: None,
        })
    }

    #[handle]
    fn create_viewing_key(entropy: String, padding: Option<String>) -> StdResult<HandleResponse> {
        AuthImpl.create_viewing_key(entropy, padding, deps, env)
    }

    #[handle]
    fn set_viewing_key(key: String, padding: Option<String>) -> StdResult<HandleResponse> {
        AuthImpl.set_viewing_key(key, padding, deps, env)
    }

    #[query]
    fn simulate_liquidation(
        block: u64,
        borrower: Binary,
        collateral: HumanAddr,
        amount: Uint256
    ) -> StdResult<SimulateLiquidationResult> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, Some(block), balance)?;

        let borrower = Account::from_id(&deps.storage, &borrower)?;
        let snapshot = borrower.get_borrow_snapshot(&deps.storage)?;
    
        let borrower_address = borrower.address(&deps.api)?;
    
        let overseer = Contracts::load_overseer(deps)?;
        checks::assert_liquidate_allowed(
            deps,
            overseer.clone(),
            borrower_address.clone(),
            snapshot.current_balance(interest.borrow_index)?,
            block,
            amount,
        )?;

        let this = Contracts::load_self_ref(deps)?;
        let this_is_collateral = this.address == collateral;
    
        let seize_amount = query_seize_amount(
            &deps.querier,
            overseer.clone(),
            this.address,
            collateral.clone(),
            amount
        )?;
    
        let shortfall = if this_is_collateral {
            borrower.can_subtract(&deps.storage, seize_amount)?
        } else {
            let market = query_market(&deps.querier, overseer, collateral)?;
            
            query_simulate_seize(
                &deps.querier,
                market.contract,
                MasterKey::load(&deps.storage)?,
                borrower_address,
                seize_amount
            )?
        };

        Ok(SimulateLiquidationResult {
            seize_amount,
            shortfall
        })
    }

    #[query]
    fn simulate_seize(
        key: MasterKey,
        borrower: HumanAddr,
        amount: Uint256
    ) -> StdResult<Uint256> {
        MasterKey::check(&deps.storage, &key)?;

        let borrower = Account::of(&deps, &borrower)?;

        borrower.can_subtract(&deps.storage, amount)
    }

    #[query]
    fn token_info() -> StdResult<snip20_msg::QueryAnswer> {
        let underlying = Contracts::load_underlying(deps)?;

        let info = snip20::token_info_query(
            &deps.querier,
            BLOCK_SIZE,
            underlying.code_hash,
            underlying.address,
        )?;

        Ok(snip20_msg::QueryAnswer::TokenInfo {
            name: format!("Sienna Lend Market for {}", info.symbol),
            symbol: format!("{}{}", TOKEN_PREFIX, info.symbol),
            decimals: info.decimals,
            total_supply: Some(TotalSupply::load(&deps.storage)?.low_u128().into()),
        })
    }

    #[query]
    fn balance(address: HumanAddr, key: String) -> StdResult<Uint128> {
        let account = Account::auth_viewing_key(deps, key, &address)?;

        Ok(account.get_balance(&deps.storage)?.low_u128().into())
    }

    #[query]
    fn balance_underlying(method: MarketAuth, block: Option<u64>) -> StdResult<Uint128> {
        let account = Account::authenticate(
            deps,
            method,
            MarketPermissions::Balance,
            Contracts::load_self_ref,
        )?;

        let exchange_rate = self.exchange_rate(block, deps)?;
        let balance = account.get_balance(&deps.storage)?;

        Ok(balance.decimal_mul(exchange_rate)?.low_u128().into())
    }

    #[query]
    fn state(block: Option<u64>) -> StdResult<State> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, block, balance)?;

        Ok(State {
            underlying_balance: balance,
            total_borrows: interest.total_borrows,
            total_reserves: interest.total_reserves,
            borrow_index: interest.borrow_index,
            total_supply: TotalSupply::load(&deps.storage)?,
            accrual_block: Global::load_accrual_block_number(&deps.storage)?,
            config: Constants::load_config(&deps.storage)?,
        })
    }

    #[query]
    fn underlying_asset() -> StdResult<ContractLink<HumanAddr>> {
        Contracts::load_underlying(deps)
    }

    #[query]
    fn interest_model() -> StdResult<ContractLink<HumanAddr>> {
        Contracts::load_interest_model(deps)
    }

    #[query]
    fn borrow_rate(block: Option<u64>) -> StdResult<Decimal256> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, block, balance)?;

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
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, block, balance)?;

        query_supply_rate(
            &deps.querier,
            Contracts::load_interest_model(deps)?,
            Decimal256::from_uint256(balance)?,
            Decimal256::from_uint256(interest.total_borrows)?,
            Decimal256::from_uint256(interest.total_reserves)?,
            Constants::load_config(&deps.storage)?.reserve_factor,
        )
    }

    #[query]
    fn exchange_rate(block: Option<u64>) -> StdResult<Decimal256> {
        let underlying_asset = Contracts::load_underlying(deps)?;

        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, block, balance)?;

        calc_exchange_rate(
            deps,
            balance.into(),
            interest.total_borrows,
            interest.total_reserves,
        )
    }

    #[query]
    fn account(method: MarketAuth, block: Option<u64>) -> StdResult<AccountInfo> {
        let account = Account::authenticate(
            deps,
            method,
            MarketPermissions::AccountInfo,
            Contracts::load_self_ref,
        )?;

        let underlying_asset = Contracts::load_underlying(deps)?;
        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, block, balance)?;

        let snapshot = account.get_borrow_snapshot(&deps.storage)?;

        Ok(AccountInfo {
            sl_token_balance: account.get_balance(&deps.storage)?,
            borrow_balance: snapshot.current_balance(interest.borrow_index)?,
            exchange_rate: calc_exchange_rate(
                deps,
                balance.into(),
                interest.total_borrows,
                interest.total_reserves,
            )?,
        })
    }

    #[query]
    fn id(method: MarketAuth) -> StdResult<Binary> {
        let account = Account::authenticate(
            deps,
            method,
            MarketPermissions::Id,
            Contracts::load_self_ref,
        )?;

        account.get_id(&deps.storage)
    }

    #[query]
    fn borrowers(block: u64, pagination: Pagination) -> StdResult<BorrowersResponse> {
        let (total, borrowers) = load_borrowers(deps, pagination)?;
        let mut result = Vec::with_capacity(borrowers.len());

        let overseer = Contracts::load_overseer(deps)?;
        let key = MasterKey::load(&deps.storage)?;

        let underlying_asset = Contracts::load_underlying(deps)?;
        let balance = snip20::balance_query(
            &deps.querier,
            Contracts::load_self_ref(deps)?.address,
            Constants::load_vk(&deps.storage)?,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?
        .amount;

        let interest = accrued_interest_at(deps, Some(block), balance)?;

        for record in borrowers {
            result.push(Borrower {
                id: record.id,
                principal_balance: record.snapshot.info.principal,
                actual_balance: record.snapshot.current_balance(interest.borrow_index)?,
                liquidity: query_account_liquidity(
                    &deps.querier,
                    overseer.clone(),
                    key.clone(),
                    record.address.clone(),
                    None,
                    Some(block),
                    Uint256::zero(),
                    Uint256::zero(),
                )?,
                markets: query_entered_markets(
                    &deps.querier,
                    overseer.clone(),
                    key.clone(),
                    record.address,
                )?,
            });
        }

        Ok(BorrowersResponse {
            total,
            entries: result
        })
    }
}

fn do_transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: &HumanAddr,
    amount: Uint256
) -> StdResult<()> {
    let sender = Account::of(deps, &env.message.sender)?;
    let recipient = Account::of(deps, &recipient)?;

    let can_transfer = query_can_transfer(
        &deps.querier,
        Contracts::load_overseer(deps)?,
        MasterKey::load(&deps.storage)?,
        env.message.sender,
        env.contract.address,
        env.block.height,
        amount,
    )?;

    if !can_transfer {
        return Err(StdError::generic_err(
            "Account has negative liquidity and cannot transfer.",
        ));
    }

    sender.subtract_balance(&mut deps.storage, amount)?;
    
    recipient.add_balance(&mut deps.storage, amount)
}

fn repay<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    mut interest: LatestInterest,
    borrower: Account,
    sender: HumanAddr,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let mut snapshot = borrower.get_borrow_snapshot(&deps.storage)?;
    let remainder = snapshot.subtract_balance(interest.borrow_index(&deps.storage)?, amount)?;
    borrower.save_borrow_snapshot(&mut deps.storage, snapshot)?;

    let amount = (amount.0 - remainder.0).into();
    TotalBorrows::decrease(&mut deps.storage, amount)?;

    if remainder > Uint256::zero() {
        let underlying = Contracts::load_underlying(deps)?;

        Ok(HandleResponse {
            messages: vec![snip20::transfer_msg(
                sender,
                remainder.low_u128().into(),
                None,
                None,
                BLOCK_SIZE,
                underlying.code_hash,
                underlying.address
            )?],
            log: vec![],
            data: None
        })
    } else {
        Ok(HandleResponse::default())
    }
}

fn liquidate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mut interest: LatestInterest,
    underlying_balance: Uint256,
    liquidator_address: HumanAddr,
    borrower: Account,
    collateral: HumanAddr,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let liquidator = Account::of(deps, &liquidator_address)?;

    if liquidator == borrower {
        return Err(StdError::generic_err(
            "Liquidator and borrower are the same account.",
        ));
    }

    let borrow_index = interest.borrow_index(&deps.storage)?;
    let mut snapshot = borrower.get_borrow_snapshot(&deps.storage)?;

    let borrower_address = borrower.address(&deps.api)?;

    let overseer = Contracts::load_overseer(deps)?;
    checks::assert_liquidate_allowed(
        deps,
        overseer.clone(),
        borrower_address.clone(),
        snapshot.current_balance(borrow_index)?,
        env.block.height,
        amount,
    )?;

    // Do repay
    snapshot.subtract_balance(borrow_index, amount)?;
    borrower.save_borrow_snapshot(&mut deps.storage, snapshot)?;

    TotalBorrows::decrease(&mut deps.storage, amount)?;

    let this_is_collateral = env.contract.address == collateral;

    let seize_amount = query_seize_amount(
        &deps.querier,
        overseer.clone(),
        env.contract.address,
        collateral.clone(),
        amount,
    )?;

    if this_is_collateral {
        seize(
            deps,
            interest,
            underlying_balance,
            liquidator,
            borrower,
            seize_amount,
        )
    } else {
        let market = query_market(&deps.querier, overseer, collateral)?;

        Ok(HandleResponse {
            messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                send: vec![],
                contract_addr: market.contract.address,
                callback_code_hash: market.contract.code_hash,
                msg: to_binary(&HandleMsg::Seize {
                    liquidator: liquidator_address,
                    borrower: borrower_address,
                    amount: seize_amount,
                })?,
            })],
            log: vec![],
            data: None,
        })
    }
}

fn seize<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    mut latest: LatestInterest,
    underlying_balance: Uint256,
    liquidator: Account,
    borrower: Account,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    if borrower
        .subtract_balance(&mut deps.storage, amount)
        .is_err()
    {
        return Err(StdError::generic_err(format!(
            "Borrower collateral balance is less than the seize amount. Shortfall: {}",
            (amount - borrower.get_balance(&deps.storage)?)?
        )));
    }

    let config = Constants::load_config(&deps.storage)?;

    let protocol_share = amount.decimal_mul(config.seize_factor)?;
    let liquidator_share = (amount - protocol_share)?;

    let interest_reserve = latest.total_reserves(&deps.storage)?;
    let exchange_rate = calc_exchange_rate(
        deps,
        underlying_balance,
        latest.total_borrows(&deps.storage)?,
        interest_reserve,
    )?;

    let protocol_amount = protocol_share.decimal_mul(exchange_rate)?;

    Global::save_interest_reserve(&mut deps.storage, &(interest_reserve + protocol_amount)?)?;

    TotalSupply::decrease(&mut deps.storage, protocol_share)?;

    liquidator.add_balance(&mut deps.storage, liquidator_share)?;

    Ok(HandleResponse::default())
}
