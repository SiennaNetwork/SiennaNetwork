use super::*;


use amm_shared::{
    fadroma::{
        ContractLink,
        cosmwasm_std::{
            StdResult,InitResponse,
            HandleResponse,
        },
        ensemble::{
            ContractEnsemble, MockEnv,
            ContractHarness, MockDeps, 
        },
        {
        platform::{
            from_binary, Binary, Coin, Env, Extern, HumanAddr, Uint128, ContractInstantiationInfo, Callback,
            testing::{mock_env, MockApi, MockStorage},
        }    
        },
        auth::Permit,
        snip20_impl::{
            msg::{
                InitMsg as Snip20InitMsg,
                QueryMsg as Snip20QueryMsg,
                QueryPermission as Snip20Permission,
                QueryWithPermit,
                QueryAnswer,
                InitConfig,
                InitialBalance
            },
            snip20_init, snip20_handle, snip20_query, SymbolValidation, Snip20
        },
    },
    msg::ido::{
        HandleMsg, InitMsg, QueryMsg, QueryResponse, ReceiverCallbackMsg, TokenSaleConfig,
    },
    querier::{MockContractInstance, MockQuerier},
    TokenPair, TokenType, Pagination,
    ExchangeSettings, Fee, Exchange,
    msg
};


pub struct Ido;
pub struct Launchpad;
pub struct Token;
use ido::contract as ido;
use launchpad::contract as launchpad;


pub const ADMIN: &str = "admin";
const BLOCK_TIME: u64 = 1_571_797_419;
const RATE: Uint128 = Uint128(1_u128);
const MIN_ALLOCATION: Uint128 = Uint128(100_u128);
const MAX_ALLOCATION: Uint128 = Uint128(500_u128);

pub const USERS: &[&str] = &[ "user_a", "user_b", "user_c" ];
pub const BURNER: &str = "burner_acc";
pub const INITIAL_BALANCE: Uint128 = Uint128(1000_000_000_000_000_000_000);
pub const NATIVE_DENOM: &str = "uscrt";


fn internal_mock_deps(
    len: usize,
    balance: &[Coin],
) -> Extern<MockStorage, MockApi, MockQuerier> {
    let contract_addr = HumanAddr::from("mock-address");
    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(len),
        querier: MockQuerier::new(
            &[(&contract_addr, balance)],
            vec![MockContractInstance {
                instance: ContractLink {
                    address: HumanAddr::from("sold-token"),
                    code_hash: "".to_string(),
                },
                token_decimals: 18,
                token_supply: Uint128::from(2500_u128),
            },
            MockContractInstance {
                instance: ContractLink {
                    address: HumanAddr::from("callback-address"),
                    code_hash: "".to_string(),
                },
                token_decimals: 18,
                token_supply: Uint128::from(2500_u128),
            }],
        ),
    }
}


impl Snip20 for Token {
    fn symbol_validation(&self) -> SymbolValidation {
        SymbolValidation {
            length: 3..=6,
            allow_upper: true,
            allow_lower: true,
            allow_numeric: true,
            allowed_special: None,
        }
    }
}

impl ContractHarness for Token {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        snip20_init(deps, env, from_binary(&msg)?, Self)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        snip20_handle(deps, env, from_binary(&msg)?, Self)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        snip20_query(deps, from_binary(&msg)?, Self)
    }
}

impl ContractHarness for Ido {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        ido::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        ido::handle(deps, env, from_binary(&msg)?)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        ido::query(deps, from_binary(&msg)?)
    }
}

impl ContractHarness for Launchpad {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        launchpad::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        launchpad::handle(deps, env, from_binary(&msg)?)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        launchpad::query(deps, from_binary(&msg)?)
    }
}

pub struct LaunchpadIdo {
    pub ensemble: ContractEnsemble,
    pub ido: ContractLink<HumanAddr>,
    pub launchpad: ContractLink<HumanAddr>
}

impl LaunchpadIdo {
    pub fn new() -> Self {
       
        let mut ensemble = ContractEnsemble::new(200);
    
        let ido = ensemble.register(Box::new(Ido));
        let launchpad = ensemble.register(Box::new(Launchpad));
        let token = ensemble.register(Box::new(Token));

        let token = ensemble.instantiate(
            token.id,
            &Snip20InitMsg {
                name: "sold-token".to_string(),
                admin: None,
                symbol: format!("TKN{}", 1),
                decimals: 18,
                initial_balances: Some(USERS
                    .iter()
                    .map(|x| InitialBalance {
                        address: (*x).into(),
                        amount: INITIAL_BALANCE
                    })
                    .collect()
                ),
                initial_allowances: None,
                prng_seed: Binary::from(b"whatever"),
                config: Some(InitConfig::builder()
                    .public_total_supply()
                    .enable_mint()
                    .build()
                ),
                callback: None
            },
            MockEnv::new(ADMIN, ContractLink {
                address: "sold_token".into(),
                code_hash: token.code_hash.clone()
            })
        ).unwrap();

        let launchpad = ensemble.instantiate(
            launchpad.id,
            &msg::launchpad::InitMsg {

                admin: HumanAddr::from("admin"),
                prng_seed: Binary::from(b"whatever"),
                entropy: Binary::from(b"whatever"),
                tokens: vec![]
            },
            MockEnv::new(ADMIN, ContractLink {
                address: "launchpad".into(),
                code_hash: launchpad.code_hash.clone()
            })
        ).unwrap();
        
        let ido = ensemble.instantiate(
            ido.id,
            &msg::ido::InitMsg {

                admin: HumanAddr::from("admin"),
                prng_seed: Binary::from(b"whatever"),
                entropy: Binary::from(b"whatever"),
                launchpad: None,
                info: amm_shared::msg::ido::TokenSaleConfig {
                    input_token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    rate: RATE,
                    sold_token: ContractLink::<HumanAddr> {
                        address: HumanAddr::from("sold_token"),
                        code_hash: "".to_string(),
                    },
                    whitelist: vec![
                        HumanAddr::from("buyer-1"),
                        HumanAddr::from("buyer-2"),
                        HumanAddr::from("buyer-3"),
                        HumanAddr::from("buyer-4"),
                    ],
                    max_seats: 5,
                    max_allocation: MAX_ALLOCATION,
                    min_allocation: MIN_ALLOCATION,
                    sale_type: None,
                }
            },
            MockEnv::new(ADMIN, ContractLink {
                address: "ido".into(),
                code_hash: ido.code_hash.clone()
            })
        ).unwrap();

        LaunchpadIdo {
            ensemble,
            ido,
            launchpad
        }
    }
}
