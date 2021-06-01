pub const MAX_STABLE_RATE_BORROW_SIZE_PERCENT: u128 = 25;
pub const REBALANCE_DOWN_RATE_DELTA: u128 = (1 * 10_u128.pow(27)) / 5;
pub const FLASHLOAN_FEE_TOTAL: u128 = 35;
pub const FLASHLOAN_FEE_PROTOCOL: u128 = 3000;

pub const DATA_PROVIDER_REVISION: u128 = 0x1;

pub fn get_revision() -> u128 {
    DATA_PROVIDER_REVISION
}

/**
 * @dev initializes the LendingPoolParametersProvider after it's added to the proxy
 * @param _addressesProvider the address of the LendingPoolAddressesProvider
 */
pub fn initilize(_address_provider: &str) {}

/**
 * @dev returns the maximum stable rate borrow size, in percentage of the available liquidity.
 **/
pub fn get_max_stable_rate_borrow_size_precent() -> u128 {
    MAX_STABLE_RATE_BORROW_SIZE_PERCENT
}

/**
 * @dev returns the delta between the current stable rate and the user stable rate at
 *      which the borrow position of the user will be rebalanced (scaled down)
 **/
pub fn get_rebalance_down_rate_delta() -> u128 {
    REBALANCE_DOWN_RATE_DELTA
}

/**
 * @dev returns the fee applied to a flashloan and the portion to redirect to the protocol, in basis points.
 **/
pub fn get_flash_loan_fees_in_bips() -> (u128, u128) {
    (FLASHLOAN_FEE_TOTAL, FLASHLOAN_FEE_PROTOCOL)
}
