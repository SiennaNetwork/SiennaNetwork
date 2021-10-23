use amm_shared::fadroma::{
    scrt_addr::{Canonize, Humanize},
    scrt_link::ContractLink,
    scrt::{
        Api, CanonicalAddr, Decimal, Extern, HumanAddr,
        Querier, StdError, StdResult, Storage, Uint128
    },
    scrt_storage::{load, ns_load, ns_save, save},
    scrt_storage_traits::Storable,
    scrt_vk::ViewingKey,
};
use amm_shared::{msg::launchpad::TokenSettings, TokenType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::helpers::*;

const KEY_CONTRACT_ADDR: &[u8] = b"this_contract";
const KEY_VIEWING_KEY: &[u8] = b"launchpad_viewing_key";
const KEY_ACCOUNTS_VEC_LENGTH: &[u8] = b"accounts:vec_length";
const NS_ACCOUNTS: &[u8] = b"accounts";
const NS_ACCOUNT_INDEXES: &[u8] = b"account_indexes";

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

pub(crate) fn load_viewing_key(storage: &impl Storage) -> StdResult<ViewingKey> {
    let vk: ViewingKey = load(storage, KEY_VIEWING_KEY)?.unwrap();

    Ok(vk)
}

pub(crate) fn save_viewing_key(storage: &mut impl Storage, vk: &ViewingKey) -> StdResult<()> {
    save(storage, KEY_VIEWING_KEY, vk)
}

/// Save account after it has been updated or created
pub(crate) fn save_account<S: Storage, T: Api, Q: Querier>(
    deps: &mut Extern<S, T, Q>,
    account: Account,
) -> StdResult<()> {
    // Match the possible index to get index even if it doesn't exist
    let index_in_vec = match load_account_index(&deps, &account.owner)? {
        Some(i) => i,
        None => {
            // Load the vec length
            let i = load(&deps.storage, KEY_ACCOUNTS_VEC_LENGTH)?.unwrap_or(0);

            // Save the index for account
            save_account_index(deps, &account.owner, i)?;

            // Increment accounts vec length
            save(&mut deps.storage, KEY_ACCOUNTS_VEC_LENGTH, &(i + 1))?;

            i
        }
    };

    // Finally, save the account
    ns_save(
        &mut deps.storage,
        NS_ACCOUNTS,
        &index_in_vec.to_be_bytes(),
        &account,
    )
}

/// Load account or create a new one
pub(crate) fn load_or_create_account<S: Storage, T: Api, Q: Querier>(
    deps: &Extern<S, T, Q>,
    address: &HumanAddr,
) -> StdResult<Account> {
    match load_account(deps, address) {
        Ok(account) => Ok(account),
        Err(_) => Ok(Account::new(address)),
    }
}

/// Load the account address index where its saved
pub(crate) fn load_account_index<S: Storage, T: Api, Q: Querier>(
    deps: &Extern<S, T, Q>,
    address: &HumanAddr,
) -> StdResult<Option<u32>> {
    let canonical_address = address.canonize(&deps.api)?;
    let index_in_vec: Option<u32> = ns_load(
        &deps.storage,
        NS_ACCOUNT_INDEXES,
        canonical_address.as_slice(),
    )?;

    Ok(index_in_vec)
}

/// Map the address with an index
pub(crate) fn save_account_index<S: Storage, T: Api, Q: Querier>(
    deps: &mut Extern<S, T, Q>,
    address: &HumanAddr,
    index: u32,
) -> StdResult<()> {
    let canonical_address = address.canonize(&deps.api)?;
    ns_save(
        &mut deps.storage,
        NS_ACCOUNT_INDEXES,
        canonical_address.as_slice(),
        &index,
    )
}

/// Load account or create a new one
pub(crate) fn load_account<S: Storage, T: Api, Q: Querier>(
    deps: &Extern<S, T, Q>,
    address: &HumanAddr,
) -> StdResult<Account> {
    let index_in_vec = load_account_index(deps, address)?;

    match index_in_vec {
        Some(i) => ns_load(&deps.storage, NS_ACCOUNTS, &i.to_be_bytes())?
            .ok_or_else(|| StdError::generic_err("Account not found")),
        None => Err(StdError::generic_err("Account not found")),
    }
}

/// Load the full hash map where we need to get all the values of the data vec
pub fn load_all_accounts<S: Storage, T: Api, Q: Querier>(
    deps: &Extern<S, T, Q>,
) -> StdResult<Vec<Account>> {
    let vec_length: u32 = load(&deps.storage, KEY_ACCOUNTS_VEC_LENGTH)?.unwrap_or(0);
    let mut accounts: Vec<Account> = Vec::with_capacity(vec_length as usize);

    for n in 0..vec_length {
        // We will unwrap the account and create a dummy account, just so we don't have
        // to have an error here that would put us in contract unrecoverable state,
        // we will filter those with an if statement below.
        let account = ns_load(&deps.storage, NS_ACCOUNTS, &n.to_be_bytes())?.unwrap();
        accounts.push(account);
    }

    Ok(accounts)
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub tokens: Vec<TokenConfig>,
}

impl Storable for Config {
    fn namespace() -> Vec<u8> {
        b"config".to_vec()
    }
    /// Setting the empty key because config is only one
    fn key(&self) -> StdResult<Vec<u8>> {
        Ok(Vec::new())
    }
}

impl Config {
    pub fn load_self<S: Storage, T: Api, Q: Querier>(deps: &Extern<S, T, Q>) -> StdResult<Config> {
        let result = Config::load(deps, b"")?;
        let result =
            result.ok_or_else(|| StdError::generic_err("Config doesn't exist in storage."))?;

        Ok(result)
    }

    /// Return the token config based on the given address
    pub fn get_token_config(&self, token_address: Option<HumanAddr>) -> StdResult<TokenConfig> {
        for token_config in &self.tokens {
            if token_address.is_none() && token_config.token_type.is_native_token() {
                return Ok(token_config.clone());
            }

            if let Some(address) = &token_address {
                match &token_config.token_type {
                    TokenType::CustomToken { contract_addr, .. } => {
                        if contract_addr == address {
                            return Ok(token_config.clone());
                        }
                    }
                    _ => (),
                }
            }
        }

        Err(StdError::generic_err("Token not supported"))
    }

    /// Add new token in the config
    pub fn add_token(&mut self, querier: &impl Querier, new_token: TokenSettings) -> StdResult<()> {
        for token in &self.tokens {
            if token.token_type == new_token.token_type {
                return Err(StdError::generic_err("Token already exists"));
            }
        }

        let token_decimals = match &new_token.token_type {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => get_token_decimals(
                querier,
                ContractLink {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
            )?,
            _ => 6,
        };

        self.tokens.push(TokenConfig {
            token_type: new_token.token_type,
            segment: new_token.segment,
            bounding_period: new_token.bounding_period,
            token_decimals,
        });

        Ok(())
    }

    /// Remove the token from the config
    pub fn remove_token(&mut self, index: u32) -> StdResult<TokenConfig> {
        if self.tokens.len() <= index as usize {
            return Err(StdError::generic_err("Token not found"));
        }
        Ok(self.tokens.remove(index as usize))
    }
}

/// Configuration for single token that can be locked into the launchpad
/// Token configuration, once set cannot be updated, only can be removed.
/// Once it is removed it will refund all its locked tokens back to users.
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct TokenConfig {
    pub token_type: TokenType<HumanAddr>,
    pub segment: Uint128,
    pub bounding_period: u64,
    pub token_decimals: u8,
}

/// Single account that has participated in the launchpad
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Account {
    pub owner: HumanAddr,
    pub tokens: Vec<AccountToken>,
}

impl Account {
    pub fn new(address: &HumanAddr) -> Self {
        Account {
            owner: address.clone(),
            tokens: vec![],
        }
    }

    /// Return a single layer depth vector with all the entries for this account
    pub fn get_entries(
        &self,
        token_configs: &Vec<TokenConfig>,
        timestamp: u64,
    ) -> Vec<(HumanAddr, AccounTokenEntry)> {
        let mut entries: Vec<(HumanAddr, AccounTokenEntry)> = vec![];

        for token_config in token_configs {
            for token in &self.tokens {
                if token.token_type == token_config.token_type {
                    for entry in &token.entries {
                        // Entry is acutally only a timestamp when it was added, so we add bonding time
                        // to it and check that the time we get is less then the current timestamp,
                        // meaning the bonding period for that entry is over.
                        if (entry + token_config.bounding_period) <= timestamp {
                            entries.push((self.owner.clone(), *entry));
                        }
                    }
                }
            }
        }

        entries
    }

    /// Lock funds in the account
    pub fn lock(
        &mut self,
        timestamp: u64,
        token_config: &TokenConfig,
        amount: Uint128,
    ) -> StdResult<(Uint128, u32)> {
        if amount < token_config.segment {
            return Err(StdError::generic_err(format!(
                "Amount is lower then the minimum segment amount of {}",
                token_config.segment
            )));
        }

        // Figure out how many entries this deposit represents
        let number_of_entry = calculate_entries(amount, token_config.segment);
        let mut entries = vec![];

        // Push the current timestamp for each entry
        for _n in 0..number_of_entry {
            entries.push(timestamp);
        }

        // Calculate the return change of the amount
        let change_amount =
            (amount - (token_config.segment * Decimal::from_str(&number_of_entry.to_string())?))?;

        // We'll try to find the token in the account if it already has it.
        for account_token in &mut self.tokens {
            if account_token.token_type == token_config.token_type {
                account_token.entries.append(&mut entries);

                // Increment the acconts token balance for the amount left
                account_token.balance += (amount - change_amount)?;

                return Ok((change_amount, number_of_entry as u32));
            }
        }

        let account_token = AccountToken {
            token_type: token_config.token_type.clone(),
            balance: (amount - change_amount)?,
            entries,
        };

        self.tokens.push(account_token);

        return Ok((change_amount, number_of_entry as u32));
    }

    /// Unlock the funds corresponding to entries from the account
    pub fn unlock(
        &mut self,
        token_config: &TokenConfig,
        entries: u32,
    ) -> StdResult<(Uint128, u32)> {
        // We'll try to find the token in the account
        for mut account_token in &mut self.tokens {
            if account_token.token_type == token_config.token_type {
                if (account_token.entries.len() as u32) < entries {
                    return Err(StdError::generic_err("Insufficient entries in the account"));
                }

                let mut removed = 0;

                for _n in 0..entries {
                    if !account_token.entries.is_empty() {
                        account_token.entries.remove(0);
                        removed += 1;
                    }
                }

                let amount = token_config.segment * Decimal::from_str(&removed.to_string())?;
                account_token.balance = (account_token.balance - amount)?;

                return Ok((amount, removed));
            }
        }

        return Err(StdError::generic_err("Invalid token provided"));
    }

    /// Remove all entries for a single token
    pub fn unlock_all(&mut self, token_config: &TokenConfig) -> StdResult<(Uint128, u32)> {
        let entries = self.get_entries(&vec![token_config.clone()], 99999999999_u64);
        self.unlock(&token_config, entries.len() as u32)
    }
}

/// Account token representation that holds all the entries for this token
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AccountToken {
    pub token_type: TokenType<HumanAddr>,
    pub balance: Uint128,
    pub entries: Vec<AccounTokenEntry>,
}

/// Token entry type is a representation of u64 timestamp
/// that will be time when this entry was created.
pub type AccounTokenEntry = u64;
