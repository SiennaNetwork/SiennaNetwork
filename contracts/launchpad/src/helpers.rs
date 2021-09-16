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
// use chrono::Utc;
// use rand::{rngs::SmallRng, RngCore, SeedableRng};

/// Wasm safe random number generator in a given range
pub(crate) fn gen_rand_range(mut min: u64, mut max: u64, seed: Option<u64>) -> u64 {
    max
    // if min == max {
    //     return min;
    // }

    // if min > max {
    //     let temp = min;
    //     max = min;
    //     min = temp;
    // }
    // let mut small_rng =
    //     SmallRng::seed_from_u64(seed.unwrap_or_else(|| Utc::now().timestamp_millis() as u64));
    // let picked_random: u64 = small_rng.next_u32() as u64;

    // let percentage: u64 = (picked_random * 100_u64) / u32::MAX as u64;
    // (percentage * (max - min) / 100_u64) + min
}

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

    #[test]
    fn test_random_number_generation() {
        let from_to = vec![
            (1, 1),
            (5, 10),
            (2000, 4000),
            (333, 500),
            (12323423, 13323423),
        ];

        for i in from_to {
            let num = gen_rand_range(i.0 as u64, i.1 as u64, None);
            println!("{} >= ({} as u64) && {} <= ({} as u64)", num, i.0, num, i.1);
            assert!(num >= (i.0 as u64) && num <= (i.1 as u64));
        }
    }
}
