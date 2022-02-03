use lend_shared::{
    fadroma::{
        ensemble::MockEnv,
        permit::Permit,
        cosmwasm_std::{Uint128, HumanAddr, StdError, Binary, to_binary},
        snip20_impl::msg as snip20,
        Decimal256, Uint256
    },
    interfaces::{
        market, overseer
    },
    core::utilization_rate
};
use crate::setup::Lend;

const BOB: &str = "Bob";
const ALICE: &str = "Alice";
const CHESTER: &str = "Chester";

impl Lend {
    #[inline]
    pub fn _underlying_balance(
        &self,
        address: impl Into<HumanAddr>,
        market: HumanAddr
    ) -> Uint256 {
        self.ensemble.query(
            market.clone(),
            market::QueryMsg::BalanceUnderlying {
                method: Permit::new(
                    address,
                    vec![ market::MarketPermissions::Balance ],
                    vec![ market ],
                    "balance"
                ).into(),
                block: None
            }
        ).unwrap()
    }

    #[inline]
    pub fn id(
        &self,
        address: impl Into<HumanAddr>,
        market: HumanAddr
    ) -> Binary {
        self.ensemble.query(
            market.clone(),
            market::QueryMsg::Id {
                method: Permit::new(
                    address,
                    vec![ market::MarketPermissions::Id ],
                    vec![ market ],
                    "id"
                ).into()
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
    )
    .unwrap()
    .contract;

    let market_two = lend.whitelist_market(
        underlying_2,
        Decimal256::percent(90),
        None
    )
    .unwrap()
    .contract;

    let prefund_amount = Uint128(500);
    let borrow_amount = Uint128(100);

    lend.prefund_and_deposit(BOB, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(ALICE, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(CHESTER, borrow_amount, market_two.address.clone());

    lend.ensemble.execute(
        &overseer::HandleMsg::Enter {
            markets: vec![
                market_one.address.clone(),
                market_two.address.clone()
            ]
        },
        MockEnv::new(CHESTER, lend.overseer.clone())
    ).unwrap();

    // Assuming 1:1 price conversion:
    // With LTV ratio of 90%, if Chester wants to borrow a 100 tokens, he'd have to deposit
    // 10% more as collateral - 100 + 10% = 110

    let liquidity = lend.get_liquidity(
        CHESTER,
        Some(market_one.address.clone()),
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
        MockEnv::new(CHESTER, market_one.clone())
    ).unwrap_err();

    assert_eq!(err, StdError::generic_err("Insufficient liquidity. Shortfall: 10"));

    // 12 instead of 10 because of rounding during exchange rate divison
    lend.prefund_and_deposit(CHESTER, Uint128(12), market_two.address.clone());

    lend.ensemble.execute(
        &market::HandleMsg::Borrow {
            amount: borrow_amount.into()
        },
        MockEnv::new(CHESTER, market_one.clone())
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
        market_one.address.clone(),
        market::QueryMsg::State { block: None }
    ).unwrap();

    assert_eq!(state.total_borrows, 100.into());
    assert_eq!(state.total_supply, 750.into());

    let utilization_rate = utilization_rate(
        Decimal256::from_uint256(state.underlying_balance).unwrap(),
        Decimal256::from_uint256(state.total_borrows).unwrap(),
        Decimal256::from_uint256(state.total_reserves).unwrap()
    ).unwrap();

    assert_eq!(utilization_rate, Decimal256::percent(10));

    let info: Vec<market::Borrower> = lend.ensemble.query(
        market_one.address.clone(),
        market::QueryMsg::Borrowers {
            limit: None,
            start_after: None,
            block: state.accrual_block
        }
    ).unwrap();

    assert_eq!(info.len(), 1);

    let chester = info.first().unwrap();

    assert_eq!(chester.liquidity.liquidity, Uint256::zero());
    assert_eq!(chester.liquidity.shortfall, Uint256::zero());

    assert_eq!(chester.markets.len(), 2);
    assert_eq!(chester.markets[0].contract, market_one);
    assert_eq!(chester.markets[1].contract, market_two);

    assert_eq!(chester.info.principal, borrow_amount.into());
    assert_eq!(chester.info.interest_index, state.borrow_index);

    let id = lend.id(CHESTER, market_one.address);
    assert_eq!(chester.id, id);
}

#[test]
fn self_repay() {
    do_repay(true)
}

#[test]
fn repay() {
    do_repay(false)
}

fn do_repay(self_repay: bool) {
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("SSCRT", 6).unwrap();
    let underlying_2 = lend.new_underlying_token("SIENNA", 18).unwrap();

    let market_one = lend.whitelist_market(
        underlying_1.clone(),
        Decimal256::percent(90),
        None
    )
    .unwrap()
    .contract;

    let market_two = lend.whitelist_market(
        underlying_2,
        Decimal256::percent(90),
        None
    )
    .unwrap()
    .contract;

    let prefund_amount = Uint128(500);
    let borrow_amount = Uint128(100);

    lend.prefund_and_deposit(BOB, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(ALICE, prefund_amount, market_one.address.clone());
    lend.prefund_and_deposit(CHESTER, Uint128(112), market_two.address.clone());

    lend.ensemble.execute(
        &overseer::HandleMsg::Enter {
            markets: vec![
                market_one.address.clone(),
                market_two.address.clone()
            ]
        },
        MockEnv::new(CHESTER, lend.overseer.clone())
    ).unwrap();

    lend.ensemble.execute(
        &market::HandleMsg::Borrow {
            amount: borrow_amount.into()
        },
        MockEnv::new(CHESTER, market_one.clone())
    ).unwrap();

    let (repayer, id) = if self_repay {
        (CHESTER, None)
    } else {
        (ALICE, Some(lend.id(CHESTER, market_one.address.clone())))
    };

    lend.prefund_user(repayer, borrow_amount, underlying_1.clone());

    lend.ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: market_one.address.clone(),
            recipient_code_hash: None,
            amount: borrow_amount,
            msg: Some(to_binary(&market::ReceiverCallbackMsg::Repay {
                borrower: id
            }).unwrap()),
            memo: None,
            padding: None
        },
        MockEnv::new(repayer, underlying_1)
    ).unwrap();

    let state: market::State = lend.ensemble.query(
        market_one.address.clone(),
        market::QueryMsg::State { block: None }
    ).unwrap();

    assert_eq!(state.total_borrows, Uint256::zero());
    assert_eq!(state.total_supply, 750.into());

    let info: Vec<market::Borrower> = lend.ensemble.query(
        market_one.address.clone(),
        market::QueryMsg::Borrowers {
            limit: None,
            start_after: None,
            block: state.accrual_block
        }
    ).unwrap();

    // Borrows repaid in full are removed from storage.
    assert_eq!(info.len(), 0);
}
