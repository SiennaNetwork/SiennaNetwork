use amm_shared::{
    fadroma::{
        ContractLink,
        Callback,
        cosmwasm_std::{
            StdResult, Env, Binary, InitResponse,
            HandleResponse, HumanAddr, Uint128,
            from_binary
        },
        ensemble::{
            ContractEnsemble, MockEnv,
            ContractHarness, MockDeps, 
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
    TokenPair, TokenType, Pagination,
    ExchangeSettings, Fee, Exchange,
    msg
};


pub struct Ido;
pub struct Launchpad;
use ido::contract as ido;
use launchpad::contract as launchpad;

pub const ADMIN: &str = "admin";
const BLOCK_TIME: u64 = 1_571_797_419;
const RATE: Uint128 = Uint128(1_u128);
const MIN_ALLOCATION: Uint128 = Uint128(100_u128);
const MAX_ALLOCATION: Uint128 = Uint128(500_u128);


pub struct Token;



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
    pub ensemble: ContractEnsemble
}

impl LaunchpadIdo {
    pub fn new() -> Self {
       
        let mut ensemble = ContractEnsemble::new(200);
    
        let ido = ensemble.register(Box::new(Ido));
        let snip20 = ensemble.register(Box::new(Token));
        let launchpad = ensemble.register(Box::new(Launchpad));

        let ido = ensemble.instantiate(
            ido.id,
            &msg::ido::InitMsg {

                admin: HumanAddr::from("sold-token"),
                prng_seed: Binary::from(b"whatever"),
                entropy: Binary::from(b"whatever"),
                launchpad: None,
                info: amm_shared::msg::ido::TokenSaleConfig {
                    input_token: TokenType::NativeToken {
                        denom: "uscrt".to_string(),
                    },
                    rate: RATE,
                    sold_token: ContractLink {
                        address: HumanAddr::from("sold-token"),
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
                },
                callback: Callback {
                    msg: Binary::from(&[]),
                    contract: ContractLink {
                        address: HumanAddr::from("callback-address"),
                        code_hash: "code-hash-of-callback-contract".to_string(),
                    },
                },
            },
            MockEnv::new(ADMIN, ContractLink {
                address: "ido".into(),
                code_hash: ido.code_hash.clone()
            })
        ).unwrap();
       
        let launchpad = ensemble.instantiate(
            launchpad.id,
            &msg::launchpad::InitMsg {

                admin: HumanAddr::from("sold-token"),
                prng_seed: Binary::from(b"whatever"),
                entropy: Binary::from(b"whatever"),
                tokens: vec![],
                callback: Callback {
                    msg: Binary::from(&[]),
                    contract: ContractLink {
                        address: HumanAddr::from("callback-address"),
                        code_hash: "code-hash-of-callback-contract".to_string(),
                    },
                },
            },
            MockEnv::new(ADMIN, ContractLink {
                address: "launchpad".into(),
                code_hash: launchpad.code_hash.clone()
            })
        ).unwrap();
       

        LaunchpadIdo {
            ensemble
        }
    }
}
