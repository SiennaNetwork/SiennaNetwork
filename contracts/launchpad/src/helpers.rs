use crate::data::TokenConfig;
use amm_shared::{
    fadroma::scrt::{
        callback::ContractInstance,
        cosmwasm_std::{
            Api, BankMsg, Coin, CosmosMsg, Decimal, Extern, HumanAddr, Querier, StdResult, Storage,
            Uint128,
        },
        toolkit::snip20,
        utils::viewing_key::ViewingKey,
        BLOCK_SIZE,
    },
    TokenType,
};

/// Query the token for number of its decimals
pub(crate) fn get_token_decimals(
    querier: &impl Querier,
    instance: ContractInstance<HumanAddr>,
) -> StdResult<u8> {
    let result =
        snip20::token_info_query(querier, BLOCK_SIZE, instance.code_hash, instance.address)?;

    Ok(result.decimals)
}

/// Query the token for number of its decimals
pub(crate) fn get_token_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    this_contract: HumanAddr,
    instance: ContractInstance<HumanAddr>,
    viewing_key: ViewingKey,
) -> StdResult<Uint128> {
    let balance = snip20::balance_query(
        &deps.querier,
        this_contract,
        viewing_key.0,
        BLOCK_SIZE,
        instance.code_hash,
        instance.address,
    )?;

    Ok(balance.amount)
}

/// Helper to calculate and floor the number of enteries in an amount
pub(crate) fn calculate_entries(amount: Uint128, segment: Uint128) -> u32 {
    if amount < segment {
        return 0;
    }

    let entries: u32 = Decimal::from_ratio(amount, segment)
        .to_string()
        .split(".")
        .collect::<Vec<&str>>()
        .get(0)
        .unwrap_or(&"0")
        .parse()
        .unwrap_or(0_u32);

    entries
}

/// Create transfer mmessage if the amount is more then zero
pub(crate) fn create_transfer_message(
    token_config: &TokenConfig,
    messages: &mut Vec<CosmosMsg>,
    from_address: HumanAddr,
    to_address: HumanAddr,
    amount: Uint128,
) -> StdResult<()> {
    if !amount.is_zero() {
        match &token_config.token_type {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => messages.push(snip20::transfer_msg(
                to_address,
                amount,
                None,
                BLOCK_SIZE,
                token_code_hash.clone(),
                contract_addr.clone(),
            )?),
            TokenType::NativeToken { denom } => messages.push(
                BankMsg::Send {
                    from_address,
                    to_address,
                    amount: vec![Coin::new(amount.u128(), &denom)],
                }
                .into(),
            ),
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_entry_calculation() {
        let amount = Uint128(1000_000_000_u128);
        let segment = Uint128(270_000_000_u128);

        let entries = calculate_entries(amount, segment);

        assert_eq!(entries, 3_u32);
    }
}
