use fadroma::{
    schemars,
    cosmwasm_std::StdResult,
    Uint256, Decimal256,
};
use serde::{Serialize, Deserialize};

pub const BLOCKS_PER_YEAR: u64 = 5259600;

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
pub struct JumpRateInterest {
    /// The multiplier of utilization rate that gives the slope of the interest rate.
    pub multiplier_block: Decimal256,
    /// The multiplier_block after hitting a specified utilization point.
    pub jump_multiplier_block: Decimal256,
    /// The base interest rate which is the y-intercept when utilization rate is 0. 
    pub base_rate_block: Decimal256,
    /// The utilization point at which the jump multiplier is applied.
    pub jump_threshold: Decimal256
}

impl JumpRateInterest {
    pub fn v1(
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>
    ) -> StdResult<Self> {
        let blocks_year = blocks_year.unwrap_or(BLOCKS_PER_YEAR);
        let blocks = Decimal256::from_uint256(Uint256::from(blocks_year))?;

        Ok(Self {
            base_rate_block: (base_rate_year / blocks)?,
            multiplier_block: (multiplier_year / blocks)?,
            jump_multiplier_block: (jump_multiplier_year / blocks)?,
            jump_threshold
        })
    }

    pub fn v2(
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>
    ) -> StdResult<Self> {
        let blocks_year = blocks_year.unwrap_or(BLOCKS_PER_YEAR);
        let blocks = Decimal256::from_uint256(Uint256::from(blocks_year))?;

        Ok(Self {
            base_rate_block: (base_rate_year / blocks)?,
            multiplier_block: ((multiplier_year * Decimal256::one())? / (blocks * jump_threshold)?)?,
            jump_multiplier_block: (jump_multiplier_year / blocks)?,
            jump_threshold
        })
    }

    pub fn borrow_rate(
        &self,
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256
    ) -> StdResult<Decimal256> {
        let util_rate = utilization_rate(market_size, num_borrows, reserves)?;

        if util_rate <= self.jump_threshold {
            return ((util_rate * self.multiplier_block)? / Decimal256::one())? + self.base_rate_block;
        }

        let normal_rate = (((self.jump_threshold * self.multiplier_block)? / Decimal256::one())? + self.base_rate_block)?;
        let excess_rate = (util_rate - self.jump_threshold)?;

        ((excess_rate * self.jump_multiplier_block)? / Decimal256::one())? + normal_rate
    }

    pub fn supply_rate(
        &self,
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
        reserve_factor: Decimal256
    ) -> StdResult<Decimal256> {
        let one_minus_reserve_factor = (Decimal256::one() - reserve_factor)?;
        let borrow_rate = self.borrow_rate(market_size, num_borrows, reserves)?;
        let rate_to_pool = ((borrow_rate * one_minus_reserve_factor)? / Decimal256::one())?;

        (utilization_rate(market_size, num_borrows, reserves)? * rate_to_pool)? / Decimal256::one()
    }
}

pub fn utilization_rate(
    market_size: Decimal256,
    num_borrows: Decimal256,
    reserves: Decimal256
) -> StdResult<Decimal256> {
    if num_borrows.is_zero() {
        return Ok(num_borrows);
    }

    (num_borrows * Decimal256::one())? / ((market_size + num_borrows)? - reserves)?
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[derive(Default)]
    struct Utilization {
        pub market_size: Decimal256,
        pub num_borrows: Decimal256,
        pub reserves: Decimal256
    }

    impl Utilization {
        fn new(util: Decimal256) -> Self {
            if util.is_zero() {
                return Self::default();
            }

            Self {
                market_size: Decimal256((Decimal256::one().0 * Decimal256::one().0) / util.0),
                num_borrows: Decimal256::one(),
                reserves: Decimal256::one()
            }
        }
    }

    #[test]
    fn borrow_rate_v1() {
        let blocks_year_raw = 2102400;
        let blocks_year = Decimal256::from_uint256(Uint256::from(blocks_year_raw)).unwrap();

        let tests = [
            (
                JumpRateInterest::v1(
                    Decimal256::from_str("0.1").unwrap(),
                    Decimal256::from_str("0.2").unwrap(),
                    Decimal256::one(),
                    Decimal256::from_str("0.9").unwrap(),
                    Some(blocks_year_raw)
                ).unwrap(),
                vec![
                    (Decimal256::zero(), Decimal256::from_str("0.1").unwrap()),
                    (Decimal256::from_str("0.1").unwrap(), Decimal256::from_str("0.12").unwrap()),
                    (Decimal256::from_str("0.89").unwrap(), Decimal256::from_str("0.278").unwrap()),
                    (Decimal256::from_str("0.9").unwrap(), Decimal256::from_str("0.28").unwrap()),
                    (Decimal256::from_str("0.91").unwrap(), Decimal256::from_str("0.29").unwrap()),
                    (Decimal256::one(), Decimal256::from_str("0.38").unwrap())
                ]
            ),
            (
                JumpRateInterest::v1(
                    Decimal256::from_str("0.1").unwrap(),
                    Decimal256::from_str("0.2").unwrap(),
                    Decimal256::from_str("0.2").unwrap(),
                    Decimal256::from_str("0.9").unwrap(),
                    Some(blocks_year_raw)
                ).unwrap(),
                vec![
                    (Decimal256::zero(), Decimal256::from_str("0.1").unwrap()),
                    (Decimal256::from_str("0.1").unwrap(), Decimal256::from_str("0.12").unwrap()),
                    (Decimal256::one(), Decimal256::from_str("0.30").unwrap())
                ]
            ),
            (
                JumpRateInterest::v1(
                    Decimal256::from_str("0.1").unwrap(),
                    Decimal256::from_str("0.2").unwrap(),
                    Decimal256::zero(),
                    Decimal256::from_str("0.9").unwrap(),
                    Some(blocks_year_raw)
                ).unwrap(),
                vec![
                    (Decimal256::zero(), Decimal256::from_str("0.1").unwrap()),
                    (Decimal256::from_str("0.1").unwrap(), Decimal256::from_str("0.12").unwrap()),
                    (Decimal256::one(), Decimal256::from_str("0.28").unwrap())
                ]
            ),
            (
                JumpRateInterest::v1(
                    Decimal256::from_str("0.1").unwrap(),
                    Decimal256::from_str("0.2").unwrap(),
                    Decimal256::zero(),
                    Decimal256::from_str("1.1").unwrap(),
                    Some(blocks_year_raw)
                ).unwrap(),
                vec![
                    (Decimal256::zero(), Decimal256::from_str("0.1").unwrap()),
                    (Decimal256::from_str("0.1").unwrap(), Decimal256::from_str("0.12").unwrap()),
                    (Decimal256::one(), Decimal256::from_str("0.30").unwrap())
                ]
            ),
            (
                JumpRateInterest::v1(
                    Decimal256::from_str("0.1").unwrap(),
                    Decimal256::from_str("0.2").unwrap(),
                    Decimal256::from_str("20").unwrap(),
                    Decimal256::zero(),
                    Some(blocks_year_raw)
                ).unwrap(),
                vec![
                    (Decimal256::zero(), Decimal256::from_str("0.1").unwrap()),
                    (Decimal256::from_str("0.1").unwrap(), Decimal256::from_str("2.1").unwrap()),
                    (Decimal256::one(), Decimal256::from_str("20.1").unwrap())
                ]
            )
        ];

        let mut index = 0;

        for test in tests {
            for (util, expected) in test.1 {
                println!("Test case: {}, util: {}, expected: {}", index, util, expected);

                let vars = Utilization::new(util);

                let result = test.0.borrow_rate(vars.market_size, vars.num_borrows, vars.reserves).unwrap();
                let result = ((result / Decimal256::one()).unwrap() * blocks_year).unwrap();
        
                assert_delta(result, expected);
            }

            index += 1;
        }
    }

    fn assert_delta(lhs: Decimal256, rhs: Decimal256) {
        let max = lhs.max(rhs);
        let min = lhs.min(rhs);

        let ok = (max - min).unwrap() < Decimal256::from_str("0.01").unwrap();

        assert!(ok, "lhs: {}, rhs: {}", lhs.0, rhs.0);
    }
}
