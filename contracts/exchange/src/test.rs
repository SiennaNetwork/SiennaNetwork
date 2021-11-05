use amm_shared::{
    fadroma::{
        scrt_link::{ContractLink, ContractInstantiationInfo},
        scrt_callback::Callback,
        scrt::{
            testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
            Extern, HumanAddr, StdError, to_binary
        },
    },
    msg::exchange::{InitMsg, HandleMsg},
    TokenPair, TokenType,
};

use crate::{contract::*, state::load_config};

fn do_init() -> Extern<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies(20, &[]);

    init(&mut deps, mock_env("factory_addr", &[]), InitMsg {
        pair: TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("addr_1".into()),
                token_code_hash: "code_hash_1".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("addr_2".into()),
                token_code_hash: "code_hash_2".into(),
            }
        ),
        lp_token_contract: ContractInstantiationInfo {
            id: 1,
            code_hash: "lp_token_hash".into()
        },
        factory_info: ContractLink {
            address: "factory_addr".into(),
            code_hash: "factory_hash".into()
        },
        callback: Callback {
            msg: to_binary(b"whatever").unwrap(),
            contract: ContractLink {
                address: "factory_addr".into(),
                code_hash: "factory_hash".into()
            }
        },
        prng_seed: to_binary(b"whatever").unwrap(),
        entropy: to_binary(b"whatever").unwrap()
    }).unwrap();

    deps
}

#[test]
fn test_change_factory() {
    let ref mut deps = do_init();

    let new_instance = ContractLink {
        address: "new_factory_addr".into(),
        code_hash: "new_factory_hash".into()
    };

    let err = handle(deps, mock_env("sender", &[]), HandleMsg::ChangeFactory {
        contract: new_instance.clone()
    }).unwrap_err();

    assert_eq!(err, StdError::unauthorized());

    handle(deps, mock_env("factory_addr", &[]), HandleMsg::ChangeFactory {
        contract: new_instance.clone()
    }).unwrap();

    let config = load_config(deps).unwrap();
    assert_eq!(config.factory_info, new_instance);

    let err = handle(deps, mock_env("factory_addr", &[]), HandleMsg::ChangeFactory {
        contract: new_instance.clone()
    }).unwrap_err();

    assert_eq!(err, StdError::unauthorized());
}