// Look Ma, no macros! ////////////////////////////////////////////////////////////////////////////
#![cfg(test)]
#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(unreachable_patterns)]
#![allow(non_snake_case)]

mod test_0000_setup;
mod test_0100_operate;
mod test_0300_migrate;

use prettytable::{/*Row, Cell,*/ format, Table};
//use yansi::Paint;

use crate::time_utils::{Duration, Moment, DAY};
use crate::total::Total;
use crate::*;
use fadroma::{
    secret_toolkit::snip20,
    composability::{ClonableMemoryStorage, MockExtern},
    testing::{MockApi, mock_env},
    ISnip20, BLOCK_SIZE
};

pub use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};

compose!(MockExtern<S, A, Q>);
pub type Deps = MockExtern<ClonableMemoryStorage, MockApi, RewardsMockQuerier>;

#[derive(Clone)]
pub struct Context {
    pub init_called: bool,
    pub rng: StdRng,
    pub name: String,
    pub link: ContractLink<HumanAddr>,
    pub table: Table,
    pub deps: Deps,
    pub initiator: HumanAddr,
    pub env: Env,
    pub time: Moment,

    pub reward_vk: String,
    pub reward_token: ISnip20,
    pub lp_token: ISnip20,
    pub closed: Option<CloseSeal>,
    pub bonding: u64,
}

impl Context {
    /// Init a test context with a single contract
    /// TODO: support multiple contracts in the same context
    pub fn new(name: &str) -> Self {
        let mut table = Table::new();

        table.set_format(
            format::FormatBuilder::new()
                .separator(
                    format::LinePosition::Title,
                    format::LineSeparator::new('-', '-', '-', '-'),
                )
                .column_separator('|')
                .borders('|')
                .padding(1, 1)
                .build(),
        );

        table.set_titles(row![
            rb->"Time",
            b->"Sender",
            b->"Recipient",
            b->"Data"
        ]);

        let initiator = HumanAddr::from("Admin");
        let time = 1;

        color_backtrace::install();

        let address = HumanAddr::from(format!("{}_addr", &name));
        let code_hash = format!("{}_hash", &name).to_string();

        let mut rng = StdRng::seed_from_u64(1);
        let bonding = rng.gen_range(100..200);

        Self {
            init_called: false,
            rng,
            name: name.to_string(),
            link: ContractLink {
                address: address.clone(),
                code_hash: code_hash.clone(),
            },
            table,

            deps: MockExtern::new(RewardsMockQuerier::new()),

            reward_vk: "reward_vk".to_string(),
            reward_token: ISnip20::attach(ContractLink {
                address: HumanAddr::from("SIENNA"),
                code_hash: "SIENNA_hash".into(),
            }),

            lp_token: ISnip20::attach(ContractLink {
                address: HumanAddr::from("LP_TOKEN"),
                code_hash: "LP_hash".into(),
            }),

            env: Env {
                block: BlockInfo {
                    height: 0,
                    time,
                    chain_id: "fadroma".into(),
                },
                message: MessageInfo {
                    sender: initiator.clone(),
                    sent_funds: vec![],
                },
                contract: ContractInfo {
                    address: address.clone(),
                },
                contract_key: Some("".into()),
                contract_code_hash: code_hash.clone(),
            },
            initiator,
            time,
            closed: None,
            bonding,
        }
    }

    /// Clone the context up to this point, branching off the test execution.
    pub fn branch<F: FnMut(Context) -> ()>(&mut self, name: &str, mut f: F) -> &mut Self {
        let mut context = self.clone();
        let name = format!("{}_{}", self.name, name).to_string();
        context.name = name.to_string();
        context.table.add_row(row!["", "", "", ""]);
        context
            .table
            .add_row(row![rb->self.time, "test", "branch", b->&name]);
        context.table.add_row(row!["", "", "", ""]);
        f(context);
        self
    }

    // Time functions

    pub fn at(&mut self, t: Moment) -> &mut Self {
        self.time = t;
        self.env.block.time = self.time;
        self
    }

    pub fn after(&mut self, t: Duration) -> &mut Self {
        self.at(self.env.block.time + t)
    }

    pub fn tick(&mut self) -> &mut Self {
        self.after(1)
    }

    pub fn during<F: FnMut(u64, &mut Context) -> ()>(
        &mut self,
        n: Duration,
        mut f: F,
    ) -> &mut Self {
        for i in 1..=n {
            self.tick();
            f(i, self);
        }
        self
    }

    pub fn later(&mut self) -> &mut Self {
        let t = self.rng.gen_range(0..self.bonding / 10);
        self.after(t)
    }

    // Address functions

    pub fn set_address(&mut self, address: &str) -> &mut Self {
        self.initiator = HumanAddr::from(address);
        self.env.message.sender = self.initiator.clone();
        self
    }

    pub fn admin(&mut self) -> &mut Self {
        self.set_address("Admin")
    }

    pub fn badman(&mut self) -> &mut Self {
        self.set_address("Badman")
    }

    pub fn user(&mut self, address: &str) -> &mut Self {
        self.set_address(address)
    }

    pub fn epoch(&mut self, next_epoch: Moment, portion: u128) -> &mut Self {
        self.after(self.bonding);
        self.fund(portion);
        self.test_handle(
            Handle::Rewards(RewardsHandle::BeginEpoch { next_epoch }),
            Ok(HandleResponse::default()),
        );
        self
    }

    /// Shorthand for the free-standing `test_handle` function
    /// with table, deps and env pre-populated from the `Context`
    pub fn test_handle(&mut self, msg: Handle, expected: StdResult<HandleResponse>) -> &mut Self {
        test_handle(&mut self.table, &mut self.deps, &self.env, msg, expected);
        self
    }

    pub fn test_query(&mut self, msg: Query, expected: StdResult<Response>) -> &mut Self {
        assert_eq!(Contract::query(&mut self.deps, msg), expected);
        self
    }

    // Contract-specific implementation below

    pub fn epoch_must_increment(&mut self, current_epoch: Moment, next_epoch: Moment) -> &mut Self {
        assert_eq!(
            Contract::handle(
                &mut self.deps,
                self.env.clone(),
                Handle::Rewards(RewardsHandle::BeginEpoch { next_epoch })
            ),
            errors::invalid_epoch_number(current_epoch, next_epoch)
        );
        self
    }

    pub fn fund(&mut self, amount: u128) -> &mut Self {
        self.table.add_row(row!["", "", "", ""]);
        self.table.add_row(row![
            rb->self.time,
            "RPT",
            self.link.address.clone(),
            b->format!("vest {}", &amount)
        ]);
        self.deps
            .querier
            .increment_balance(&self.reward_token.link.address, amount);
        self
    }

    pub fn init(&mut self) -> &mut Self {
        if self.init_called {
            panic!("context.init() called twice");
        }
        self.init_called = true;

        assert_eq!(
            Contract::init(
                &mut self.deps,
                self.env.clone(),
                Init {
                    admin: None,
                    governance_config: Some(GovernanceConfig::default()),
                    config: RewardsConfig {
                        lp_token: Some(self.lp_token.link.clone()),
                        reward_token: Some(self.reward_token.link.clone()),
                        reward_vk: Some(self.reward_vk.clone()),
                        bonding: Some(self.bonding),
                        timekeeper: Some(HumanAddr::from("Admin")),
                    }
                }
            ),
            InitResponse::default()
                .msg(snip20::register_receive_msg(
                        self.env.contract_code_hash.clone(),
                        None,
                        BLOCK_SIZE,
                        self.lp_token.link.code_hash.clone(),
                        self.lp_token.link.address.clone()
                    ).unwrap()
                )
                .unwrap()
                .msg(snip20::set_viewing_key_msg(
                        self.reward_vk.clone(),
                        None,
                        BLOCK_SIZE,
                        self.reward_token.link.code_hash.clone(),
                        self.reward_token.link.address.clone()
                    )
                    .unwrap()
                )
        );

        assert_eq!(
            Contract::query(&self.deps, Query::TokenInfo {}),
            Ok(Response::TokenInfo {
                name: format!("Staked FooToken"),
                symbol: "FOO (staked)".into(),
                decimals: 0,
                total_supply: None
            })
        );

        assert_eq!(
            Contract::query(&self.deps, Query::Governance(GovernanceQuery::Config {})),
            Ok(Response::Governance(GovernanceResponse::Config(
                GovernanceConfig::default()
            )))
        );

        self
    }
    pub fn init_invalid(&mut self) -> &mut Self {
        let invalid_config = RewardsConfig {
            lp_token: None,
            reward_token: None,
            reward_vk: None,
            bonding: None,
            timekeeper: None,
        };
        assert!(Rewards::init(&mut self.deps, self.env.clone(), invalid_config).is_err());
        self
    }
    pub fn configures(&mut self, config: RewardsConfig) -> &mut Self {
        let mut expected = HandleResponse::default();
        if config.reward_vk.is_some() && config.reward_token.is_some() {
            expected.messages.push(
                snip20::set_viewing_key_msg(
                    config.reward_vk.clone().unwrap(),
                    None,
                    BLOCK_SIZE,
                    config.reward_token.clone().unwrap().code_hash,
                    config.reward_token.clone().unwrap().address,
                )
                .unwrap(),
            )
        }
        test_handle(
            &mut self.table,
            &mut self.deps,
            &self.env,
            Handle::Rewards(RewardsHandle::Configure(config)),
            Ok(expected),
        );
        self
    }
    pub fn sets_bonding(&mut self, bonding: Duration) -> &mut Self {
        self.configures(RewardsConfig {
            lp_token: None,
            reward_token: None,
            reward_vk: None,
            bonding: Some(bonding),
            timekeeper: None,
        })
    }
    pub fn cannot_configure(&mut self) -> &mut Self {
        assert_eq!(
            Rewards::handle(
                &mut self.deps,
                self.env.clone(),
                RewardsHandle::Configure(RewardsConfig {
                    lp_token: None,
                    reward_token: None,
                    reward_vk: None,
                    bonding: None,
                    timekeeper: None,
                })
            ),
            Err(StdError::unauthorized())
        );
        self
    }
    pub fn closes_pool(&mut self) -> &mut Self {
        let message = "closed";
        test_handle(
            &mut self.table,
            &mut self.deps,
            &self.env,
            Handle::Rewards(RewardsHandle::Close {
                message: message.to_string(),
            }),
            Ok(HandleResponse::default()),
        );
        self.closed = Some((self.time, message.to_string()));
        self
    }
    pub fn cannot_close_pool(&mut self) -> &mut Self {
        test_handle(
            &mut self.table,
            &mut self.deps,
            &self.env,
            Handle::Rewards(RewardsHandle::Close {
                message: String::from("closed"),
            }),
            Err(StdError::unauthorized()),
        );
        self
    }
    pub fn drains_pool(&mut self, key: &str) -> &mut Self {
        assert_eq!(
            Contract::handle(
                &mut self.deps,
                self.env.clone(),
                Handle::Drain {
                    snip20: self.reward_token.link.clone(),
                    key: key.into(),
                    recipient: None
                }
            ),
            Ok(HandleResponse {
                messages: vec![
                    self.reward_token
                        .increase_allowance(
                            &self.initiator,
                            Uint128(u128::MAX),
                            Some(self.env.block.time + DAY * 10000)
                        )
                        .unwrap(),
                    self.reward_token.set_viewing_key(key.into()).unwrap()
                ],
                log: vec![],
                data: None
            })
        );
        let vk: Option<ViewingKey> = self.deps.get(crate::RewardsConfig::REWARD_VK).unwrap();
        assert_eq!(vk.unwrap().0, String::from(key));
        self
    }
    pub fn cannot_drain(&mut self, key: &str) -> &mut Self {
        assert!(Contract::handle(
            &mut self.deps,
            self.env.clone(),
            Handle::Drain {
                snip20: self.reward_token.link.clone(),
                key: key.into(),
                recipient: None
            }
        )
        .is_err());
        self
    }
    pub fn set_vk(&mut self, key: &str) -> &mut Self {
        let msg = crate::AuthHandle::SetViewingKey {
            key: key.into(),
            padding: None,
        };
        assert_eq!(
            crate::Auth::handle(&mut self.deps, self.env.clone(), msg),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn deposits(&mut self, amount: u128) -> &mut Self {
        // YES, THIS IS A HACK - DEAL WITH IT
        let old_sender = self.env.message.sender.clone();
        self.env.message.sender = self.lp_token.link.address.clone();

        self.test_handle(
            Handle::Rewards(RewardsHandle::DepositReceiver {
                from: self.initiator.clone(),
                amount: amount.into()
            }),
            HandleResponse::default()
                .log("deposit", &amount.to_string())
        );

        self.env.message.sender = old_sender;

        self.deps
            .querier
            .increment_balance(&self.lp_token.link.address, amount);

        self
    }
    pub fn withdraws(&mut self, amount: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Withdraw {
                amount: amount.into(),
            }),
            HandleResponse::default().msg(
                self.lp_token
                    .transfer(&self.env.message.sender, amount.into())
                    .unwrap(),
            ),
        );
        self.deps
            .querier
            .decrement_balance(&self.lp_token.link.address, amount);
        self
    }
    pub fn cannot_withdraw(&mut self, staked: u128, amount: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Withdraw {
                amount: amount.into(),
            }),
            errors::withdraw(staked.into(), amount.into()),
        );
        self
    }
    pub fn claims(&mut self, reward: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Claim { to: None }),
            HandleResponse::default()
                .msg(
                    self.reward_token
                        .transfer(&self.env.message.sender, reward.into())
                        .unwrap(),
                )
                .unwrap()
                .log("reward", &reward.to_string())
                .unwrap()
                .log("recipient", &self.env.message.sender.as_str())
        );
        self.deps
            .querier
            .decrement_balance(&self.reward_token.link.address, reward);
        self
    }
    pub fn withdraws_claims(&mut self, stake: u128, reward: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Withdraw {
                amount: stake.into(),
            }),
            HandleResponse::default()
                .msg(
                    self.reward_token
                        .transfer(&self.env.message.sender, reward.into())
                        .unwrap(),
                )
                .unwrap()
                .msg(
                    self.lp_token
                        .transfer(&self.env.message.sender, stake.into())
                        .unwrap(),
                )
                .unwrap()
                .log("reward", &reward.to_string()),
        );
        self.deps
            .querier
            .decrement_balance(&self.reward_token.link.address, reward);
        self.deps
            .querier
            .decrement_balance(&self.lp_token.link.address, stake);
        self
    }
    pub fn must_wait(&mut self, remaining: Duration) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Claim { to: None }),
            errors::claim_bonding(remaining),
        )
    }
    pub fn pool_empty(&mut self) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Claim { to: None }),
            errors::claim_pool_empty(),
        )
    }
    pub fn enable_migration_to(&mut self, contract: &ContractLink<HumanAddr>) -> &mut Self {
        self.test_handle(
            Handle::Emigration(EmigrationHandle::EnableMigrationTo(contract.clone())),
            Ok(HandleResponse::default()),
        );
        self
    }
    pub fn disable_migration_to(&mut self, contract: &ContractLink<HumanAddr>) -> &mut Self {
        self.test_handle(
            Handle::Emigration(EmigrationHandle::DisableMigrationTo(contract.clone())),
            Ok(HandleResponse::default()),
        );
        self
    }
    pub fn enable_migration_from(&mut self, contract: &ContractLink<HumanAddr>) -> &mut Self {
        self.test_handle(
            Handle::Immigration(ImmigrationHandle::EnableMigrationFrom(contract.clone())),
            Ok(HandleResponse::default()),
        );
        self
    }
    pub fn disable_migration_from(&mut self, contract: &ContractLink<HumanAddr>) -> &mut Self {
        self.test_handle(
            Handle::Immigration(ImmigrationHandle::DisableMigrationFrom(contract.clone())),
            Ok(HandleResponse::default()),
        );
        self
    }
    pub fn migrate_from(
        &mut self,
        last_version: &mut Context,
        expected_stake: u128,
        expected_reward: u128,
    ) -> &mut Self {
        let migrant = self.initiator.clone();

        // 1. User calls RequestMigration on NEW.
        self.table.add_row(row!["Migration step 1", "", "", ""]);
        let export = Handle::Emigration(EmigrationHandle::ExportState(migrant.clone()));
        self.test_handle(
            Handle::Immigration(ImmigrationHandle::RequestMigration(
                last_version.link.clone(),
            )),
            HandleResponse::default().msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: last_version.link.address.clone(),
                callback_code_hash: last_version.link.code_hash.clone(),
                msg: to_binary(&export).unwrap(),
                send: vec![],
            })),
        );

        // 2. NEW calls ExportState(migrant) on OLD.
        last_version
            .table
            .add_row(row!["Migration step 2", "", "", ""]);
        let mut export_result = HandleResponse::default().msg(
            self.lp_token
                .transfer(&self.link.address.clone(), expected_stake.into())
                .unwrap(),
        );
        if expected_reward > 0 {
            export_result = export_result.unwrap().msg(
                self.reward_token
                    .transfer_from(
                        &self.env.message.sender,
                        &self.env.contract.address,
                        expected_reward.into(),
                    )
                    .unwrap(),
            );
        }
        let receive_vk_snapshot = Handle::Immigration(ImmigrationHandle::ReceiveMigration(
            to_binary(
                &((
                    migrant.clone(),
                    Auth::load_vk(&last_version.deps, &migrant)
                        .unwrap()
                        .map(|vk| vk.0),
                    expected_stake.into(),
                ) as AccountSnapshot),
            )
            .unwrap(),
        ));
        export_result = export_result
            .unwrap()
            .msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.link.address.clone(),
                callback_code_hash: self.link.code_hash.clone(),
                msg: to_binary(&receive_vk_snapshot).unwrap(),
                send: vec![],
            }));
        let mut env = last_version.env.clone();
        env.message.sender = self.link.address.clone();
        test_handle(
            &mut last_version.table,
            &mut last_version.deps,
            &env,
            export,
            export_result,
        );

        // 3. OLD calls ReceiveMigration on NEW
        self.table.add_row(row!["Migration step 3", "", "", ""]);
        let mut env = self.env.clone();
        env.message.sender = last_version.link.address.clone();
        test_handle(
            &mut self.table,
            &mut self.deps,
            &env,
            receive_vk_snapshot,
            HandleResponse::default().log("migrated", &expected_stake.to_string()),
        );

        self
    }
    pub fn emigration_fails(&mut self, last_version: &mut Context) -> &mut Self {
        assert_eq!(
            Contract::handle(
                &mut self.deps,
                self.env.clone(),
                Handle::Immigration(ImmigrationHandle::RequestMigration(
                    last_version.link.clone()
                ))
            ),
            errors::emigration_disallowed()
        );
        self
    }
    pub fn immigration_fails(&mut self, last_version: &mut Context) -> &mut Self {
        assert!(Contract::handle(
            &mut self.deps,
            self.env.clone(),
            Handle::Immigration(ImmigrationHandle::RequestMigration(
                last_version.link.clone()
            ))
        )
        .is_ok());

        assert_eq!(
            Contract::handle(
                &mut last_version.deps,
                last_version.env.clone(),
                Handle::Emigration(EmigrationHandle::ExportState(self.initiator.clone()))
            ),
            errors::immigration_disallowed()
        );

        self
    }
    pub fn staked(&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().staked.0;
        if let Ok(Response::Balance { amount }) = Contract::query(
            &self.deps,
            Query::Balance {
                address: self.initiator.clone(),
                key: String::from(""),
            },
        ) {
            assert_eq!(amount, actual.into());
        } else {
            panic!("keplr balance query returned unexpected type");
        };
        self.test_field("account.staked              ", actual, expected)
    }
    pub fn volume(&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().volume.0;
        self.test_field("account.volume              ", actual, expected.into())
    }
    pub fn bonding(&mut self, expected: Duration) -> &mut Self {
        let actual = self.account_status().bonding;
        self.test_field("account.bonding             ", actual, expected.into())
    }
    pub fn earned(&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().earned.0;
        self.test_field("account.earned              ", actual, expected)
    }
    pub fn entry(&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().starting_pool_volume.0;
        self.test_field("account.starting_pool_volume", actual, expected.into())
    }
    pub fn account_status_requires_vk(&mut self) -> &mut Self {
        assert_eq!(
            Contract::query(
                &self.deps,
                Query::Rewards(RewardsQuery::UserInfo {
                    at: self.env.block.time,
                    address: self.initiator.clone(),
                    key: String::from("invalid")
                })
            ),
            Err(StdError::unauthorized())
        );
        self
    }
    pub fn account_status(&mut self) -> Account {
        match Contract::query(
            &self.deps,
            Query::Rewards(RewardsQuery::UserInfo {
                at: self.env.block.time,
                address: self.initiator.clone(),
                key: String::from(""),
            }),
        ) {
            Ok(result) => match result {
                Response::Rewards(crate::RewardsResponse::UserInfo(account)) => account,
                _ => panic!(),
            },
            Err(e) => {
                self.table
                    .add_row(row![rbBrFd->"ERROR", bBrFd->"status", "", bBrFd->e]);
                panic!("status query failed: {:?}", e);
            }
        }
    }
    pub fn total_staked(&mut self, expected: u128) -> &mut Self {
        let actual = self.pool_status().staked.0;
        self.test_field("total.staked                ", actual, expected.into())
    }
    pub fn pool_volume(&mut self, expected: u128) -> &mut Self {
        let actual = self.pool_status().volume.0;
        self.test_field("total.volume                ", actual, expected.into())
    }
    pub fn distributed(&mut self, expected: u128) -> &mut Self {
        let actual = self.pool_status().distributed.0;
        self.test_field("total.distributed           ", actual, expected)
    }

    // GOVERNANCE

    pub fn pool_status(&mut self) -> Total {
        match Contract::query(
            &self.deps,
            Query::Rewards(RewardsQuery::PoolInfo {
                at: self.env.block.time,
            }),
        ) {
            Ok(result) => match result {
                Response::Rewards(crate::RewardsResponse::PoolInfo(total)) => total,
                _ => panic!(),
            },
            Err(e) => {
                self.table
                    .add_row(row![rbBrFd->"ERROR", bBrFd->"query(status)", "", bBrFd->e]);
                panic!("status query failed: {:?}", e);
            }
        }
    }
    fn test_field<V: std::fmt::Debug + Clone + PartialEq>(
        &mut self,
        name: &'static str,
        actual: V,
        expected: V,
    ) -> &mut Self {
        self.table.add_row(row![
             r->self.time,
             self.link.address,
             self.initiator,
             format!("{} = {:?}", &name, &actual),
        ]);
        if expected != actual {
            self.table.add_row(row![
                rbBrFd->"ERROR",
                 bBrFd->"EXPECTED",
                 "",
                 bBrFd->format!("{} = {:?}", &name, &expected),
            ]);
        }
        assert_eq!(expected, actual, "{}", name);
        self
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        println!("writing to test/{}.csv", &self.name);
        let file = std::fs::File::create(format!("test/{}.csv", &self.name)).unwrap();
        self.table.to_csv(file).unwrap();
        self.table.printstd();
    }
}

pub fn env(signer: &HumanAddr, time: u64) -> Env {
    let mut env = mock_env(signer, &[]);
    env.block.time = time;
    env
}

pub fn test_handle(
    table: &mut Table,
    deps: &mut Deps,
    env: &Env,
    msg: Handle,
    expected: StdResult<HandleResponse>,
) {
    let initiator = env.message.sender.clone();
    let own_address = env.contract.address.clone();

    let add_result = |table: &mut Table, result: &StdResult<HandleResponse>| match result {
        Ok(ref result) => {
            for message in result.messages.iter() {
                if let CosmosMsg::Wasm(WasmMsg::Execute {
                    ref msg,
                    ref contract_addr,
                    ..
                }) = message
                {
                    let ref decoded = decode_msg(msg).unwrap();
                    table.add_row(row![rb->"tx", own_address, contract_addr, decoded[4..],]);
                } else {
                    table.add_row(
                        row![r->"msg", own_address, "", serde_yaml::to_string(&message).unwrap()],
                    );
                }
            }
            for log in result.log.iter() {
                table.add_row(row![
                    rb->"log",
                    own_address,
                    "",
                    format!("{} = {}", &log.key, &log.value),
                ]);
            }
        }

        Err(ref error) => {
            table.add_row(row![r->"=>", "err", error,""]);
        }
    };

    fn decode_msg(msg: &Binary) -> Option<String> {
        let msg: serde_json::Value = serde_json::from_slice(msg.as_slice()).unwrap();
        Some(serde_yaml::to_string(&msg).unwrap())
    }

    table.add_row(row!["", "", "", ""]);

    let msg_ser = serde_yaml::to_string(&msg).unwrap();
    table.add_row(row![
        rb->env.block.time,
        &initiator,
        own_address,
        b->msg_ser.trim()[4..]
    ]);

    let result = Contract::handle(deps, env.clone(), msg);
    add_result(table, &result);

    if result != expected {
        table.add_row(row!["", "", "", ""]);
        table.add_row(row![
            rbBrFd->"FAIL",
            bBrFd->"was expecting",
            bBrFd->"the following",""
        ]);
        table.add_row(row!["", "", "", ""]);
        add_result(table, &expected);
    }

    assert_eq!(result, expected);
}

#[derive(Clone)]
pub struct RewardsMockQuerier {
    pub balances: std::collections::HashMap<HumanAddr, u128>,
}

impl RewardsMockQuerier {
    pub fn new() -> Self {
        let mut balances = std::collections::HashMap::new();
        balances.insert("SIENNA".into(), 0u128);
        balances.insert("LP_TOKEN".into(), 0u128);
        Self { balances }
    }
    fn get_balance(&self, address: &HumanAddr) -> u128 {
        *self.balances.get(address).unwrap()
    }
    pub fn increment_balance(&mut self, address: &HumanAddr, amount: u128) -> () {
        self.balances
            .insert(address.clone(), self.get_balance(address) + amount)
            .unwrap();
    }
    pub fn decrement_balance(&mut self, address: &HumanAddr, amount: u128) -> () {
        self.balances
            .insert(address.clone(), self.get_balance(address) - amount)
            .unwrap();
    }
    pub fn mock_query_dispatch(
        &self,
        contract: &ContractLink<HumanAddr>,
        msg: &Snip20Query,
    ) -> Snip20Response {
        match msg {
            Snip20Query::Balance { .. } => {
                let amount = self.get_balance(&contract.address).into();
                Snip20Response::Balance { amount }
            }
            Snip20Query::TokenInfo { .. } => {
                Snip20Response::TokenInfo {
                    name: "FooToken".into(),
                    symbol: "FOO".into(), // unused
                    decimals: 0,          // unused
                    total_supply: None,   // unused
                }
            } //_ => unimplemented!()
        }
    }
}

impl Querier for RewardsMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(_) => unimplemented!(),
        };
        match request {
            QueryRequest::Wasm(WasmQuery::Smart {
                callback_code_hash,
                contract_addr,
                msg,
            }) => Ok(to_binary(&self.mock_query_dispatch(
                &ContractLink {
                    code_hash: callback_code_hash,
                    address: contract_addr,
                },
                &from_binary(&msg).unwrap(),
            ))),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Query {
    Balance {},
    TokenInfo {},
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Response {
    Balance {
        amount: Amount,
    },
    TokenInfo {
        name: String,
        symbol: String,
        decimals: u64,
        total_supply: Option<u128>,
    },
}
