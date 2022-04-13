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
                InitMsg as Snip20InitMsg,
                HandleMsg as Snip20HandleMsg,
                QueryMsg as Snip20QueryMsg,
                QueryPermission as Snip20Permission,
                QueryWithPermit, InitialBalance, QueryAnswer, InitConfig
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
use rewards::{auth::AuthHandle, Response};

#[cfg(feature = "gov")]
use rewards::gov::{
    poll::{Poll, PollInfo},
    query::GovernanceQuery,
    response::GovernanceResponse,
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
                &sienna_rewards::Init {
                    admin: Some(ADMIN.into()),
                    config: sienna_rewards::config::RewardsConfig {
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

    pub fn mint(
        &mut self,
        token: ContractLink<HumanAddr>,
        recipient: impl Into<HumanAddr>,
        amount: Uint128
    ) {
        self.ensemble.execute(
            &Snip20HandleMsg::Mint {
                recipient: recipient.into(),
                amount,
                memo: None,
                padding: None
            },
            MockEnv::new(ADMIN, token)
        ).unwrap();
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

    #[cfg(feature = "gov")]
    pub fn get_poll(&self, id: u64, now: u64) -> PollInfo {
        let result = self
            .ensemble
            .query(
                self.rewards.address.clone(),
                &rewards::Query::Governance(GovernanceQuery::Poll { id, now }),
            )
            .unwrap();
        match result {
            rewards::Response::Governance(GovernanceResponse::Poll(poll)) => poll,
            _ => panic!("wrong response"),
        }
    }

    #[cfg(feature = "gov")]
    pub fn get_polls(&self, page: u64, take: u64, asc: bool, now: u64) -> Vec<Poll> {
        let result = self
            .ensemble
            .query(
                self.rewards.address.clone(),
                &rewards::Query::Governance(GovernanceQuery::Polls {
                    now,
                    page,
                    take,
                    asc,
                }),
            )
            .unwrap();
        match result {
            rewards::Response::Governance(GovernanceResponse::Polls {
                polls,
                total: _,
                total_pages: _,
            }) => polls,
            _ => panic!("wrong response"),
        }
    }

    pub fn deposit_lp_into_rewards(&mut self, address: impl Into<HumanAddr>, amount: Uint128) {
        let config = self.get_rewards_config();

        self.ensemble
            .execute(
                &rewards::fadroma::snip20_impl::msg::HandleMsg::Send {
                    recipient: self.rewards.address.clone(),
                    recipient_code_hash: None,
                    amount,
                    msg: None,
                    memo: None,
                    padding: None
                },
                MockEnv::new(address, config.lp_token.unwrap()),
            )
            .unwrap();
    }

    pub fn fund_rewards(&mut self, amount: Uint128) {
        let config = self.get_rewards_config();

        self.mint(
            config.reward_token.unwrap(),
            self.rewards.address.clone(),
            amount
        )
    }

    pub fn get_rewards_config(&self) -> rewards::config::RewardsConfig {
        let resp: rewards::Response = self.ensemble.query(
            self.rewards.address.clone(),
            rewards::Query::Rewards(rewards::query::RewardsQuery::Config)
        ).unwrap();

        match resp {
            rewards::Response::Rewards(resp) => match resp {
                rewards::query::RewardsResponse::Config(config) => config,
                _ => panic!("Expecting rewards::query::RewardsResponse::Config")
            },
            _ => panic!("sienna_rewards::Response::Rewards")
        }
    }

    pub fn get_rewards_user(&self, address: impl Into<HumanAddr>, at: u64) -> rewards::account::Account {
        let resp: rewards::Response = self.ensemble.query(
            self.rewards.address.clone(),
            rewards::Query::Rewards(rewards::query::RewardsQuery::WithPermit {
                query: rewards::query::QueryWithPermit::UserInfo { at },
                permit: rewards::permit::Permit::new(
                    address.into(),
                    vec![ rewards::query::RewardsPermissions::UserInfo ],
                    vec![ self.rewards.address.clone() ],
                    "user_info"
                )
            })
        ).unwrap();

        match resp {
            Response::Rewards(resp) => match resp {
                rewards::query::RewardsResponse::UserInfo(account) => account,
                _ => panic!("Expecting rewards::query::RewardsResponse")
            },
            _ => panic!("Expecting rewards::Response")
        }
    }

    pub fn set_rewards_viewing_key(&mut self, address: impl Into<HumanAddr>, key: String) {
        self.ensemble
            .execute(
                &rewards::Handle::Auth(AuthHandle::SetViewingKey { key, padding: None }),
                MockEnv::new(address, self.rewards.to_owned().try_into().unwrap()),
            )
            .unwrap();
    }

    pub fn get_rewards_staked(&mut self, address: impl Into<HumanAddr>, key: String) -> Uint128 {
        let response: Response = self
            .ensemble
            .query(
                self.rewards.address.clone(),
                &rewards::Query::Balance {
                    address: address.into(),
                    key
                },
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
