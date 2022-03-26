use crate::impl_contract_harness_default;
use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma::snip20_impl::msg::{InitConfig, InitialBalance};
use fadroma::snip20_impl::msg::{
    QueryAnswer, QueryMsg as Snip20QueryMsg, QueryPermission as Snip20Permission, QueryWithPermit, HandleMsg as Snip20HandleMsg
};
use fadroma::{
    from_binary, snip20_impl, snip20_impl::msg::InitMsg as Snip20InitMsg, Binary, ContractLink,
    Env, HandleResponse, HumanAddr, InitResponse, StdResult,
};
use fadroma::{Permit, StdError, Uint128};
use sienna_mgmt;
use sienna_rpt::{self, LinearMap};
use sienna_schedule::{Account, Pool, Schedule};


pub const DEFAULT_EPOCH_START: u64 = 1571797419;
const REWARD_TOKEN_ADDR: &str = "REWARD";
pub const ADMIN: &str = "admin";
pub const MGMT_ADDR: &str = "MGMT_CONTRACT";
pub const RPT_ADDR: &str = "RPT_CONTRACT";

pub struct MGMT;
impl_contract_harness_default!(MGMT, sienna_mgmt);

pub struct RPT;
impl_contract_harness_default!(RPT, sienna_rpt);

pub struct Token;
impl ContractHarness for Token {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        snip20_impl::snip20_init(
            deps,
            env,
            from_binary(&msg)?,
            snip20_impl::DefaultSnip20Impl,
        )
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        snip20_impl::snip20_handle(
            deps,
            env,
            from_binary(&msg)?,
            snip20_impl::DefaultSnip20Impl,
        )
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        snip20_impl::snip20_query(deps, from_binary(&msg)?, snip20_impl::DefaultSnip20Impl)
    }
}

pub struct TGE {
    pub ensemble: ContractEnsemble,
    pub mgmt: ContractLink<HumanAddr>,
    pub rpt: ContractLink<HumanAddr>,
    pub token: ContractLink<HumanAddr>,
}


impl TGE {
    pub fn new(prefund: bool) -> Self {
        let mut ensemble = ContractEnsemble::new(50);
        let mgmt_model = ensemble.register(Box::new(MGMT));
        let rpt_model = ensemble.register(Box::new(RPT));
        let token = ensemble.register(Box::new(Token));
        let schedule = Schedule::new(&[Pool::partial("TEST", 25, &[])]);

        
        let token = ensemble
            .instantiate(
                token.id,
                &Snip20InitMsg {
                    name: REWARD_TOKEN_ADDR.into(),
                    admin: Some(MGMT_ADDR.into()),
                    symbol: "TKN".into(),
                    decimals: 18,
                    initial_balances: match prefund {
                        true => Some(vec![InitialBalance {
                                    address: MGMT_ADDR.into(),
                                    amount:  Uint128(1_000_000_000_000_000_000_000),
                                }]),
                        false => None
                    },
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
                        address: REWARD_TOKEN_ADDR.into(),
                        code_hash: token.code_hash.clone(),
                    },
                ).time(DEFAULT_EPOCH_START),
            )
            .unwrap();
        ensemble
            .execute(
                &snip20_sienna::msg::HandleMsg::AddMinters {
                    minters: vec![ADMIN.into(), MGMT_ADDR.into()],
                    padding: None,
                },
                MockEnv::new(
                    MGMT_ADDR,
                    ContractLink {
                        address: REWARD_TOKEN_ADDR.into(),
                        code_hash: token.code_hash.clone(),
                    },
                ),
            )
            .unwrap();

        let mgmt = ensemble
            .instantiate(
                mgmt_model.id,
                &sienna_mgmt::InitMsg {
                    admin: Some(ADMIN.into()),
                    prefund,
                    schedule,
                    token: token.clone(),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: MGMT_ADDR.into(),
                        code_hash: mgmt_model.code_hash.clone(),
                    },
                ),
            )
            .unwrap();

        let distribution = LinearMap(vec![
            (HumanAddr::from(ADMIN), Uint128(20)),
            (HumanAddr::from("STRANGER"), Uint128(5)),
        ]);

        let rpt = ensemble
            .instantiate(
                rpt_model.id,
                &sienna_rpt::InitMsg {
                    admin: Some(ADMIN.into()),
                    mgmt: mgmt.clone(),
                    distribution,
                    token: ContractLink {
                        address: REWARD_TOKEN_ADDR.into(),
                        code_hash: token.code_hash.clone(),
                    },
                    portion: Uint128(25),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: RPT_ADDR.into(),
                        code_hash: rpt_model.code_hash,
                    },
                ),
            )
            .unwrap();

        Self {
            ensemble,
            mgmt,
            token,
            rpt,
        }
    }

    pub fn get_rpt_env(&self, sender: &str) -> MockEnv {
        MockEnv::new(
            sender.clone(),
            ContractLink {
                address: RPT_ADDR.into(),
                code_hash: self.rpt.code_hash.clone(),
            }
        )
    }

    pub fn get_mgmt_env(&self, sender: &str) -> MockEnv {
        MockEnv::new(
            sender.clone(),
            ContractLink {
                address: MGMT_ADDR.into(),
                code_hash: self.mgmt.code_hash.clone(),
            }
        )
    }

    pub fn get_mgmt_env_as_admin(&self) -> MockEnv {
        MockEnv::new(
            ADMIN,
            ContractLink {
                address: MGMT_ADDR.into(),
                code_hash: self.mgmt.code_hash.clone(),
            }
        )
    }

    pub fn add_account(
        &mut self,
        pool_name: String,
        account: Account<HumanAddr>,
    ) -> Result<(), StdError> {
        self.ensemble.execute(
            &sienna_mgmt::HandleMsg::AddAccount { pool_name, account },
            self.get_mgmt_env_as_admin()
        )
    }

    pub fn query_schedule(&self) -> Schedule<HumanAddr> {
        let schedule = self
            .ensemble
            .query(
                self.mgmt.address.clone(),
                &sienna_mgmt::QueryMsg::Schedule {},
            )
            .unwrap();

        schedule
    }
    pub fn query_balance(&self, address: &str) -> Uint128 {
        let result = self
            .ensemble
            .query(
                self.token.address.clone(),
                Snip20QueryMsg::WithPermit {
                    permit: Permit::<Snip20Permission>::new(
                        address,
                        vec![Snip20Permission::Balance],
                        vec![self.token.address.clone()],
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

    pub fn set_shedule(&mut self, schedule: Schedule<HumanAddr>) -> StdResult<()>{
        self.ensemble.execute(
            &sienna_mgmt::HandleMsg::Configure {
                schedule
            }, 
            self.get_mgmt_env_as_admin()
        )
    }
    pub fn launch(&mut self) -> StdResult<()>  {
        self.ensemble.execute(
            &sienna_mgmt::HandleMsg::Launch {}, 
            self.get_mgmt_env_as_admin().time(DEFAULT_EPOCH_START)
        )
    }


    pub fn claim_for(&mut self, user_name: &str, seconds_after: u64) -> StdResult<()> {
        self.ensemble.execute(
            &sienna_mgmt::HandleMsg::Claim {}, 
            self.get_mgmt_env(user_name).time(DEFAULT_EPOCH_START + seconds_after)
        )
    }

}

impl Default for TGE {
    fn default() -> Self {
        TGE::new(false)
    }
}

pub trait AccountFactory {
    fn create(name: &str, amount: u128, duration: u64, interval: u64) -> Account<HumanAddr>;
    fn create_ext(name: String, address: &str, amount: u128, cliff: u128, duration: u64, interval: u64, start_at: u64) -> Account<HumanAddr>;
}

impl AccountFactory for Account<HumanAddr> {
    fn create(name: &str, amount: u128, duration: u64, interval: u64) -> Account<HumanAddr>{
        Self::create_ext(name.to_string(), name, amount, 0, duration, interval, 0)
    }
    fn create_ext(name: String, address: &str, amount: u128, cliff: u128, duration: u64, interval: u64, start_at: u64) -> Account<HumanAddr>{
        Account {
            name,
            address: HumanAddr::from(address),
            amount: Uint128(amount),
            cliff: Uint128(cliff),
            duration,
            interval,
            start_at
        }
    }
}