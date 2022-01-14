use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::{ContractEnsemble, MockEnv},
        secret_toolkit::utils::InitCallback,
        snip20_impl::msg::{
            HandleAnswer, HandleMsg as Snip20HandleMsg, InitMsg as Snip20InitMsg, InitialBalance,
        },
        to_binary, Binary, Callback, ContractLink, Decimal256, HumanAddr, Uint128,
    },
    interfaces::{oracle, overseer::*},
};

use crate::setup::{MockBand, Oracle, Overseer, Token};
use crate::ADMIN;

pub struct InitResult {
    ensemble: ContractEnsemble,
    overseer: ContractLink<HumanAddr>,
}

fn init() -> InitResult {
    let mut ensemble = ContractEnsemble::new(50);

    let overseer = ensemble.register(Box::new(Overseer));
    let oracle = ensemble.register(Box::new(Oracle));
    let mock_band = ensemble.register(Box::new(MockBand));

    let mock_band = ensemble
        .instantiate(
            mock_band.id,
            &{},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "mock_band".into(),
                    code_hash: mock_band.code_hash,
                },
            ),
        )
        .unwrap();

    let overseer = ensemble
        .instantiate(
            overseer.id,
            &InitMsg {
                admin: None,
                prng_seed: Binary::from(b"whatever"),
                close_factor: Decimal256::from_uint256(50000000000000000u128).unwrap(),
                premium: Decimal256::one(),
                oracle_contract: oracle.clone(),
                oracle_source: mock_band
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

    InitResult { ensemble, overseer }
}

#[test]
fn test_init() {
    let _result = init();
}
