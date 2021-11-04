#![allow(non_snake_case)]
use prettytable::{Table, /*Row, Cell,*/ format};
//use yansi::Paint;

use crate::*;
use fadroma::*;
use fadroma::secret_toolkit::snip20;
use fadroma::testing::*;
use rand::Rng;

pub type Deps = Extern<MemoryStorage, MockApi, RewardsMockQuerier>;

pub struct Context {
    pub table:        Table,
    pub deps:         Deps,
    pub reward_vk:    String,
    pub reward_token: ISnip20,
    pub lp_token:     ISnip20,
    pub address:      HumanAddr,
    pub env:          Env,
    pub time:         Moment
}

impl Context {
    pub fn new () -> Self {
        let mut table = Table::new();
        table
            .set_format(format::FormatBuilder::new()
                .column_separator('|')
                .borders('|')
                .padding(1, 1)
            .build());

        let address = HumanAddr::from("Admin");
        let time = 1;

        //color_backtrace::install();

        Self {
            table,

            deps: Extern {
                storage: MemoryStorage::default(),
                api:     MockApi::new(20),
                querier: RewardsMockQuerier::new()
            },

            reward_vk: "reward_vk".to_string(),
            reward_token: ISnip20::attach(
                ContractLink { address: HumanAddr::from("SIENNA_addr"), code_hash: "SIENNA_hash".into() }
            ),

            lp_token: ISnip20::attach(
                ContractLink { address: HumanAddr::from("LP_addr"),     code_hash: "LP_hash".into() }
            ),

            env: env(&address, time),
            address,
            time,
        }
    }
    fn update_env (&mut self) -> &mut Self {
        self.env = env(&self.address, self.time);
        self
    }
    pub fn at (&mut self, t: Moment) -> &mut Self {
        self.time = t;
        self.update_env()
    }
    pub fn after (&mut self, t: Duration) -> &mut Self {
        self.at(self.env.block.time + t)
    }
    pub fn next (&mut self) -> &mut Self {
        self.after(1)
    }
    pub fn later (&mut self) -> &mut Self {
        self.after(rand::thread_rng().gen())
    }
    pub fn epoch (&mut self) -> &mut Self {
        self.after(86400)
    }
    pub fn set_address (&mut self, address: &str) -> &mut Self {
        self.address = HumanAddr::from(address);
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
        self.deps.querier.increment_balance(&self.reward_token.link.address, amount);
        self
    }
    pub fn test_handle (&mut self, msg: RewardsHandle, expected: StdResult<HandleResponse>) -> &mut Self {
        test_handle(
            &mut self.table,
            &mut self.deps,
            &self.env,
            self.address.clone(),
            msg,
            expected
        ); self
    }
    pub fn init (&mut self) -> &mut Self {
        crate::Auth::init(&mut self.deps, &self.env, &None).unwrap();
        assert_eq!(
            Rewards::init(&mut self.deps, &self.env, RewardsConfig {
                lp_token:     Some(self.lp_token.link.clone()),
                reward_token: Some(self.reward_token.link.clone()),
                reward_vk:    Some(self.reward_vk.clone()),
                ratio:        None,
                threshold:    None,
                cooldown:     None,
            }).unwrap(),
            Some(snip20::set_viewing_key_msg(
                self.reward_vk.clone(),
                None, BLOCK_SIZE,
                self.reward_token.link.code_hash.clone(),
                self.reward_token.link.address.clone()
            ).unwrap())
        );
        self
    }
    pub fn init_invalid (&mut self) -> &mut Self {
        assert!(
            Rewards::init(&mut self.deps, &self.env, RewardsConfig {
                lp_token:     None,
                reward_token: None,
                reward_vk:    None,
                ratio:        None,
                threshold:    None,
                cooldown:     None,
            }).is_err(),
        );
        self
    }
    pub fn configure (&mut self, config: RewardsConfig) -> &mut Self {
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
            &mut self.deps, &self.env, self.address.clone(),
            RewardsHandle::Configure(config),
            Ok(expected)
        );
        self
    }
    pub fn cannot_configure (&mut self) -> &mut Self {
        assert_eq!(Rewards::handle(&mut self.deps, &self.env, RewardsHandle::Configure(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        })), Err(StdError::unauthorized()));
        self
    }
    pub fn set_ratio (&mut self, ratio: (u128, u128)) -> &mut Self  {
        // TODO query!!!
        test_handle(
            &mut self.table,
            &mut self.deps, &self.env, self.address.clone(),
            RewardsHandle::Configure(RewardsConfig {
                lp_token:     None,
                reward_token: None,
                reward_vk:    None,
                ratio:        Some((ratio.0.into(), ratio.1.into())),
                threshold:    None,
                cooldown:     None,
            }),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn set_threshold (&mut self, threshold: Duration) -> &mut Self  {
        assert_eq!(Rewards::handle(&mut self.deps, &self.env, RewardsHandle::Configure(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            ratio:        None,
            threshold:    Some(threshold),
            cooldown:     None,
        })), Ok(HandleResponse::default()));
        // TODO query!!!
        self
    }
    pub fn closes_pool (&mut self) -> &mut Self {
        test_handle(
            &mut self.table,
            &mut self.deps, &self.env, self.address.clone(),
            RewardsHandle::Close { message: String::from("closed") },
            Ok(HandleResponse::default())
        ); self
    }
    pub fn cannot_close_pool (&mut self) -> &mut Self {
        assert_eq!(
            Rewards::handle(&mut self.deps, &self.env, RewardsHandle::Close {
                message: String::from("closed")
            }),
            Err(StdError::unauthorized())
        ); self
    }
    pub fn drains_pool (&mut self, key: &str) -> &mut Self {
        assert!(
            Rewards::handle(&mut self.deps, &self.env, RewardsHandle::Drain {
                snip20:    self.reward_token.link.clone(),
                key:       key.into(),
                recipient: None
            }).is_ok()
        );
        let vk: Option<ViewingKey> = self.deps.get(crate::algo::pool::REWARD_VK).unwrap();
        assert_eq!(vk.unwrap().0, String::from(key));
        self
    }
    pub fn cannot_drain (&mut self, key: &str) -> &mut Self {
        assert!(
            Rewards::handle(&mut self.deps, &self.env, RewardsHandle::Drain {
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
        dbg!(&self.lp_token);
        dbg!(&self.reward_token);
        self.test_handle(
            RewardsHandle::Lock { amount: amount.into() },
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
            RewardsHandle::Retrieve { amount: amount.into() },
            HandleResponse::default().msg(self.lp_token.transfer(
                &self.env.message.sender,
                amount.into()
            ).unwrap())
        );
        self.deps.querier.decrement_balance(&self.lp_token.link.address, amount);
        self
    }
    pub fn claims (&mut self, amount: u128) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            HandleResponse::default().msg(self.reward_token.transfer(
                &self.env.message.sender,
                amount.into()
            ).unwrap())
        );
        self.deps.querier.decrement_balance(&self.reward_token.link.address, amount);
        self
    }
    pub fn needs_age_threshold (&mut self, remaining: Duration) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Rewards::err_claim_threshold(&self.deps, remaining, 0)
        )
    }
    pub fn needs_cooldown (&mut self, remaining: Duration) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Rewards::err_claim_cooldown(&self.deps, remaining)
        )
    }
    pub fn ratio_is_zero (&mut self) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Rewards::err_claim_global_ratio_zero(&self.deps)
        )
    }
    pub fn status (&mut self) -> User {
        match Rewards::query_status(
            &self.deps, self.env.block.time, Some(self.address.clone()), Some(String::from(""))
        ).unwrap() {
            crate::RewardsResponse::Status { user, .. } => user.unwrap(),
            //_ => panic!()
        }
    }
    pub fn locked <A: Into<Amount>> (&mut self, v: A) -> &mut Self {
        assert_eq!(self.status().locked, v.into(), "user.locked");
        self
    }
    pub fn lifetime <V: Into<Volume>> (&mut self, v: V) -> &mut Self {
        assert_eq!(self.status().lifetime, v.into(), "user.lifetime");
        self
    }
    pub fn liquid (&mut self, t: u64) -> &mut Self {
        assert_eq!(self.status().liquid, t, "user.liquid");
        self
    }
    pub fn existed (&mut self, t: u64) -> &mut Self {
        assert_eq!(self.status().existed, t, "user.existed");
        self
    }
    pub fn claimed <A: Into<Amount>> (&mut self, a: A) -> &mut Self {
        assert_eq!(self.status().claimed, a.into(), "user.claimed");
        self
    }
    pub fn claimable <A: Into<Amount>> (&mut self, a: A) -> &mut Self {
        assert_eq!(self.status().claimable, a.into(), "user.claimable");
        self
    }
}
impl Drop for Context {
    fn drop (&mut self) {
        self.table.printstd();
    }
}

pub fn env (signer: &HumanAddr, time: u64) -> Env {
    let mut env = mock_env(signer, &[]);
    env.block.time = time;
    env
}

/*.msg(snip20::set_viewing_key_msg( // this is for own reward vk, not user status vk
                key.to_string(),
                None, BLOCK_SIZE,
                lp_token.link.code_hash.clone(),
                lp_token.link.address.clone()
            ).unwrap())*/

// Helpers will be indented 1 level above the test cases

pub fn test_handle (
    table:    &mut Table,
    deps:     &mut Deps,
    env:      &Env,
    address:  HumanAddr,
    msg:      RewardsHandle,
    expected: StdResult<HandleResponse>
) {
    table.add_row(row![]);
    let msg_ser = serde_yaml::to_string(&msg).unwrap();
    table.add_row(row![rb->env.block.time, b->&address, b->msg_ser.trim()[4..]]);
    let result = Rewards::handle(deps, env, msg);
    let add_result = |table: &mut Table, result: &StdResult<HandleResponse>| match result {
        Ok(ref result) => {
            for message in result.messages.iter() {
                if let CosmosMsg::Wasm(WasmMsg::Execute {
                    ref msg, ref contract_addr, ..
                }) = message {
                    let ref decoded = decode_msg(msg).unwrap();
                    table.add_row(row![
                        r->"=>",
                        b->format!("execute\n{}", contract_addr),
                        decoded[4..]
                    ]);
                } else {
                    table.add_row(row![r->"=>", "msg", serde_yaml::to_string(&message).unwrap()]);
                }
            }
        },
        Err(ref error) => {
            table.add_row(row![r->"=>", "err", error]);
        }
    };
    add_result(table, &result);
    if result != expected {
        table.add_row(row![]);
        table.add_row(row![rbBrFd->"FAIL", bBrFd->"was expecting", bBrFd->"the following:"]);
        table.add_row(row![]);
        add_result(table, &expected);
    }
    fn decode_msg (msg: &Binary) -> Option<String> {
        let msg: serde_json::Value = serde_json::from_slice(msg.as_slice()).unwrap();
        Some(serde_yaml::to_string(&msg).unwrap())
    }
    assert_eq!(result, expected);

}

pub struct RewardsMockQuerier {
    pub balances: std::collections::HashMap<HumanAddr, u128>
}

impl RewardsMockQuerier {
    pub fn new () -> Self {
        let mut balances = std::collections::HashMap::new();
        balances.insert("SIENNA_addr".into(), 0u128);
        balances.insert("LP_addr".into(),     0u128);
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
        dbg!(contract);
        dbg!(msg);
        match msg {
            Snip20Query::Balance { .. } => {
                let amount = self.get_balance(&contract.address).into();
                dbg!(amount);
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
