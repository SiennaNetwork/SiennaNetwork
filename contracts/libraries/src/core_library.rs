use crate::wad_ray_math;

use super::wad_ray_math::WadRayMath;
use cosmwasm_std::{CosmosMsg, Env, StdError, StdResult, Uint128};
use std::ops::{Add, Div, Sub};

enum InterestRateMode {
    Stable,
    Varible,
    None,
}
const SECODNS_PER_YEAR: u128 = 31_536_000;

struct UserReserveData {
    //principal amount borrowed by the user.
    principal_borrow_balance: u128,
    //cumulated variable borrow index for the user. Expressed in ray
    last_variable_borrow_cumulative_index: u128,
    //origination fee cumulated by the user
    origination_fee: u128,
    // stable borrow rate at which the user has borrowed. Expressed in ray
    stable_borrow_rate: u128,
    last_update_timestamp: u64,

    //defines if a specific deposit should or not be used as a collateral in borrows
    use_as_collateral: bool,
}

pub struct ReserveData {
    //the liquidity index. Expressed in ray
    last_liquidity_cumulate_index: u128,
    //the current supply rate. Expressed in ray
    current_liquidity_rate: u128,
    //the total borrows of the reserve at a stable rate. Expressed in the currency decimals
    total_borrow_stable: u128,
    //the total borrows of the reserve at a variable rate. Expressed in the currency decimals
    total_borrows_variable: u128,
    //the current variable borrow rate. Expressed in ray
    current_variable_borrow_rate: u128,
    //the current stable borrow rate. Expressed in ray
    current_stable_borrow_rate: u128,
    //the current average stable borrow rate (weighted average of all the different stable rate loans). Expressed in ray
    current_average_stable_borrow_rate: u128,
    //variable borrow index. Expressed in ray
    last_variable_borrow_cumulative_index: u128,
    //the ltv of the reserve. Expressed in percentage (0-100)
    base_ltv_as_collateral: u128,
    //the liquidation threshold of the reserve. Expressed in percentage (0-100)
    liquidation_threshold: u128,
    //the liquidation bonus of the reserve. Expressed in percentage
    liquidation_bonus: u128,
    //the decimals of the reserve asset
    decimals: u128,

    /*
    address of the aToken representing the asset
     */
    a_token_address: String,

    /*
    address of the interest rate strategy contract
     */
    interest_rate_strategy_address: String,
    last_update_timestamp: u64,
    // borrowingEnabled = true means users can borrow from this reserve
    borrowing_enabled: bool,
    // usageAsCollateralEnabled = true means users can use this reserve as collateral
    usage_as_collateral_enabled: bool,
    // isStableBorrowRateEnabled = true means users can borrow at a stable rate
    is_stable_borrow_rate_enabled: bool,
    // isActive = true means the reserve has been activated and properly configured
    is_active: bool,
    // isFreezed = true means the reserve only allows repays and redeems, but not deposits, new borrowings or rate swap
    is_freezed: bool,
}

impl ReserveData {
    /**
     * @dev returns the ongoing normalized income for the reserve.
     * a value of 1e27 means there is no income. As time passes, the income is accrued.
     * A value of 2*1e27 means that the income of the reserve is double the initial amount.
     * @param _reserve the reserve object
     * @return the normalized income. expressed in ray
     **/
    pub fn get_normolized_income(&self, env: &Env) -> u128 {
        calculate_linear_interest(self.current_liquidity_rate, self.last_update_timestamp, env)
            .ray_mul(self.last_liquidity_cumulate_index)
    }
    /**
     * @dev Updates the liquidity cumulative index Ci and variable borrow cumulative index Bvc. Refer to the whitepaper for
     * a formal specification.
     * @param _self the reserve object
     **/
    pub fn update_cumulative_indexes(&mut self, env: &Env) {
        let total_borrows = _get_total_borrows(&self);
        if total_borrows > 0 {
            let camulated_liquidity_interest = calculate_linear_interest(
                self.current_liquidity_rate,
                self.last_update_timestamp,
                env,
            );
            self.last_variable_borrow_cumulative_index =
                camulated_liquidity_interest.ray_mul(self.last_variable_borrow_cumulative_index);
            let cumulated_varible_borrow_interest = calculate_compouned_interest(
                self.current_variable_borrow_rate,
                self.last_update_timestamp,
                env,
            );
            self.last_variable_borrow_cumulative_index = cumulated_varible_borrow_interest
                .ray_mul(self.last_variable_borrow_cumulative_index);
        }
    }

    /**
     * @dev accumulates a predefined amount of asset to the reserve as a fixed, one time income. Used for example to accumulate
     * the flashloan fee to the reserve, and spread it through the depositors.
     * @param _self the reserve object
     * @param _totalLiquidity the total liquidity available in the reserve
     * @param _amount the amount to accomulate
     **/
    pub fn cumulated_to_liquidity_index(&mut self, _total_liquidity: u128, _amount: u128) {
        let amount_to_liquidity_ratio = _amount.wad_to_ray().ray_div(_total_liquidity.wad_to_ray());

        let cumulated_liquidity = amount_to_liquidity_ratio.add(wad_ray_math::ray());

        self.last_liquidity_cumulate_index =
            cumulated_liquidity.ray_mul(self.last_liquidity_cumulate_index);
    }

    /**
     * @dev initializes a reserve
     * @param _self the reserve object
     * @param _aTokenAddress the address of the overlying atoken contract
     * @param _decimals the number of decimals of the underlying asset
     * @param _interestRateStrategyAddress the address of the interest rate strategy contract
     **/
    pub fn init(
        &mut self,
        _a_token_address: String,
        _decimals: u128,
        _interested_rate_strategy_address: String,
    ) -> StdResult<()> {
        if self.a_token_address.is_empty() {
            return Err(StdError::generic_err(
                "Reserve has already been initialized",
            ));
        }

        if self.last_liquidity_cumulate_index == 0 {
            self.last_liquidity_cumulate_index = wad_ray_math::ray();
        }

        if self.last_variable_borrow_cumulative_index == 0 {
            self.last_variable_borrow_cumulative_index = wad_ray_math::ray();
        }
        self.a_token_address = _a_token_address;
        self.decimals = _decimals;

        self.interest_rate_strategy_address = _interested_rate_strategy_address;
        self.is_active = true;
        self.is_freezed = false;

        Ok(())
    }

    pub fn get_total_borrows(&self) -> u128 {
        _get_total_borrows(self)
    }
}

/**
 * @dev Updates the liquidity cumulative index Ci and variable borrow cumulative index Bvc. Refer to the whitepaper for
 * a formal specification.
 * @param _self the reserve object
**/

//pub fn

pub fn calculate_linear_interest(_rate: u128, _last_update_timestamp: u64, env: &Env) -> u128 {
    //solim-disable-next-line
    let time_defference = env.block.time.sub(_last_update_timestamp) as u128;

    let time_delta = time_defference
        .wad_to_ray()
        .ray_div(SECODNS_PER_YEAR.wad_to_ray());

    _rate.ray_mul(time_delta).add(wad_ray_math::ray())
}

pub fn calculate_compouned_interest(_rate: u128, _last_update_timestamp: u64, env: &Env) -> u128 {
    let time_defference = env.block.time.sub(_last_update_timestamp) as u128;

    let rate_pre_second = _rate.div(SECODNS_PER_YEAR);

    rate_pre_second
        .add(wad_ray_math::ray())
        .ray_pow(time_defference)
}
/**
 * @dev returns the total borrows on the reserve
 * @param _reserve the reserve object
 * @return the total borrows (stable + variable)
 **/

fn _get_total_borrows(_reserve: &ReserveData) -> u128 {
    _reserve
        .total_borrow_stable
        .add(_reserve.total_borrows_variable)
}
