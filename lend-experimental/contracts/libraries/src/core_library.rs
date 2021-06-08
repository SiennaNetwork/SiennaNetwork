use super::uint256::Uint256;
use crate::wad_ray_math::{self, WadRayMathU256};
use bigint::U256;
use cosmwasm_std::{Env, StdError, StdResult, Uint128};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Sub};
pub enum InterestRateMode {
    Stable,
    Varible,
    None,
}
lazy_static! {
    static ref SECODNS_PER_YEAR: U256 = U256::from(31_536_000);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserReserveData {
    //principal amount borrowed by the user.
    principal_borrow_balance: Uint256,
    //cumulated variable borrow index for the user. Expressed in ray
    last_variable_borrow_cumulative_index: Uint256,
    //origination fee cumulated by the user
    origination_fee: Uint256,
    // stable borrow rate at which the user has borrowed. Expressed in ray
    stable_borrow_rate: Uint256,
    last_update_timestamp: u64,

    //defines if a specific deposit should or not be used as a collateral in borrows
    use_as_collateral: bool,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveData {
    //the liquidity index. Expressed in ray
    last_liquidity_cumulate_index: Uint256,
    //the current supply rate. Expressed in ray
    current_liquidity_rate: Uint256,
    //the total borrows of the reserve at a stable rate. Expressed in the currency decimals
    total_borrow_stable: Uint256,
    //the total borrows of the reserve at a variable rate. Expressed in the currency decimals
    total_borrows_variable: Uint256,
    //the current variable borrow rate. Expressed in ray
    current_variable_borrow_rate: Uint256,
    //the current stable borrow rate. Expressed in ray
    current_stable_borrow_rate: Uint256,
    //the current average stable borrow rate (weighted average of all the different stable rate loans). Expressed in ray
    current_average_stable_borrow_rate: Uint256,
    //variable borrow index. Expressed in ray
    last_variable_borrow_cumulative_index: Uint256,
    //the ltv of the reserve. Expressed in percentage (0-100)
    base_ltv_as_collateral: Uint256,
    //the liquidation threshold of the reserve. Expressed in percentage (0-100)
    liquidation_threshold: Uint256,
    //the liquidation bonus of the reserve. Expressed in percentage
    liquidation_bonus: Uint256,
    //the decimals of the reserve asset
    decimals: Uint256,

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
    pub fn get_normolized_income(&self, env: &Env) -> U256 {
        calculate_linear_interest(
            self.current_liquidity_rate.0,
            self.last_update_timestamp,
            env,
        )
        .ray_mul(self.last_liquidity_cumulate_index.0)
    }
    /**
     * @dev Updates the liquidity cumulative index Ci and variable borrow cumulative index Bvc. Refer to the whitepaper for
     * a formal specification.
     * @param _self the reserve object
     **/
    pub fn update_cumulative_indexes(&mut self, env: &Env) {
        let total_borrows = _get_total_borrows(&self);
        if total_borrows > U256::from(0) {
            let camulated_liquidity_interest = calculate_linear_interest(
                self.current_liquidity_rate.0,
                self.last_update_timestamp,
                env,
            );
            self.last_variable_borrow_cumulative_index = Uint256::from(
                camulated_liquidity_interest.ray_mul(self.last_variable_borrow_cumulative_index.0),
            );
            let cumulated_varible_borrow_interest = calculate_compouned_interest(
                self.current_variable_borrow_rate,
                self.last_update_timestamp,
                env,
            );
            self.last_variable_borrow_cumulative_index = Uint256::from(
                cumulated_varible_borrow_interest
                    .ray_mul(self.last_variable_borrow_cumulative_index.0),
            );
        }
    }

    /**
     * @dev accumulates a predefined amount of asset to the reserve as a fixed, one time income. Used for example to accumulate
     * the flashloan fee to the reserve, and spread it through the depositors.
     * @param _self the reserve object
     * @param _totalLiquidity the total liquidity available in the reserve
     * @param _amount the amount to accomulate
     **/
    pub fn cumulated_to_liquidity_index(&mut self, _total_liquidity: U256, _amount: U256) {
        let amount_to_liquidity_ratio = _amount.wad_to_ray().ray_div(_total_liquidity.wad_to_ray());

        let cumulated_liquidity = amount_to_liquidity_ratio.add(wad_ray_math::ray_256());

        self.last_liquidity_cumulate_index =
            Uint256::from(cumulated_liquidity.ray_mul(self.last_liquidity_cumulate_index.0));
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
        _decimals: Uint256,
        _interested_rate_strategy_address: String,
    ) -> StdResult<()> {
        if !self.a_token_address.is_empty() {
            return Err(StdError::generic_err(
                "Reserve has already been initialized",
            ));
        }

        if self.last_liquidity_cumulate_index.0 == U256::from(0) {
            self.last_liquidity_cumulate_index = wad_ray_math::ray_256().into();
        }

        if self.last_variable_borrow_cumulative_index == U256::from(0).into() {
            self.last_variable_borrow_cumulative_index = Uint256::from(wad_ray_math::ray_256());
        }
        self.a_token_address = _a_token_address;
        self.decimals = _decimals;

        self.interest_rate_strategy_address = _interested_rate_strategy_address;
        self.is_active = true;
        self.is_freezed = false;

        Ok(())
    }

    pub fn get_total_borrows(&self) -> U256 {
        _get_total_borrows(self)
    }

    /**
     * @dev enables borrowing on a reserve
     * @param _self the reserve object
     * @param _stableBorrowRateEnabled true if the stable borrow rate must be enabled by default, false otherwise
     **/

    pub fn enable_borrowing(&mut self, _stable_borrow_rate_enable: bool) -> StdResult<()> {
        if self.borrowing_enabled == true {
            return Err(StdError::generic_err("Reserve is already enabled"));
        }
        self.borrowing_enabled = true;
        self.is_stable_borrow_rate_enabled = _stable_borrow_rate_enable;
        Ok(())
    }

    /**
     * @dev disables borrowing on a reserve
     * @param _self the reserve object
     **/

    pub fn disable_borrowing(&mut self) {
        self.borrowing_enabled = false;
    }

    /**
     * @dev enables a reserve to be used as collateral
     * @param _self the reserve object
     * @param _baseLTVasCollateral the loan to value of the asset when used as collateral
     * @param _liquidationThreshold the threshold at which loans using this asset as collateral will be considered undercollateralized
     * @param _liquidationBonus the bonus liquidators receive to liquidate this asset
     **/

    pub fn enable_as_collateral(
        &mut self,
        _base_ltv_as_collateral: Uint256,
        _liqudation_threshold: Uint256,
        _liqudation_bouns: Uint256,
    ) -> StdResult<()> {
        if self.borrowing_enabled == true {
            return Err(StdError::generic_err(
                "Reserve is already enabled as collateral",
            ));
        }
        self.usage_as_collateral_enabled = true;
        self.base_ltv_as_collateral = _base_ltv_as_collateral;
        self.liquidation_threshold = _liqudation_threshold;
        self.liquidation_bonus = _liqudation_bouns;

        if self.last_liquidity_cumulate_index == U256::from(0).into() {
            self.last_liquidity_cumulate_index = wad_ray_math::ray_256().into();
        }
        Ok(())
    }

    /**
     * @dev disables a reserve as collateral
     * @param _self the reserve object
     **/

    pub fn disable_as_collateral(&mut self) {
        self.usage_as_collateral_enabled = false;
    }

    pub fn increase_total_borrows_stable_and_update_averege_rate(
        &mut self,
        _amount: Uint256,
        _rate: Uint256,
    ) {
        let previus_total_borrow_stable = self.total_borrow_stable;
        //updating reserve borrows stable
        self.total_borrow_stable = self.total_borrow_stable.add(_amount);

        //update the average stable rate
        //weighted average of all the borrows
        let weighted_last_borrows = _amount.0.wad_to_ray().ray_mul(_rate.0);
        let _weighted_previous_total_borrows = previus_total_borrow_stable
            .0
            .wad_to_ray()
            .ray_mul(self.current_average_stable_borrow_rate.0);
        self.current_average_stable_borrow_rate = Uint256::from(
            weighted_last_borrows
                .add(weighted_last_borrows)
                .ray_div(self.total_borrow_stable.0.wad_to_ray()),
        );
    }
    /**
     * @dev decreases the total borrows at a stable rate on a specific reserve and updates the
     * average stable rate consequently
     * @param _reserve the reserve object
     * @param _amount the amount to substract to the total borrows stable
     * @param _rate the rate at which the amount has been repaid
     **/
    pub fn decrease_total_borrows_stable_and_update_average_rate(
        &mut self,
        _amount: Uint256,
        _rate: Uint256,
    ) -> StdResult<()> {
        if self.total_borrow_stable < _amount {
            return Err(StdError::generic_err("Invalid amount to decrease"));
        }

        let previus_total_borrow_stable = self.total_borrow_stable;

        //updating reserve borrows stable
        self.total_borrow_stable = self.total_borrow_stable.sub(_amount);
        if self.total_borrow_stable == U256::from(0).into() {
            self.current_average_stable_borrow_rate = U256::from(0).into();
            return Ok(());
        }

        //update the average stable rate
        //weighted average of all the borrows
        let weighted_last_borrow = _amount.0.wad_to_ray().ray_mul(_rate.0);
        let weighted_previous_total_borrows = previus_total_borrow_stable
            .0
            .wad_to_ray()
            .ray_mul(self.current_average_stable_borrow_rate.0);
        if weighted_previous_total_borrows < weighted_last_borrow {
            return Err(StdError::generic_err("The amounts to subtract don't match"));
        }

        self.current_average_stable_borrow_rate = Uint256::from(
            weighted_previous_total_borrows
                .sub(weighted_last_borrow)
                .ray_div(self.total_borrow_stable.0.wad_to_ray()),
        );

        Ok(())
    }

    /**
     * @dev increases the total borrows at a variable rate
     * @param _reserve the reserve object
     * @param _amount the amount to add to the total borrows variable
     **/
    pub fn increase_total_borrows_variable(&mut self, _amount: Uint256) {
        self.total_borrows_variable = self.total_borrows_variable.add(_amount);
    }

    /**
     * @dev decreases the total borrows at a variable rate
     * @param _reserve the reserve object
     * @param _amount the amount to substract to the total borrows variable
     **/

    pub fn decrease_total_borrows_varible(&mut self, _amount: Uint256) -> StdResult<()> {
        if self.total_borrows_variable < _amount {
            return Err(StdError::generic_err(
                "The amount that is being subtracted from the variable total borrows is incorrect",
            ));
        }
        Ok(())
    }
}

impl UserReserveData {
    /**
     * @dev calculates the compounded borrow balance of a user
     * @param _self the userReserve object
     * @param _reserve the reserve object
     * @return the user compounded borrow balance
     **/
    pub fn get_compounded_borrow_balance(&self, _reserve: &ReserveData, env: &Env) -> U256 {
        if self.principal_borrow_balance.0 == U256::from(0) {
            return U256::from(0);
        }

        let principal_borrow_balance_ray = self.principal_borrow_balance.0.wad_to_ray();
        let mut compounded_balance = U256::from(0);
        let mut cumulated_interest = U256::from(0);

        if self.stable_borrow_rate > Uint256::from(0_u64) {
            cumulated_interest = calculate_compouned_interest(
                self.stable_borrow_rate,
                self.last_update_timestamp,
                env,
            );
        } else {
            //variable interest
            cumulated_interest = calculate_compouned_interest(
                _reserve.current_variable_borrow_rate,
                _reserve.last_update_timestamp,
                env,
            )
            .ray_mul(_reserve.last_variable_borrow_cumulative_index.0)
            .ray_div(self.last_variable_borrow_cumulative_index.0);
        }

        compounded_balance = principal_borrow_balance_ray
            .ray_mul(cumulated_interest)
            .ray_to_wad();
        if compounded_balance == self.principal_borrow_balance.0 {
            //solium-disable-next-line
            if self.last_update_timestamp != env.block.time {
                //no interest cumulation because of the rounding - we add 1 wei
                //as symbolic cumulated interest to avoid interest free loans.

                return self.principal_borrow_balance.0.add(U256::from(1));
            }
        }
        compounded_balance
    }
}
/**
 * @dev Updates the liquidity cumulative index Ci and variable borrow cumulative index Bvc. Refer to the whitepaper for
 * a formal specification.
 * @param _self the reserve object
**/

//pub fn

pub fn calculate_linear_interest(_rate: U256, _last_update_timestamp: u64, env: &Env) -> U256 {
    //solim-disable-next-line
    let time_defference = U256::from(env.block.time.sub(_last_update_timestamp));

    let time_delta = time_defference
        .wad_to_ray()
        .ray_div(SECODNS_PER_YEAR.wad_to_ray());

    _rate.ray_mul(time_delta).add(wad_ray_math::ray_256())
}

pub fn calculate_compouned_interest(
    _rate: Uint256,
    _last_update_timestamp: u64,
    env: &Env,
) -> U256 {
    let time_defference = U256::from(env.block.time.sub(_last_update_timestamp));

    let rate_pre_second = _rate.0.div(*SECODNS_PER_YEAR);

    rate_pre_second
        .add(wad_ray_math::ray_256())
        .ray_pow(time_defference)
}
/**
 * @dev returns the total borrows on the reserve
 * @param _reserve the reserve object
 * @return the total borrows (stable + variable)
 **/

fn _get_total_borrows(_reserve: &ReserveData) -> U256 {
    _reserve
        .total_borrow_stable
        .0
        .add(_reserve.total_borrows_variable.0)
}

#[cfg(test)]
mod core_lib_tests {
    use cosmwasm_std::{
        testing::MOCK_CONTRACT_ADDR, BlockInfo, ContractInfo, HumanAddr, MessageInfo,
    };

    use super::*;

    #[test]
    fn inti_test() {
        let mut data = ReserveData::default();
        data.init(
            "deafult_address".to_string(),
            U256::from(1000).into(),
            "test_rate_strategy_address".to_string(),
        )
        .unwrap();
    }

    #[test]
    fn double_init_test() {
        let mut data = ReserveData::default();
        data.init(
            "a_token_address".to_string(),
            U256::from(10).into(),
            "interested_rate_strategy_address".to_string(),
        )
        .unwrap();
        let res = data.init(
            "a_token_address".to_string(),
            U256::from(10).into(),
            "interested_rate_strategy_address".to_string(),
        );
        let want = Err(StdError::generic_err(
            "Reserve has already been initialized",
        ));

        assert_eq!(want, res);
    }

    #[test]
    fn get_normolize_test() {
        let reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(2).pow(U256::from(1)).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(10).pow(U256::from(27)).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_520,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        let normolze = reserve_data.get_normolized_income(&Env {
            block: BlockInfo {
                height: 100,
                time: 1_571_800_620,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: HumanAddr("bob".to_string()),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        });

        assert_eq!(50, normolze.as_u32());
    }

    #[test]
    fn calculate_compouned_interest_test() {
        let env = Env {
            block: BlockInfo {
                height: 100,
                time: 1_571_800_420,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: HumanAddr("bob".to_string()),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        };

        let res = calculate_compouned_interest(Uint256::from(10_u64), 1_571_800_320, &env);
        assert_eq!(U256::from(10).pow(U256::from(27)), res);
    }

    #[test]
    fn calculate_linear_interest_test() {
        let env = Env {
            block: BlockInfo {
                height: 100,
                time: 1_571_800_420,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: HumanAddr("bob".to_string()),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        };
        let res = calculate_linear_interest(U256::from(10), 1_571_800_320, &env);
        assert_eq!(U256::from(10).pow(U256::from(27)), res);
    }

    #[test]
    fn get_compounded_borrow_balance_test() {
        let user_data = UserReserveData {
            principal_borrow_balance: Uint256::from(10000u64), //U256::from(10000),
            last_variable_borrow_cumulative_index: Uint256::from(10_u64),
            origination_fee: Uint256::from(5_u64),
            stable_borrow_rate: Uint256::from(50_u64),
            last_update_timestamp: 1_571_800_320,
            use_as_collateral: true,
        };
        let reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: true,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(7).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };
        let env = Env {
            block: BlockInfo {
                height: 100,
                time: 1_571_800_420,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: HumanAddr("bob".to_string()),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        };
        let res = user_data.get_compounded_borrow_balance(&reserve_data, &env);
        assert_eq!(U256::from(10001), res);
    }

    #[test]
    fn update_cumulative_indexes_test() {
        let env = Env {
            block: BlockInfo {
                height: 100,
                time: 1_571_800_420,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: HumanAddr("bob".to_string()),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        };

        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: true,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data.update_cumulative_indexes(&env);

        assert_eq!(
            Uint256::from(70u64),
            reserve_data.last_variable_borrow_cumulative_index
        );
    }

    #[test]
    fn get_total_borrows_test() {
        let reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: true,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };
        let res = reserve_data.get_total_borrows();
        assert_eq!(170, res.as_u64());
    }

    #[test]
    fn enable_borrowing_test_1() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: true,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        let res = reserve_data.enable_borrowing(true);
        let want = Err(StdError::generic_err("Reserve is already enabled"));
        assert_eq!(want, res);
    }

    #[test]
    fn enable_borrowing_test_2() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data.enable_borrowing(true).unwrap();
        assert_eq!(true, reserve_data.borrowing_enabled);
    }

    #[test]
    fn disable_borrowing_test() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: true,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data.disable_borrowing();
        assert_eq!(false, reserve_data.borrowing_enabled);
    }

    #[test]
    fn enable_as_cllateral_test_1() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: true,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        let res = reserve_data.enable_as_collateral(
            U256::from(77).into(),
            U256::from(14).into(),
            U256::from(80).into(),
        );
        let want = Err(StdError::generic_err(
            "Reserve is already enabled as collateral",
        ));
        assert_eq!(want, res);
    }

    #[test]
    fn enable_as_cllateral_test_2() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data
            .enable_as_collateral(
                U256::from(77).into(),
                U256::from(14).into(),
                U256::from(80).into(),
            )
            .unwrap();
        let want = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(77).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(14).into(),
            liquidation_bonus: U256::from(80).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        assert_eq!(want, reserve_data);
    }

    #[test]
    fn disable_as_collateral_test() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(70).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };
        reserve_data.disable_as_collateral();
        assert_eq!(false, reserve_data.usage_as_collateral_enabled);
    }

    #[test]
    fn increase_total_borrows_stable_and_update_averege_rate_test() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(0).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };
        let ten = U256::from(10);
        let amount = ten.pow(U256::from(27)).into();
        reserve_data.increase_total_borrows_stable_and_update_averege_rate(
            amount,
            U256::from(80000_u64).into(),
        );
        let want: Uint256 = U256::from(160000).into();
        assert_eq!(amount, reserve_data.total_borrow_stable);
        assert_eq!(want, reserve_data.current_average_stable_borrow_rate);
    }

    #[test]
    fn descrees_total_borrows_stable_and_update_average_rate_test_1() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(20).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        let want = Err(StdError::generic_err("Invalid amount to decrease"));
        let res = reserve_data.decrease_total_borrows_stable_and_update_average_rate(
            U256::from(40).into(),
            U256::from(70).into(),
        );

        assert_eq!(want, res);
    }

    #[test]
    fn descrees_total_borrows_stable_and_update_average_rate_test_2() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(140).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data
            .decrease_total_borrows_stable_and_update_average_rate(
                U256::from(70).into(),
                U256::from(70).into(),
            )
            .unwrap();
        assert_eq!(Uint256::from(70u64), reserve_data.total_borrow_stable);
        assert_eq!(
            Uint256::zero(),
            reserve_data.current_average_stable_borrow_rate
        );
    }

    #[test]
    fn descrees_total_borrows_stable_and_update_average_rate_test_3() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(100).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(140).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data
            .decrease_total_borrows_stable_and_update_average_rate(
                U256::from(140).into(),
                U256::from(70).into(),
            )
            .unwrap();
        assert_eq!(Uint256::zero(), reserve_data.total_borrow_stable);
        assert_eq!(
            Uint256::zero(),
            reserve_data.current_average_stable_borrow_rate
        );
    }

    #[test]
    fn descrees_total_borrows_stable_and_update_average_rate_test_4() {
        //let total_borrow_stabl
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(2).pow(U256::from(1)).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(10).pow(U256::from(27)).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        let res = reserve_data.decrease_total_borrows_stable_and_update_average_rate(
            U256::from(10).pow(U256::from(26)).into(),
            U256::from(70).into(),
        );
        let want = Err(StdError::generic_err("The amounts to subtract don't match"));
        assert_eq!(want, res);
    }

    #[test]
    fn increase_total_borrows_variable_test() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(2).pow(U256::from(1)).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(10).pow(U256::from(27)).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data.increase_total_borrows_variable(U256::from(100).into());

        assert_eq!(Uint256::from(200_u64), reserve_data.total_borrows_variable);
    }

    #[test]
    fn decrease_total_borrows_varible_test_1() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(2).pow(U256::from(1)).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(10).pow(U256::from(27)).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };
        let res = reserve_data.decrease_total_borrows_varible(U256::from(200).into());
        let want = Err(StdError::generic_err(
            "The amount that is being subtracted from the variable total borrows is incorrect",
        ));
        assert_eq!(want, res);
    }

    #[test]
    fn decrease_total_borrows_varible_test_2() {
        let mut reserve_data = ReserveData {
            a_token_address: "tokena_addrr".to_string(),
            base_ltv_as_collateral: U256::from(50).into(),
            borrowing_enabled: false,
            current_average_stable_borrow_rate: U256::from(2).pow(U256::from(1)).into(),
            current_liquidity_rate: U256::from(400).into(),
            last_liquidity_cumulate_index: U256::from(50).into(),
            total_borrow_stable: U256::from(10).pow(U256::from(27)).into(),
            total_borrows_variable: U256::from(100).into(),
            current_variable_borrow_rate: Uint256::from(70_u64),
            current_stable_borrow_rate: U256::from(78).into(),
            last_variable_borrow_cumulative_index: U256::from(70).into(),
            liquidation_threshold: U256::from(7).into(),
            liquidation_bonus: U256::from(45).into(),
            decimals: U256::from(10).into(),
            interest_rate_strategy_address: "test_rate_strategy_addr".to_string(),
            last_update_timestamp: 1_571_800_320,
            usage_as_collateral_enabled: true,
            is_stable_borrow_rate_enabled: true,
            is_active: true,
            is_freezed: false,
        };

        reserve_data
            .decrease_total_borrows_stable_and_update_average_rate(
                U256::from(50).into(),
                U256::from(50).into(),
            )
            .unwrap();
        let want: Uint256 = Uint256::from(10_u64).0.pow(U256::from(27)).into();
        assert_eq!(
            want - Uint256::from(50_u64),
            reserve_data.total_borrow_stable
        );
        assert_eq!(
            Uint256::from(2u64),
            reserve_data.current_average_stable_borrow_rate
        );
    }
}
