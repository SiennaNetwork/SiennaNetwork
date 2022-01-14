use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::{ContractEnsemble, MockEnv},
        snip20_impl::msg::{
            HandleAnswer, HandleMsg as Snip20HandleMsg, InitMsg as Snip20InitMsg, InitialBalance,
        },
        to_binary, Binary, Callback, ContractLink, Decimal256, HumanAddr, Uint128,
    },
    interfaces::{oracle, overseer::*},
};

use crate::setup::{Oracle, Overseer, Token};
use crate::ADMIN;

pub struct InitResult {
    ensemble: ContractEnsemble,
    overseer: ContractLink<HumanAddr>,
}

fn init() -> InitResult {
    let mut ensemble = ContractEnsemble::new(50);

    let overseer = ensemble.register(Box::new(Overseer));
    let oracle = ensemble.register(Box::new(Oracle));

    let overseer = ensemble
        .instantiate(
            overseer.id,
            &InitMsg {
                admin: None,
                prng_seed: Binary::from(b"whatever"),
                close_factor: Decimal256::from_uint256(50000000000000000u128).unwrap(),
                premium: Decimal256::one(),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "overseer".into(),
                    code_hash: overseer.code_hash,
                },
            ),
        )
        .unwrap();

    let oracle = ensemble
        .instantiate(
            oracle.id,
            &oracle::InitMsg {
                admin: None,
                source: ContractLink::default(),
                initial_assets: vec![],
                callback: Callback {
                    msg: to_binary(&HandleMsg::RegisterOracle {}).unwrap(),
                    contract: overseer.clone(),
                },
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "oracle".into(),
                    code_hash: oracle.code_hash,
                },
            ),
        )
        .unwrap();

    InitResult { ensemble, overseer }
}

#[test]
fn test_init() {
    let _result = init();
}
