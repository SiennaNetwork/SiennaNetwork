use lend_shared::{
    fadroma::{
        ensemble::MockEnv,
        permit::Permit,
        cosmwasm_std::{Uint128, HumanAddr, StdError},
        Decimal256, Uint256
    },
    interfaces::{
        market, overseer,
        market::MarketPermissions
    },
    core::utilization_rate
};
use crate::setup::Lend;

const BOB: &str = "Bob";
const ALICE: &str = "Alice";
const CHESTER: &str = "Chester";

impl Lend {
    #[inline]
    pub fn underlying_balance(
        &self,
        address: impl Into<HumanAddr>,
        market: HumanAddr
    ) -> Uint256 {
        self.ensemble.query(
            market.clone(),
            market::QueryMsg::BalanceUnderlying {
                method: Permit::new(
                    address,
                    vec![ MarketPermissions::Balance ],
                    vec![ market ],
                    "balance"
                ).into(),
                block: None
            }
        ).unwrap()
    }
}

#[test]
fn borrow() {
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("SSCRT", 6).unwrap();
    let underlying_2 = lend.new_underlying_token("SIENNA", 18).unwrap();

    let market_one = lend.whitelist_market(
        underlying_1,
        Decimal256::percent(90),
        None
    ).unwrap();

    let market_two = lend.whitelist_market(
        underlying_2,
        Decimal256::percent(90),
        None
    ).unwrap();

    let prefund_amount = Uint128(500);
    let borrow_amount = Uint128(100);

    lend.prefund_and_deposit(BOB, prefund_amount, market_one.contract.address.clone());
    lend.prefund_and_deposit(ALICE, prefund_amount, market_one.contract.address.clone());
    lend.prefund_and_deposit(CHESTER, borrow_amount, market_two.contract.address.clone());

    lend.ensemble.execute(
        &overseer::HandleMsg::Enter {
            markets: vec![
                market_one.contract.address.clone(),
                market_two.contract.address.clone()
            ]
        },
        MockEnv::new(CHESTER, lend.overseer.clone())
    ).unwrap();

    // Assuming 1:1 price conversion:
    // With LTV ratio of 90%, if Chester wants to borrow a 100 tokens, he'd have to deposit
    // 10% as collateral - 100 + 10% = 110

    let liquidity = lend.get_liquidity(
        CHESTER,
        Some(market_one.contract.address.clone()),
        Uint256::zero(),
        borrow_amount.into(),
        None
    ).unwrap();

    assert_eq!(liquidity.liquidity, Uint256::zero());
    assert_eq!(liquidity.shortfall, Uint256::from(10));
    
    let err = lend.ensemble.execute(
        &market::HandleMsg::Borrow {
            amount: borrow_amount.into()
        },
        MockEnv::new(CHESTER, market_one.contract.clone())
    ).unwrap_err();

    assert_eq!(err, StdError::generic_err("Insufficient liquidity. Shortfall: 10"));

    // 12 instead of 10 because of rounding during exchange rate divison
    lend.prefund_and_deposit(CHESTER, Uint128(12), market_two.contract.address.clone());

    lend.ensemble.execute(
        &market::HandleMsg::Borrow {
            amount: borrow_amount.into()
        },
        MockEnv::new(CHESTER, market_one.contract.clone())
    ).unwrap();

    let liquidity = lend.get_liquidity(
        CHESTER,
        None,
        Uint256::zero(),
        Uint256::zero(),
        None
    ).unwrap();

    assert_eq!(liquidity.liquidity, Uint256::zero());
    assert_eq!(liquidity.shortfall, Uint256::zero());

    let state: market::State = lend.ensemble.query(
        market_one.contract.address,
        market::QueryMsg::State { block: None }
    ).unwrap();

    let utilization_rate = utilization_rate(
        Decimal256::from_uint256(state.underlying_balance).unwrap(),
        Decimal256::from_uint256(state.total_borrows).unwrap(),
        Decimal256::from_uint256(state.total_reserves).unwrap()
    ).unwrap();

    assert_eq!(utilization_rate, Decimal256::percent(10));
}