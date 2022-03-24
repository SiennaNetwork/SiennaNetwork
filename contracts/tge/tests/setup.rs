use crate::impl_contract_harness_default;
use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma::snip20_impl::msg::InitConfig;
use fadroma::snip20_impl::msg::{
    QueryAnswer, QueryMsg as Snip20QueryMsg, QueryPermission as Snip20Permission, QueryWithPermit,
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

pub const ADMIN: &str = "admin";

impl TGE {
    pub fn new() -> Self {
        let mut ensemble = ContractEnsemble::new(50);

        let mgmt_model = ensemble.register(Box::new(MGMT));
        let rpt_model = ensemble.register(Box::new(RPT));
        let token = ensemble.register(Box::new(Token));

        let schedule = Schedule::new(&[Pool::partial("TEST", 25, &[])]);

        let token = ensemble
            .instantiate(
                token.id,
                &Snip20InitMsg {
                    name: "REWARD".into(),
                    admin: Some("MGMT_CONTRACT".into()),
                    symbol: "TKN".into(),
                    decimals: 18,
                    initial_balances: None,
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
                        address: "REWARD".into(),
                        code_hash: token.code_hash.clone(),
                    },
                ).time(DEFAULT_EPOCH_START),
            )
            .unwrap();
        ensemble
            .execute(
                &snip20_sienna::msg::HandleMsg::AddMinters {
                    minters: vec!["admin".into(), "MGMT_CONTRACT".into()],
                    padding: None,
                },
                MockEnv::new(
                    "MGMT_CONTRACT",
                    ContractLink {
                        address: "REWARD".into(),
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
                    prefund: false,
                    schedule,
                    token: token.clone(),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "MGMT_CONTRACT".into(),
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
                        address: "REWARD".into(),
                        code_hash: token.code_hash.clone(),
                    },
                    portion: Uint128(25),
                },
                MockEnv::new(
                    ADMIN,
                    ContractLink {
                        address: "RPT_CONTRACT".into(),
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
                address: "RPT_CONTRACT".into(),
                code_hash: self.rpt.code_hash.clone(),
            }
        )
    }

    pub fn get_mgmt_env(&self, sender: &str) -> MockEnv {
        MockEnv::new(
            sender.clone(),
            ContractLink {
                address: "MGMT_CONTRACT".into(),
                code_hash: self.mgmt.code_hash.clone(),
            }
        )
    }

    pub fn get_mgmt_env_as_admin(&self) -> MockEnv {
        MockEnv::new(
            ADMIN,
            ContractLink {
                address: "MGMT_CONTRACT".into(),
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


    pub fn claim_for(&mut self, user_name: &str, seconds_after: u64) {
        self.ensemble.execute(
            &sienna_mgmt::HandleMsg::Claim {}, 
            self.get_mgmt_env(user_name).time(DEFAULT_EPOCH_START + seconds_after)
        ).unwrap();
    }

}

impl Default for TGE {
    fn default() -> Self {
        TGE::new()
    }
}
