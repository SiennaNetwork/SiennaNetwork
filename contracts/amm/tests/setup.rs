use std::convert::TryInto;

use amm_shared::{
    fadroma::{
        ContractLink,
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

use factory::contract as factory;
use exchange::contract as exchange;
use router::contract as router;
use lp_token;

pub const ADMIN: &str = "admin";
pub const USERS: &[&str] = &[ "user_a", "user_b", "user_c" ];
pub const BURNER: &str = "burner_acc";
pub const INITIAL_BALANCE: Uint128 = Uint128(1000_000_000_000_000_000_000);
pub const NATIVE_DENOM: &str = "uscrt";

pub struct Amm {
    pub ensemble: ContractEnsemble,
    pub factory: ContractLink<HumanAddr>
}

impl Amm {
    pub fn new() -> Self {
        use std::iter::FromIterator;
    
        let mut ensemble = ContractEnsemble::new(200);
    
        let factory = ensemble.register(Factory);
        let snip20 = ensemble.register(Token);
        let lp_token = ensemble.register(LpToken);
        let pair = ensemble.register(Pair);
        let _router = ensemble.register(Router);
        
        let factory = ensemble.instantiate(
            factory.id,
            &msg::factory::InitMsg {
                lp_token_contract: lp_token,
                pair_contract: pair,
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
    
        for i in 1..=2 {
            let name = format!("TOKEN_{}", i);
    
            let token = ensemble.instantiate(
                snip20.id,
                &Snip20InitMsg {
                    name: name.clone(),
                    admin: None,
                    symbol: format!("TKN{}", i),
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

        ensemble.execute(
            &msg::factory::HandleMsg::CreateExchange {
                pair: TokenPair(
                    TokenType::from(tokens[0].clone()),
                    TokenType::NativeToken {
                        denom: NATIVE_DENOM.into()
                    },
                ),
                entropy: Binary::from(b"whatever")
            },
            MockEnv::new(ADMIN, factory.clone())
        ).unwrap();
    
        Amm {
            ensemble,
            factory
        }
    }

    pub fn get_pairs(&self) -> Vec<Exchange<HumanAddr>> {
        let response = self.ensemble.query(
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

    pub fn increase_allowances(&mut self, pair: &Exchange<HumanAddr>) {
        for token in pair.pair.into_iter() {
            if token.is_native_token() {
                continue;
            }

            let token: ContractLink<HumanAddr> = token.to_owned().try_into().unwrap();
    
            for user in USERS {
                self.ensemble.execute(
                    &msg::snip20::HandleMsg::IncreaseAllowance {
                        spender: pair.contract.address.clone(),
                        amount: Uint128(u128::MAX),
                        expiration: None,
                        padding: None
                    },
                    MockEnv::new(*user, token.clone())
                ).unwrap();
            }
        }
    }

    pub fn get_balance(
        &self,
        address: impl Into<HumanAddr>,
        token: TokenType<HumanAddr>
    ) -> Uint128 {
        match token {
            TokenType::CustomToken { contract_addr, .. } => {
                let result = self.ensemble.query(contract_addr.clone(), Snip20QueryMsg::WithPermit {
                    permit: Permit::<Snip20Permission>::new(
                        address,
                        vec![ Snip20Permission::Balance ],
                        vec![ contract_addr ],
                        "balance"
                    ),
                    query: QueryWithPermit::Balance {}
                }).unwrap();
        
                match result {
                    QueryAnswer::Balance { amount } => amount,
                    _ => panic!("Expecting QueryAnswer::Balance")
                }
            },
            TokenType::NativeToken { denom } => {
                self.ensemble.balances(address).unwrap().get(&denom).unwrap().to_owned()
            }
        }
    }

    pub fn get_lp_balance(
        &self,
        address: impl Into<HumanAddr>,
        pair: HumanAddr
    ) -> Uint128 {
        let result = self.ensemble.query(
            pair,
            msg::exchange::QueryMsg::PairInfo
        ).unwrap();

        match result {
            msg::exchange::QueryMsgResponse::PairInfo { liquidity_token, .. } => {
                self.get_balance(address, liquidity_token.into())
            }
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
