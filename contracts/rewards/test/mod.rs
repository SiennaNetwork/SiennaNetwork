// Look Ma, no macros! ////////////////////////////////////////////////////////////////////////////
#![cfg(test)]
#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(unreachable_patterns)]
#![allow(non_snake_case)]

mod test_0000_setup;
mod test_0100_operate;
mod test_0200_auth;
mod test_0300_migrate;

use prettytable::{Table, /*Row, Cell,*/ format};
//use yansi::Paint;

use crate::*;
use fadroma::secret_toolkit::snip20;
use fadroma::testing::*;

pub use rand::Rng;
use rand::{SeedableRng, rngs::StdRng};

compose!(MockExtern<S, A, Q>);
pub type Deps = MockExtern<ClonableMemoryStorage, MockApi, RewardsMockQuerier>;

#[derive(Clone)]
pub struct Context {
    pub rng:          StdRng,
    pub name:         String,
    pub link:         ContractLink<HumanAddr>,
    pub table:        Table,
    pub deps:         Deps,
    pub initiator:  HumanAddr,
    pub env:          Env,
    pub time:         Moment,

    pub reward_vk:    String,
    pub reward_token: ISnip20,
    pub lp_token:     ISnip20,
    pub closed:       Option<CloseSeal>,
    pub bonding:      u64
}

impl Context {
    pub fn new (name: &str) -> Self {
        let mut table = Table::new();

        table.set_format(format::FormatBuilder::new()
            .separator(format::LinePosition::Title, format::LineSeparator::new('-', '-', '-', '-'))
            .column_separator('|')
            .borders('|')
            .padding(1, 1)
        .build());

        table.set_titles(row![rb->"Time", b->"Sender", b->"Recipient", b->"Data"]);

        let initiator = HumanAddr::from("Admin");
        let time = 1;

        //color_backtrace::install();

        let mut rng = StdRng::seed_from_u64(1);
        let bonding = rng.gen_range(100..200);
        Self {
            rng,
            name: name.to_string(),
            link: ContractLink {
                address:   HumanAddr::from(format!("{}_addr", &name)),
                code_hash: format!("{}_hash", &name).to_string(),
            },
            table,

            deps: MockExtern {
                storage: ClonableMemoryStorage::default(),
                api:     MockApi::new(20),
                querier: RewardsMockQuerier::new()
            },

            reward_vk: "reward_vk".to_string(),
            reward_token: ISnip20::attach(
                ContractLink { address: HumanAddr::from("SIENNA"),   code_hash: "SIENNA_hash".into() }
            ),

            lp_token: ISnip20::attach(
                ContractLink { address: HumanAddr::from("LP_TOKEN"), code_hash: "LP_hash".into() }
            ),

            env: env(&initiator, time),
            initiator,
            time,
            closed: None,
            bonding
        }
    }
    pub fn branch <F: FnMut(Context)->()> (&mut self, name: &str, mut f: F) -> &mut Self {
        let mut context = self.clone();
        let name = format!("{}_{}", self.name, name).to_string();
        context.name = name.to_string();
        context.table.add_row(row!["","","",""]);
        context.table.add_row(row![rb->self.time, "test", "branch", b->&name]);
        context.table.add_row(row!["","","",""]);
        f(context);
        self
    }
    fn update_env (&mut self) -> &mut Self {
        self.env = env(&self.initiator, self.time);
        self
    }
    pub fn at (&mut self, t: Moment) -> &mut Self {
        self.time = t;
        self.update_env()
    }
    pub fn after (&mut self, t: Duration) -> &mut Self {
        self.at(self.env.block.time + t)
    }
    pub fn tick (&mut self) -> &mut Self {
        self.after(1)
    }
    pub fn during <F: FnMut(u64, &mut Context)->()> (&mut self, n: Duration, mut f: F) -> &mut Self {
        for i in 1..=n {
            self.tick();
            f(i, self);
        }
        self
    }
    pub fn later (&mut self) -> &mut Self {
        let t = self.rng.gen_range(0..self.bonding/10);
        self.after(t)
    }
    pub fn epoch (&mut self, next_epoch: Moment, portion: u128) -> &mut Self {
        self.after(self.bonding);
        self.fund(portion);
        self.test_handle(
            Handle::Rewards(RewardsHandle::BeginEpoch { next_epoch }),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn set_address (&mut self, address: &str) -> &mut Self {
        self.initiator = HumanAddr::from(address);
        self.update_env()
    }
    pub fn admin (&mut self) -> &mut Self {
        self.set_address("Admin")
    }
    pub fn badman (&mut self) -> &mut Self {
        self.set_address("Badman")
    }
    pub fn user (&mut self, address: &str) -> &mut Self {
        self.set_address(address)
    }
    pub fn fund (&mut self, amount: u128) -> &mut Self {
        self.table.add_row(row!["","","",""]);
        self.table.add_row(row![
            rb->self.time,
            "RPT",
            self.link.address.clone(),
            b->format!("vest {}", &amount)
        ]);
        self.deps.querier.increment_balance(&self.reward_token.link.address, amount);
        self
    }
    pub fn test_handle (&mut self, msg: Handle, expected: StdResult<HandleResponse>)
        -> &mut Self
    {
        test_handle(
            &mut self.table, &mut self.deps, &self.env,
            self.initiator.clone(), msg, expected, self.link.address.clone()
        );
        self
    }
    pub fn init (&mut self) -> &mut Self {
        crate::Auth::init(&mut self.deps, &self.env, &None).unwrap();
        let config = RewardsConfig {
            lp_token:     Some(self.lp_token.link.clone()),
            reward_token: Some(self.reward_token.link.clone()),
            reward_vk:    Some(self.reward_vk.clone()),
            bonding:      Some(self.bonding),
            timekeeper:   Some(HumanAddr::from("Admin")),
        };
        let actual = Rewards::init(&mut self.deps, &self.env, config).unwrap();
        let expected = vec![
            snip20::set_viewing_key_msg(
                self.reward_vk.clone(),
                None, BLOCK_SIZE,
                self.reward_token.link.code_hash.clone(),
                self.reward_token.link.address.clone()
            ).unwrap()
        ];
        assert_eq!(actual, expected);
        self
    }
    pub fn init_invalid (&mut self) -> &mut Self {
        let invalid_config = RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            bonding:      None,
            timekeeper:   None
        };
        assert!(Rewards::init(&mut self.deps, &self.env, invalid_config).is_err());
        self
    }
    pub fn configures (&mut self, config: RewardsConfig) -> &mut Self {
        let mut expected = HandleResponse::default();
        if config.reward_vk.is_some() && config.reward_token.is_some() {
            expected.messages.push(snip20::set_viewing_key_msg(
                config.reward_vk.clone().unwrap(),
                None, BLOCK_SIZE,
                config.reward_token.clone().unwrap().code_hash,
                config.reward_token.clone().unwrap().address
            ).unwrap())
        }
        test_handle(
            &mut self.table,
            &mut self.deps, &self.env, self.initiator.clone(),
            Handle::Rewards(RewardsHandle::Configure(config)),
            Ok(expected),
            self.link.address.clone()
        );
        self
    }
    pub fn sets_bonding (&mut self, bonding: Duration) -> &mut Self {
        self.configures(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            bonding:      Some(bonding),
            timekeeper:   None,
        })
    }
    pub fn cannot_configure (&mut self) -> &mut Self {
        assert_eq!(Rewards::handle(&mut self.deps, self.env.clone(), RewardsHandle::Configure(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            bonding:      None,
            timekeeper:   None,
        })), Err(StdError::unauthorized()));
        self
    }
    pub fn closes_pool (&mut self) -> &mut Self {
        let message = "closed";
        test_handle(
            &mut self.table,
            &mut self.deps, &self.env, self.initiator.clone(),
            Handle::Rewards(RewardsHandle::Close { message: message.to_string() }),
            Ok(HandleResponse::default()),
            self.link.address.clone()
        );
        self.closed = Some((self.time, message.to_string()));
        self
    }
    pub fn cannot_close_pool (&mut self) -> &mut Self {
        test_handle(
            &mut self.table,
            &mut self.deps, &self.env, self.initiator.clone(),
            Handle::Rewards(RewardsHandle::Close { message: String::from("closed") }),
            Err(StdError::unauthorized()),
            self.link.address.clone()
        ); self
    }
    pub fn drains_pool (&mut self, key: &str) -> &mut Self {
        assert!(
            Contract::handle(&mut self.deps, self.env.clone(), Handle::Drain {
                snip20:    self.reward_token.link.clone(),
                key:       key.into(),
                recipient: None
            }).is_ok()
        );
        let vk: Option<ViewingKey> = self.deps.get(crate::algo::RewardsConfig::REWARD_VK).unwrap();
        assert_eq!(vk.unwrap().0, String::from(key));
        self
    }
    pub fn cannot_drain (&mut self, key: &str) -> &mut Self {
        assert!(
            Contract::handle(&mut self.deps, self.env.clone(), Handle::Drain {
                snip20:    self.reward_token.link.clone(),
                key:       key.into(),
                recipient: None
            }).is_err()
        ); self
    }
    pub fn set_vk (&mut self, key: &str) -> &mut Self {
        let msg = crate::AuthHandle::SetViewingKey { key: key.into(), padding: None };
        assert_eq!(
            crate::Auth::handle(&mut self.deps, self.env.clone(), msg),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn deposits (&mut self, amount: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Deposit { amount: amount.into() }),
            HandleResponse::default().msg(self.lp_token.transfer_from(
                &self.env.message.sender,
                &self.env.contract.address,
                amount.into()
            ).unwrap())
        );
        self.deps.querier.increment_balance(&self.lp_token.link.address, amount);
        self
    }
    pub fn withdraws (&mut self, amount: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Withdraw { amount: amount.into() }),
            HandleResponse::default().msg(self.lp_token.transfer(
                &self.env.message.sender,
                amount.into()
            ).unwrap())
        );
        self.deps.querier.decrement_balance(&self.lp_token.link.address, amount);
        self
    }
    pub fn claims (&mut self, reward: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Claim {}),
            HandleResponse::default().msg(
                self.reward_token.transfer(
                    &self.env.message.sender,
                    reward.into()
                ).unwrap()
            ).unwrap().log(
                "reward", &reward.to_string()
            )
        );
        self.deps.querier.decrement_balance(&self.reward_token.link.address, reward);
        self
    }
    pub fn withdraws_claims (&mut self, stake: u128, reward: u128) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Withdraw { amount: stake.into() }),
            HandleResponse::default()
                .msg(
                    self.reward_token.transfer(&self.env.message.sender, reward.into()).unwrap()
                ).unwrap()
                .msg(
                    self.lp_token.transfer(&self.env.message.sender, stake.into()).unwrap()
                ).unwrap()
                .log(
                    "reward", &reward.to_string()
                )
        );
        self.deps.querier.decrement_balance(&self.reward_token.link.address, reward);
        self.deps.querier.decrement_balance(&self.lp_token.link.address,     stake);
        self
    }
    pub fn must_wait (&mut self, remaining: Duration) -> &mut Self {
        self.test_handle(
            Handle::Rewards(RewardsHandle::Claim {}),
            errors::claim_bonding(remaining)
        )
    }
    pub fn enable_migration_to (&mut self, contract: &ContractLink<HumanAddr>) -> &mut Self {
        self.test_handle(
            Handle::MigrationExport(MigrationExportHandle::EnableMigrationTo(contract.clone())),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn enable_migration_from (&mut self, contract: &ContractLink<HumanAddr>) -> &mut Self {
        self.test_handle(
            Handle::MigrationImport(MigrationImportHandle::EnableMigrationFrom(contract.clone())),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn migrate_from (
        &mut self,
        last_version: &mut Context,
        migrated_stake: u128,
        claimed_reward: u128
    ) -> &mut Self {

        let request = MigrationImportHandle::RequestMigration(last_version.link.clone());
        let export  = MigrationExportHandle::ExportState(self.initiator.clone());

        self.test_handle(
            Handle::MigrationImport(request),
            HandleResponse::default().msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      last_version.link.address.clone(),
                callback_code_hash: last_version.link.code_hash.clone(),
                msg:  to_binary(&export).unwrap(),
                send: vec![], })) );

        let mut export_result = HandleResponse::default().msg(self.lp_token.transfer(
            &self.env.message.sender,
            migrated_stake.into() ).unwrap());

        if claimed_reward > 0 {
            export_result = export_result.unwrap().msg(self.reward_token.transfer_from(
                &self.env.message.sender,
                &self.env.contract.address,
                claimed_reward.into()).unwrap()); }

        let id = self.deps.canonize(self.initiator.clone()).unwrap();
        let vk_snapshot = (
            self.initiator.clone(), 
            if let Some(vk) = Auth::load_vk(&last_version.deps, id.as_slice()).unwrap() {
                Some(vk.0)
            } else {
                None
            },
            self.deps.get_ns(Account::STAKED, id.as_slice()).unwrap().unwrap_or(Amount::zero())
        );
        let receive_vk_snapshot = MigrationImportHandle::ReceiveMigration(
            to_binary(&vk_snapshot).unwrap());

        export_result = export_result.unwrap().msg(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr:      self.link.address.clone(),
            callback_code_hash: self.link.code_hash.clone(),
            msg:  to_binary(&receive_vk_snapshot).unwrap(),
            send: vec![], }));

        test_handle(
            &mut last_version.table,
            &mut last_version.deps,
            &env(&self.link.address, self.time),
            self.link.address.clone(),
            Handle::MigrationExport(export),
            export_result,
            self.link.address.clone());

        self.test_handle(
            Handle::MigrationImport(receive_vk_snapshot),
            HandleResponse::default()
                .msg(self.lp_token.transfer_from(
                    &self.initiator,
                    &self.link.address,
                    migrated_stake.into()
                ).unwrap()));

        self
    }
    pub fn staked (&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().staked.0;
        self.test_field("account.staked              ", actual, expected)
    }
    pub fn volume (&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().volume.0;
        self.test_field("account.volume              ", actual, expected.into())
    }
    pub fn bonding (&mut self, expected: Duration) -> &mut Self {
        let actual = self.account_status().bonding;
        self.test_field("account.bonding             ", actual, expected.into())
    }
    pub fn earned (&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().earned.0;
        self.test_field("account.earned              ", actual, expected)
    }
    pub fn entry (&mut self, expected: u128) -> &mut Self {
        let actual = self.account_status().starting_pool_volume.0;
        self.test_field("account.starting_pool_volume", actual, expected.into())
    }
    pub fn account_status (&mut self) -> Account {
        let at      = self.env.block.time;
        let address = self.initiator.clone();
        let key     = String::from("");
        let result = Rewards::query(&self.deps, RewardsQuery::UserInfo { at, address, key });
        match result {
            Ok(result) => {
                match result {
                    crate::RewardsResponse::UserInfo(account) => account,
                    _ => panic!()
                }
            },
            Err(e) => {
                self.table.add_row(row![rbBrFd->"ERROR", bBrFd->"status", "", bBrFd->e]);
                panic!("status query failed: {:?}", e);
            }
        }
    }
    pub fn total_staked (&mut self, expected: u128) -> &mut Self {
        let actual = self.pool_status().staked.0;
        self.test_field("total.staked                ", actual, expected.into())
    }
    pub fn pool_volume (&mut self, expected: u128) -> &mut Self {
        let actual = self.pool_status().volume.0;
        self.test_field("total.volume                ", actual, expected.into())
    }
    pub fn distributed (&mut self, expected: u128) -> &mut Self {
        let actual = self.pool_status().distributed.0;
        self.test_field("total.distributed           ", actual, expected)
    }
    pub fn pool_status (&mut self) -> Total {
        let result = Rewards::query(&self.deps, RewardsQuery::PoolInfo { at: self.env.block.time });
        match result {
            Ok(result) => {
                match result {
                    crate::RewardsResponse::PoolInfo(total) => total,
                    _ => panic!()
                }
            },
            Err(e) => {
                self.table.add_row(row![rbBrFd->"ERROR", bBrFd->"query(status)", "", bBrFd->e]);
                panic!("status query failed: {:?}", e);
            }
        }
    }
    fn test_field <V: std::fmt::Debug + Clone + PartialEq> (&mut self, name: &'static str, actual: V, expected: V) -> &mut Self {
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
    fn drop (&mut self) {
        println!("writing to test/{}.csv", &self.name);
        let file = std::fs::File::create(format!("test/{}.csv", &self.name)).unwrap();
        self.table.to_csv(file).unwrap();
        self.table.printstd();
    }
}

pub fn env (signer: &HumanAddr, time: u64) -> Env {
    let mut env = mock_env(signer, &[]);
    env.block.time = time;
    env
}

pub fn test_handle (
    table:       &mut Table,
    deps:        &mut Deps,
    env:         &Env,
    initiator:   HumanAddr,
    msg:         Handle,
    expected:    StdResult<HandleResponse>,
    own_address: HumanAddr
) {
    table.add_row(row!["","","",""]);
    let msg_ser = serde_yaml::to_string(&msg).unwrap();
    table.add_row(row![rb->env.block.time, &initiator, own_address, b->msg_ser.trim()[4..]]);
    let result = Contract::handle(deps, env.clone(), msg);
    let add_result = |table: &mut Table, result: &StdResult<HandleResponse>| match result {
        Ok(ref result) => {
            for message in result.messages.iter() {
                if let CosmosMsg::Wasm(WasmMsg::Execute {
                    ref msg, ref contract_addr, ..
                }) = message {
                    let ref decoded = decode_msg(msg).unwrap();
                    table.add_row(row![rb->"tx", own_address, contract_addr, decoded[4..],]);
                } else {
                    table.add_row(row![r->"msg", own_address, "", serde_yaml::to_string(&message).unwrap()]);
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
        },
        Err(ref error) => {
            table.add_row(row![r->"=>", "err", error,""]);
        }
    };
    add_result(table, &result);
    if result != expected {
        table.add_row(row!["","","",""]);
        table.add_row(row![rbBrFd->"FAIL", bBrFd->"was expecting", bBrFd->"the following",""]);
        table.add_row(row!["","","",""]);
        add_result(table, &expected);
    }
    fn decode_msg (msg: &Binary) -> Option<String> {
        let msg: serde_json::Value = serde_json::from_slice(msg.as_slice()).unwrap();
        Some(serde_yaml::to_string(&msg).unwrap())
    }
    assert_eq!(result, expected);

}

#[derive(Clone)]
pub struct RewardsMockQuerier {
    pub balances: std::collections::HashMap<HumanAddr, u128>
}

impl RewardsMockQuerier {
    pub fn new () -> Self {
        let mut balances = std::collections::HashMap::new();
        balances.insert("SIENNA".into(), 0u128);
        balances.insert("LP_TOKEN".into(),     0u128);
        Self { balances }
    }
    fn get_balance (&self, address: &HumanAddr) -> u128 {
        *self.balances.get(address).unwrap()
    }
    pub fn increment_balance (&mut self, address: &HumanAddr, amount: u128) -> () {
        self.balances.insert(address.clone(), self.get_balance(address) + amount).unwrap();
    }
    pub fn decrement_balance (&mut self, address: &HumanAddr, amount: u128) -> () {
        self.balances.insert(address.clone(), self.get_balance(address) - amount).unwrap();
    }
    pub fn mock_query_dispatch(
        &self, contract: &ContractLink<HumanAddr>, msg: &Snip20Query
    ) -> Snip20Response {
        match msg {
            Snip20Query::Balance { .. } => {
                let amount = self.get_balance(&contract.address).into();
                Snip20Response::Balance { amount }
            }
            //_ => unimplemented!()
        }
    }
}

impl Querier for RewardsMockQuerier {
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(_) => unimplemented!()
        };
        match request {
            QueryRequest::Wasm(WasmQuery::Smart { callback_code_hash, contract_addr, msg }) => {
                Ok(to_binary(&self.mock_query_dispatch(&ContractLink {
                    code_hash: callback_code_hash,
                    address:   contract_addr
                }, &from_binary(&msg).unwrap())))
            },
            _ => unimplemented!()
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20Query { Balance {} }

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20Response { Balance { amount: Amount } }

#[derive(Default, Clone)]
pub struct ClonableMemoryStorage {
    data: std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
}
impl ClonableMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}
impl ReadonlyStorage for ClonableMemoryStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }
}
impl Storage for ClonableMemoryStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.data.insert(key.to_vec(), value.to_vec());
    }
    fn remove(&mut self, key: &[u8]) {
        self.data.remove(key);
    }
}
