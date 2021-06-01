use cosmwasm_std::{Binary, HumanAddr, ReadonlyStorage, Storage};

use crate::state::get_address;

pub const LENDING_POOL: &[u8] = b"LENDING_POOL";
pub const LENDING_POOL_CORE: &[u8] = b"LENDING_POOL_CORE";
pub const LENDING_POOL_CONFIGURATOR: &[u8] = b"LENDING_POOL_CONFIGURATOR";
pub const LENDING_POOL_PARAMETETRS_PROVIDER: &[u8] = b"PARAMETRS_PROVIDER";
pub const LENDING_POOL_MANAGER: &[u8] = b"LENDING_POOL_MANAGER";
pub const LENDING_POOL_LIQUIDATION_MANAGER: &[u8] = b"LIQUIDATION_MANAGER";
pub const LENDING_POOL_FLASHLOAN_PROVIDER: &[u8] = b"FLASHLOAN_PROVIDER";
pub const DATA_PROVIDER: &[u8] = b"DATA_PROVIDER";
pub const ETHEREUM_ADDRESS: &[u8] = b"ETHEREUM_ADDRESS";
pub const PRICE_ORACLE: &[u8] = b"PRICE_ORACLE";
pub const LENDING_RATE_ORACLE: &[u8] = b"LENDING_RATE_ORACLE";
pub const FEE_PROVIDER: &[u8] = b"FEE_PROVIDER";
pub const WALLET_BALANCE_PROVIDER: &[u8] = b"WALLET_BALANCE_PROVIDER";
pub const TOKEN_DISTRIBUTOR: &[u8] = b"TOKEN_DISTRIBUTOR";

/**
 * @dev returns the address of the LendingPool proxy
 * @return the lending pool proxy address
 **/
pub fn get_lending_pool<S: ReadonlyStorage>(storage: &S) -> Option<Binary> {
    let res = get_address(storage, LENDING_POOL)?;
    Some(res)
}

/**
 * @dev internal function to update the implementation of a specific component of the protocol
 * @param _id the id of the contract to be updated
 * @param _newAddress the address of the new implementation
 **/
pub fn update_impl_internal<S: Storage>(storage: &mut S, _id: &[u8], address: &HumanAddr) { // unimplemented!
    let proxy_address = get_address(storage, _id); 
}
