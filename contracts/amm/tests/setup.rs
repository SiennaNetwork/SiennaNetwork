use amm_shared::{
    fadroma::{
        ContractLink, one_token,
        cosmwasm_std::{
            StdResult, Env, Binary, InitResponse,
            HandleResponse, HumanAddr, Uint128,
            from_binary
        },
        ensemble::{
            ContractEnsemble, MockEnv,
            ContractHarness, MockDeps, 
        },
        snip20_impl::{
            msg::{InitMsg as Snip20InitMsg, InitConfig, InitialBalance},
            snip20_init, snip20_handle, snip20_query, SymbolValidation, Snip20
        },
    },
    TokenPair, TokenType, Pagination,
    ExchangeSettings, Fee, Exchange,
    msg
};

use factory::contract as factory;
use exchange::contract as exchange;
use ido::contract as ido;
use launchpad::contract as launchpad;
use router::contract as router;
use lp_token;

pub const ADMIN: &str = "admin";
pub const USER: &str = "user";
pub const BURNER: &str = "burner_acc";

pub struct Amm {
    pub ensemble: ContractEnsemble,
    pub factory: ContractLink<HumanAddr>
}

impl Amm {
    pub fn new() -> Self {
        use std::iter::FromIterator;
    
        let mut ensemble = ContractEnsemble::new(200);
    
        let factory = ensemble.register(Box::new(Factory));
        let snip20 = ensemble.register(Box::new(Token));
        let lp_token = ensemble.register(Box::new(LpToken));
        let pair = ensemble.register(Box::new(Pair));
        let ido = ensemble.register(Box::new(Ido));
        let launchpad = ensemble.register(Box::new(Launchpad));
        let router = ensemble.register(Box::new(Router));
        
        let factory = ensemble.instantiate(
            factory.id,
            &msg::factory::InitMsg {
                snip20_contract: snip20.clone(),
                lp_token_contract: lp_token,
                pair_contract: pair,
                launchpad_contract: launchpad,
                ido_contract: ido,
                router_contract: router,
                exchange_settings: ExchangeSettings {
                    swap_fee: Fee::new(28, 10000),
                    sienna_fee: Fee::new(2, 10000),
                    sienna_burner: Some(HumanAddr::from(BURNER)),
                },
                admin: None,
                prng_seed: Binary::from(b"whatever"),
            },
            MockEnv::new(ADMIN, ContractLink {
                address: "factory".into(),
                code_hash: factory.code_hash.clone()
            })
        ).unwrap();
    
        let mut tokens = Vec::new();
    
        for i in 1..=4 {
            let name = format!("TOKEN_{}", i);
    
            let token = ensemble.instantiate(
                snip20.id,
                &Snip20InitMsg {
                    name: name.clone(),
                    admin: None,
                    symbol: format!("TKN{}", i),
                    decimals: 18,
                    initial_balances: Some(vec![InitialBalance {
                        address: USER.into(),
                        amount: Uint128(1000 * one_token(18))
                    }]),
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
                    address: name.into(),
                    code_hash: snip20.code_hash.clone()
                })
            ).unwrap();
    
            tokens.push(token);
        }
    
        for pair in tokens.chunks(2) {
            let pair = Vec::from_iter(pair);
    
            ensemble.execute(
                &msg::factory::HandleMsg::CreateExchange {
                    pair: TokenPair(
                        TokenType::from(pair[0].clone()),
                        TokenType::from(pair[1].clone()),
                    ),
                    entropy: Binary::from(b"whatever")
                },
                MockEnv::new(ADMIN, factory.clone())
            ).unwrap();
        }
    
        Amm {
            ensemble,
            factory
        }
    }

    pub fn get_pairs(&self) -> Vec<Exchange<HumanAddr>> {
        let response: msg::factory::QueryResponse = self.ensemble.query(
            self.factory.address.clone(),
            &msg::factory::QueryMsg::ListExchanges {
                pagination: Pagination {
                    start: 0,
                    limit: 30
                }
            }
        ).unwrap();

        match response {
            msg::factory::QueryResponse::ListExchanges { exchanges } => {
                exchanges
            },
            _ => panic!("Expected QueryResponse::ListExchanges")
        }
    }
}

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

pub struct Factory;

impl ContractHarness for Factory {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        factory::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        factory::handle(deps, env, from_binary(&msg)?)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        factory::query(deps, from_binary(&msg)?)
    }
}

pub struct Pair;

impl ContractHarness for Pair {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        exchange::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        exchange::handle(deps, env, from_binary(&msg)?)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        exchange::query(deps, from_binary(&msg)?)
    }
}

pub struct LpToken;

impl ContractHarness for LpToken {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        lp_token::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        lp_token::handle(deps, env, from_binary(&msg)?)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        lp_token::query(deps, from_binary(&msg)?)
    }
}

pub struct Ido;

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

pub struct Launchpad;

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

pub struct Router;

impl ContractHarness for Router {
    fn init(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<InitResponse> {
        router::init(deps, env, from_binary(&msg)?)
    }

    fn handle(
        &self,
        deps: &mut MockDeps,
        env: Env,
        msg: Binary
    ) -> StdResult<HandleResponse> {
        router::handle(deps, env, from_binary(&msg)?)
    }

    fn query(
        &self,
        deps: &MockDeps,
        msg: Binary
    ) -> StdResult<Binary> {
        router::query(deps, from_binary(&msg)?)
    }
}
