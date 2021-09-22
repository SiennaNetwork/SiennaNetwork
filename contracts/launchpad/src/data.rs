use amm_shared::fadroma::scrt::{
    addr::{Canonize, Humanize},
    callback::ContractInstance,
    cosmwasm_std::{
        Api, CanonicalAddr, Decimal, Extern, HumanAddr, Querier, StdError, StdResult, Storage,
        Uint128,
    },
    storage::{load, save, Storable},
    utils::viewing_key::ViewingKey,
};
use amm_shared::{msg::launchpad::TokenSettings, TokenType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::helpers::*;

const KEY_CONTRACT_ADDR: &[u8] = b"this_contract";
const KEY_VIEWING_KEY: &[u8] = b"launchpad_viewing_key";
const KEY_ACCOUNTS_VEC_LENGTH: &[u8] = b"accounts:vec_length";

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
    mut account: Account,
) -> StdResult<()> {
    // If the account doesn't have its num_in_vec we'll set it to the end of len
    // and we'll increment the vec_length for next time.
    if account.num_in_vec.is_none() {
        let mut vec_length: u32 = load(&deps.storage, KEY_ACCOUNTS_VEC_LENGTH)?.unwrap_or(0);
        account.num_in_vec = Some(vec_length);
        vec_length += 1;

        // Increment accounts vec length
        save(&mut deps.storage, KEY_ACCOUNTS_VEC_LENGTH, &vec_length)?;

        // Store the address together with its index in Vec
        let canonical_address = account.owner.canonize(&deps.api)?;
        save(
            &mut deps.storage,
            canonical_address.as_slice(),
            &account.num_in_vec,
        )?;
    }

    save(
        &mut deps.storage,
        format!("accounts:{}", account.num_in_vec.unwrap()).as_bytes(),
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

/// Load account or create a new one
pub(crate) fn load_account<S: Storage, T: Api, Q: Querier>(
    deps: &Extern<S, T, Q>,
    address: &HumanAddr,
) -> StdResult<Account> {
    let canonical_address = address.canonize(&deps.api)?;
    let index_in_vec: Option<u32> = load(&deps.storage, canonical_address.as_slice())?;

    match index_in_vec {
        Some(i) => load(&deps.storage, format!("accounts:{}", i).as_bytes())?
            .ok_or_else(|| StdError::generic_err("Account not found")),
        None => Err(StdError::generic_err("Account not found")),
    }
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
                ContractInstance {
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
            active: true,
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
/// Token configuration, once set cannot be updated, only enabled/disabled.
///
/// Reason for no update is that we have to hold each tokens entry representation
/// in the users account so we can keep track of the bounding time so those entries
/// are not really fungible to be scaled up or down easily.
///
/// TBD: maybe we could add removing of the token config and then re-adding it in order
/// to give it some extra options, but then the removing option would have to re-calculate
/// everything for all the addresses that were participating and their amounts... Or even
/// send those tokens back to users and then they can lock them again to participate in the
/// future...
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct TokenConfig {
    pub token_type: TokenType<HumanAddr>,
    pub segment: Uint128,
    pub bounding_period: u64,
    pub active: bool,
    pub token_decimals: u8,
}

impl TokenConfig {
    pub fn activate(&mut self) {
        self.active = true;
    }
    pub fn disable(&mut self) {
        self.active = false;
    }
}

/// Holder of all the accounts that have participated in the launchpad,
/// this holder is just an internal struct that is never saved as it is.
pub struct Accounts {
    pub accounts: Vec<Account>,
    pub vec_length: u32,
}

impl Accounts {
    /// Load the full hash map where we need to get all the values of the data vec
    pub fn load<S: Storage, T: Api, Q: Querier>(deps: &Extern<S, T, Q>) -> StdResult<Accounts> {
        let vec_length: u32 = load(&deps.storage, KEY_ACCOUNTS_VEC_LENGTH)?.unwrap_or(0);
        let mut accounts: Vec<Account> = vec![];

        for n in 0..vec_length {
            // We will unwrap the account and create a dummy account, just so we don't have
            // to have an error here that would put us in contract unrecoverable state,
            // we will filter those with an if statement below.
            let account = load(&deps.storage, format!("accounts:{}", n).as_bytes())?
                .unwrap_or_else(|| Account::new(&HumanAddr::from("")));

            if account.num_in_vec.is_some() {
                accounts.push(account);
            }
        }

        Ok(Accounts {
            vec_length,
            accounts,
        })
    }
}

/// Single account that has participated in the launchpad
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Account {
    pub owner: HumanAddr,
    pub tokens: Vec<AccountToken>,
    pub num_in_vec: Option<u32>,
}

impl Account {
    pub fn new(address: &HumanAddr) -> Self {
        Account {
            owner: address.clone(),
            tokens: vec![],
            num_in_vec: None,
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
                // Logic that will exclude draws that are in the cooldown period if this
                // option is enabled some time in the future.
                // if let Some(time) = token.last_draw {
                //     if let Some(cooldown) = token_config.cooldown_period {
                //         if time > (token.last_draw + cooldown_period) && !token.cooldown_disabled {
                //             continue;
                //         }
                //     }
                // }
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

    /// Mark account tokens with last draw timestamp
    pub fn mark_as_drawn(&mut self, token_configs: &Vec<TokenConfig>, timestamp: u64) {
        for token_config in token_configs {
            for token in &mut self.tokens {
                if token.token_type == token_config.token_type {
                    token.last_draw = Some(timestamp);
                }
            }
        }
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
            last_draw: None,
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
    pub last_draw: Option<u64>,
}

/// Token entry type is a representation of u64 timestamp
/// that will be time when this entry was created.
pub type AccounTokenEntry = u64;
