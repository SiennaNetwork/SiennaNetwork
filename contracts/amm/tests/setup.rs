use std::convert::TryInto;

use amm_shared::{
    fadroma::{
        auth::Permit,
        cosmwasm_std::{
            from_binary, Binary, Env, HandleResponse, HumanAddr, InitResponse, StdResult, Uint128,
        },
        ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv},
        snip20_impl::{
            msg::{
                InitConfig, InitMsg as Snip20InitMsg, InitialBalance, QueryAnswer,
                QueryMsg as Snip20QueryMsg, QueryPermission as Snip20Permission, QueryWithPermit,
            },
            snip20_handle, snip20_init, snip20_query, Snip20, SymbolValidation,
        },
        ContractLink,
    },
    msg, Exchange, ExchangeSettings, Fee, Pagination, TokenPair, TokenType,
};

use exchange::contract as exchange;
use factory::contract as factory;
use lp_token;
use rewards::{
    auth::AuthHandle,
    gov::{poll::{PollInfo}, query::GovernanceQuery, response::GovernanceResponse},
    handle::RewardsHandle,
    Response,
};
use router::contract as router;
use sienna_rewards as rewards;

pub const ADMIN: &str = "admin";
pub const USERS: &[&str] = &["user_a", "user_b", "user_c"];
pub const BURNER: &str = "burner_acc";
pub const INITIAL_BALANCE: Uint128 = Uint128(1000_000_000_000_000_000_000);
pub const NATIVE_DENOM: &str = "uscrt";

pub struct Amm {
    pub ensemble: ContractEnsemble,
    pub factory: ContractLink<HumanAddr>,
    pub rewards: ContractLink<HumanAddr>,
}

impl Amm {
    pub fn new() -> Self {
        use std::iter::FromIterator;

        let mut ensemble = ContractEnsemble::new(200);

        let factory = ensemble.register(Box::new(Factory));
        let snip20 = ensemble.register(Box::new(Token));
        let lp_token = ensemble.register(Box::new(LpToken));
        let pair = ensemble.register(Box::new(Pair));
        let _router = ensemble.register(Box::new(Router));
        let rewards = ensemble.register(Box::new(Rewards));

        let factory = ensemble
            .instantiate(
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
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "factory".into(),
                        code_hash: factory.code_hash.clone(),
                    },
                ),
            )
            .unwrap();

        let mut tokens = Vec::new();

        for i in 1..=2 {
            let name = format!("TOKEN_{}", i);

            let token = ensemble
                .instantiate(
                    snip20.id,
                    &Snip20InitMsg {
                        name: name.clone(),
                        admin: None,
                        symbol: format!("TKN{}", i),
                        decimals: 18,
                        initial_balances: Some(
                            USERS
                                .iter()
                                .map(|x| InitialBalance {
                                    address: (*x).into(),
                                    amount: INITIAL_BALANCE,
                                })
                                .collect(),
                        ),
                        initial_allowances: None,
                        prng_seed: Binary::from(b"whatever"),
                        config: Some(
                            InitConfig::builder()
                                .public_total_supply()
                                .enable_mint()
                                .build(),
                        ),
                        callback: None,
                    },
                    MockEnv::new(
                        ADMIN,
                        ContractLink {
                            address: name.into(),
                            code_hash: snip20.code_hash.clone(),
                        },
                    ),
                )
                .unwrap();

            tokens.push(token);
        }

        let rewards = ensemble
            .instantiate(
                rewards.id,
                &msg::rewards::Init {
                    admin: Some(ADMIN.into()),
                    config: msg::rewards::RewardsConfig {
                        bonding: None,
                        lp_token: Some(ContractLink {
                            address: tokens[0].address.clone(),
                            code_hash: tokens[0].code_hash.clone(),
                        }),
                        reward_token: Some(ContractLink {
                            address: tokens[1].address.clone(),
                            code_hash: tokens[1].code_hash.clone(),
                        }),
                        reward_vk: Some("whatever".to_string()),
                        timekeeper: Some(ADMIN.into()),
                    },
                    governance_config: None,
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "rewards".into(),
                        code_hash: rewards.code_hash,
                    },
                ),
            )
            .unwrap();

        for user in USERS {
            ensemble
                .execute(
                    &msg::snip20::HandleMsg::IncreaseAllowance {
                        spender: rewards.address.clone(),
                        amount: Uint128(u128::MAX),
                        expiration: None,
                        padding: None,
                    },
                    MockEnv::new(*user, tokens[0].to_owned().try_into().unwrap()),
                )
                .unwrap();
        }

        for pair in tokens.chunks(2) {
            let pair = Vec::from_iter(pair);

            ensemble
                .execute(
                    &msg::factory::HandleMsg::CreateExchange {
                        pair: TokenPair(
                            TokenType::from(pair[0].clone()),
                            TokenType::from(pair[1].clone()),
                        ),
                        entropy: Binary::from(b"whatever"),
                    },
                    MockEnv::new(ADMIN, factory.clone()),
                )
                .unwrap();
        }

        ensemble
            .execute(
                &msg::factory::HandleMsg::CreateExchange {
                    pair: TokenPair(
                        TokenType::from(tokens[0].clone()),
                        TokenType::NativeToken {
                            denom: NATIVE_DENOM.into(),
                        },
                    ),
                    entropy: Binary::from(b"whatever"),
                },
                MockEnv::new(ADMIN, factory.clone()),
            )
            .unwrap();

        Amm {
            ensemble,
            factory,
            rewards,
        }
    }

    pub fn get_pairs(&self) -> Vec<Exchange<HumanAddr>> {
        let response = self
            .ensemble
            .query(
                self.factory.address.clone(),
                &msg::factory::QueryMsg::ListExchanges {
                    pagination: Pagination {
                        start: 0,
                        limit: 30,
                    },
                },
            )
            .unwrap();

        match response {
            msg::factory::QueryResponse::ListExchanges { exchanges } => exchanges,
            _ => panic!("Expected QueryResponse::ListExchanges"),
        }
    }

    pub fn increase_allowances(&mut self, pair: &Exchange<HumanAddr>) {
        for token in pair.pair.into_iter() {
            if token.is_native_token() {
                continue;
            }

            let token: ContractLink<HumanAddr> = token.to_owned().try_into().unwrap();

            for user in USERS {
                self.ensemble
                    .execute(
                        &msg::snip20::HandleMsg::IncreaseAllowance {
                            spender: pair.contract.address.clone(),
                            amount: Uint128(u128::MAX),
                            expiration: None,
                            padding: None,
                        },
                        MockEnv::new(*user, token.clone()),
                    )
                    .unwrap();
            }
        }
    }

    pub fn get_balance(
        &self,
        address: impl Into<HumanAddr>,
        token: TokenType<HumanAddr>,
    ) -> Uint128 {
        match token {
            TokenType::CustomToken { contract_addr, .. } => {
                let result = self
                    .ensemble
                    .query(
                        contract_addr.clone(),
                        Snip20QueryMsg::WithPermit {
                            permit: Permit::<Snip20Permission>::new(
                                address,
                                vec![Snip20Permission::Balance],
                                vec![contract_addr],
                                "balance",
                            ),
                            query: QueryWithPermit::Balance {},
                        },
                    )
                    .unwrap();

                match result {
                    QueryAnswer::Balance { amount } => amount,
                    _ => panic!("Expecting QueryAnswer::Balance"),
                }
            }
            TokenType::NativeToken { denom } => self
                .ensemble
                .balances(address)
                .unwrap()
                .get(&denom)
                .unwrap()
                .to_owned(),
        }
    }

    pub fn get_lp_balance(&self, address: impl Into<HumanAddr>, pair: HumanAddr) -> Uint128 {
        let result = self
            .ensemble
            .query(pair, msg::exchange::QueryMsg::PairInfo)
            .unwrap();

        match result {
            msg::exchange::QueryMsgResponse::PairInfo {
                liquidity_token, ..
            } => self.get_balance(address, liquidity_token.into()),
        }
    }
    pub fn get_poll(&self, id: u64, now: u64) -> PollInfo {
        let result = self
            .ensemble
            .query(
                self.rewards.address.clone(),
                &sienna_rewards::Query::Governance(GovernanceQuery::Poll { id, now }),
            )
            .unwrap();
        match result {
            sienna_rewards::Response::Governance(GovernanceResponse::Poll(poll)) => poll,
            _ => panic!("wrong response"),
        }
    }
    pub fn deposit_lp_into_rewards(&mut self, address: HumanAddr, amount: Uint128) {
        self.ensemble
            .execute(
                &sienna_rewards::Handle::Rewards(RewardsHandle::Deposit { amount }),
                MockEnv::new(address, self.rewards.to_owned().try_into().unwrap()),
            )
            .unwrap();
    }
    pub fn set_rewards_viewing_key(&mut self, address: HumanAddr, key: String) {
        self.ensemble
            .execute(
                &sienna_rewards::Handle::Auth(AuthHandle::SetViewingKey { key, padding: None }),
                MockEnv::new(address, self.rewards.to_owned().try_into().unwrap()),
            )
            .unwrap();
    }
    pub fn get_rewards_staked(&mut self, address: HumanAddr, key: String) -> Uint128 {
        let response: Response = self
            .ensemble
            .query(
                self.rewards.address.clone(),
                &sienna_rewards::Query::Balance { address, key },
            )
            .unwrap();

        match response {
            Response::Balance { amount } => amount,
            _ => panic!("wrong type returned for balance"),
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
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        snip20_init(deps, env, from_binary(&msg)?, Self)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        snip20_handle(deps, env, from_binary(&msg)?, Self)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        snip20_query(deps, from_binary(&msg)?, Self)
    }
}

pub struct Factory;

impl ContractHarness for Factory {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        factory::init(deps, env, from_binary(&msg)?)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        factory::handle(deps, env, from_binary(&msg)?)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        factory::query(deps, from_binary(&msg)?)
    }
}

pub struct Rewards;
impl ContractHarness for Rewards {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        rewards::init(deps, env, from_binary(&msg)?)
    }
    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        rewards::handle(deps, env, from_binary(&msg)?)
    }
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        rewards::query(deps, from_binary(&msg)?)
    }
}

pub struct Pair;

impl ContractHarness for Pair {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        exchange::init(deps, env, from_binary(&msg)?)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        exchange::handle(deps, env, from_binary(&msg)?)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        exchange::query(deps, from_binary(&msg)?)
    }
}

pub struct LpToken;

impl ContractHarness for LpToken {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        lp_token::init(deps, env, from_binary(&msg)?)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        lp_token::handle(deps, env, from_binary(&msg)?)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        lp_token::query(deps, from_binary(&msg)?)
    }
}

pub struct Router;

impl ContractHarness for Router {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        router::init(deps, env, from_binary(&msg)?)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        router::handle(deps, env, from_binary(&msg)?)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        router::query(deps, from_binary(&msg)?)
    }
}
