use crate::impl_contract_harness_default;
use fadroma::ensemble::{ContractEnsemble, ContractHarness, MockDeps, MockEnv};
use fadroma::Uint128;
use fadroma::{
    from_binary, snip20_impl, snip20_impl::msg::InitMsg as Snip20InitMsg, Binary, ContractLink,
    Env, HandleResponse, HumanAddr, InitResponse, StdResult,
};
use sienna_mgmt;
use sienna_rpt::{self, LinearMap};
use sienna_schedule::{Pool, Schedule};

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
}

pub const ADMIN: &str = "admin";

impl TGE {
    pub fn new() -> Self {
        let mut ensemble = ContractEnsemble::new(50);

        let mgmt_model = ensemble.register(Box::new(MGMT));
        let rpt_model = ensemble.register(Box::new(RPT));
        let token = ensemble.register(Box::new(Token));

        let schedule = Schedule::new(&[Pool::full("test", &[])]);

        let token = ensemble
            .instantiate(
                token.id,
                &Snip20InitMsg {
                    name: "REWARD".into(),
                    admin: None,
                    symbol: "TKN".into(),
                    decimals: 18,
                    initial_balances: None,
                    initial_allowances: None,
                    prng_seed: Binary::from(b"whatever"),
                    config: None,
                    callback: None,
                },
                MockEnv::new(
                    ADMIN,
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
                        address: "mgmt_contract".into(),
                        code_hash: mgmt_model.code_hash,
                    },
                ),
            )
            .unwrap();

        let distribution = LinearMap(vec![(HumanAddr::from(ADMIN), Uint128(2500))]);

        let rpt = ensemble
            .instantiate(
                rpt_model.id,
                &sienna_rpt::InitMsg {
                    admin: None,
                    mgmt: mgmt.clone(),
                    distribution,
                    token: ContractLink {
                        address: "REWARD".into(),
                        code_hash: token.code_hash.clone(),
                    },
                    portion: Uint128(2500),
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
            rpt,
        }
    }
}

impl Default for TGE {
    fn default() -> Self {
        TGE::new()
    }
}
