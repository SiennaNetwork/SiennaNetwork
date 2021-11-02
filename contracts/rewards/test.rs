#![allow(non_snake_case)]
use prettytable::{Table, /*Row, Cell,*/ format};
//use yansi::Paint;

use crate::*;
use fadroma::*;
use fadroma::secret_toolkit::snip20;
use fadroma::testing::*;
//use rand::Rng;

pub type Deps = Extern<MemoryStorage, MockApi, RewardsMockQuerier>;

pub struct Context (
    pub Table,
    pub Deps,    // deps
    pub String,  // reward_vk
    pub ISnip20, // reward_token
    pub ISnip20, // lp_token
);

impl Context {
    pub fn entities () -> Self {
        //color_backtrace::install();
        Self(
            Self::new_table(),
            Extern {
                storage: MemoryStorage::default(),
                api:     MockApi::new(20),
                querier: RewardsMockQuerier { balance: 0u128.into() }
            },
            "reward_vk".to_string(),
            ISnip20::attach(
                ContractLink { address: HumanAddr::from("SIENNA_addr"), code_hash: "SIENNA_hash".into() }
            ),
            ISnip20::attach(
                ContractLink { address: HumanAddr::from("LP_addr"),     code_hash: "LP_hash".into() }
            ),
        )
    }
    pub fn entities_init () -> Self {
        let mut context = Context::entities();
        let Context(ref mut table, ref mut deps, ref VK, ref SIENNA, ref LP) = context;
        admin(table, deps).at(1).init(LP, SIENNA, VK.to_string());
        context
    }
    fn new_table () -> Table {
        let mut table = Table::new();
        table.set_format(format::FormatBuilder::new()
            .column_separator('|')
            .borders('|')
            .padding(1, 1)
        .build());
        table
    }
}
impl Drop for Context {
    fn drop (&mut self) {
        self.0.printstd();
    }
}

//pub type Context = (
    //Table,
    //Deps,    // deps
    //String,  // reward_vk
    //ISnip20, // reward_token
    //ISnip20, // lp_token
//);

//pub fn run_test<T: FnOnce(Context) -> () + std::panic::UnwindSafe>(test: T) -> () {
    //let Context(table, deps, VK, SIENNA, LP) = Context::entities();
    //let result = std::panic::catch_unwind(|| {
        //test((table, deps, VK, SIENNA, LP))
    //});
    //take(&mut table, |table| { table.printstd(); table });
    //assert!(result.is_ok())
//}

pub fn take<T, F>(mut_ref: &mut T, closure: F)
  where F: FnOnce(T) -> T {
    use std::ptr;

    unsafe {
        let old_t = ptr::read(mut_ref);
        let new_t = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| closure(old_t)))
            .unwrap_or_else(|_| ::std::process::abort());
        ptr::write(mut_ref, new_t);
    }
}

pub fn env (signer: &HumanAddr, time: u64) -> Env {
    let mut env = mock_env(signer, &[]);
    env.block.time = time;
    env
}

pub fn admin<'a>(table: &'a mut Table, deps: &'a mut Deps) -> AdminRole<'a> {
    let address = HumanAddr::from("Admin");
    AdminRole  { deps, env: env(&address, 0), address, table }
}
pub struct AdminRole<'a> {
    pub table:   &'a mut Table,
    pub deps:    &'a mut Deps,
    pub address: HumanAddr,
    pub env:     Env,
}

pub fn user<'a>(table: &'a mut Table, deps: &'a mut Deps, address: &str) -> UserRole<'a> {
    let address = HumanAddr::from(address);
    UserRole   { deps, env: env(&address, 0), address, table }
}
pub struct UserRole<'a> {
    pub table:   &'a mut Table,
    pub deps:    &'a mut Deps,
    pub address: HumanAddr,
    pub env:     Env,
}

pub fn badman<'a>(table: &'a mut Table, deps: &'a mut Deps) -> BadmanRole<'a> {
    let address = HumanAddr::from("Badman");
    BadmanRole { deps, env: env(&address, 0), address, table }
}
pub struct BadmanRole<'a> {
    pub table:   &'a mut Table,
    pub deps:    &'a mut Deps,
    pub address: HumanAddr,
    pub env:     Env,
}

impl<'a> AdminRole<'a> {
    pub fn at (&mut self, t: Moment) -> &mut Self {
        self.env = env(&self.address, t);
        self
    }
    pub fn init_invalid (&mut self) -> &mut Self {
        assert!(
            Rewards::init(self.deps, &self.env, RewardsConfig {
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
    pub fn init (&mut self, lp: &ISnip20, reward: &ISnip20, vk: String) -> &mut Self {
        crate::Auth::init(self.deps, &self.env, &None).unwrap();
        assert_eq!(
            Rewards::init(self.deps, &self.env, RewardsConfig {
                lp_token:     Some(lp.link.clone()),
                reward_token: Some(reward.link.clone()),
                reward_vk:    Some(vk.clone()),
                ratio:        None,
                threshold:    None,
                cooldown:     None,
            }).unwrap(),
            Some(snip20::set_viewing_key_msg(
                vk.clone(),
                None, BLOCK_SIZE,
                reward.link.code_hash.clone(),
                reward.link.address.clone()
            ).unwrap())
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
            self.deps, &self.env, self.address.clone(),
            RewardsHandle::Configure(config),
            Ok(expected)
        );
        self
    }
    pub fn set_ratio (&mut self, ratio: (u128, u128)) -> &mut Self  {
        // TODO query!!!
        test_handle(
            &mut self.table,
            self.deps, &self.env, self.address.clone(),
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
        assert_eq!(Rewards::handle(self.deps, &self.env, RewardsHandle::Configure(RewardsConfig {
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
    pub fn closes_pool (&mut self) -> &mut Self{
        test_handle(
            &mut self.table,
            self.deps, &self.env, self.address.clone(),
            RewardsHandle::Close { message: String::from("closed") },
            Ok(HandleResponse::default())
        ); self
    }
    pub fn drains_pool (&mut self, reward_token: &ISnip20, key: &str) {
        assert!(
            Rewards::handle(self.deps, &self.env, RewardsHandle::Drain {
                snip20:    reward_token.link.clone(),
                key:       key.into(),
                recipient: None
            }).is_ok()
        );
        let vk: Option<ViewingKey> = self.deps.get(crate::algo::pool::REWARD_VK).unwrap();
        assert_eq!(vk.unwrap().0, String::from(key));
    }
}

impl<'a> UserRole<'a> {
    pub fn at (&mut self, t: Moment) -> &mut Self {
        self.env = env(&self.address, t);
        self
    }
    //pub fn later (&mut self) -> &mut Self {
        //let t: Time = rand::thread_rng().gen();
        //self.at(self.env.block.time + t)
    //}
    pub fn set_vk (&mut self, key: &str) -> &mut Self {
        let msg = crate::AuthHandle::SetViewingKey { key: key.into(), padding: None };
        assert_eq!(
            crate::Auth::handle(self.deps, self.env.clone(), msg),
            Ok(HandleResponse::default())
        );
        self
    }
    pub fn deposits (&mut self, lp_token: &ISnip20, amount: u128) -> &mut Self {
        self.test_handle(
            RewardsHandle::Lock { amount: amount.into() },
            HandleResponse::default().msg(lp_token.transfer_from(
                &self.env.message.sender,
                &self.env.contract.address,
                amount.into()
            ).unwrap())
        )
    }
    pub fn withdraws (&mut self, lp_token: &ISnip20, amount: u128) -> &mut Self {
        self.test_handle(
            RewardsHandle::Retrieve { amount: amount.into() },
            HandleResponse::default().msg(lp_token.transfer(
                &self.env.message.sender,
                amount.into()
            ).unwrap())
        )
    }
    pub fn claims (&mut self, reward_token: &ISnip20, amount: u128) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            HandleResponse::default().msg(reward_token.transfer(
                &self.env.message.sender,
                amount.into()
            ).unwrap())
        )
    }
    pub fn needs_age_threshold (&mut self, remaining: Duration) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Rewards::err_claim_threshold(self.deps, remaining, 0)
        )
    }
    pub fn needs_cooldown (&mut self, remaining: Duration) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Rewards::err_claim_cooldown(self.deps, remaining)
        )
    }
    pub fn ratio_is_zero (&mut self) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Rewards::err_claim_global_ratio_zero(self.deps)
        )
    }

    pub fn test_handle (&mut self, msg: RewardsHandle, expected: StdResult<HandleResponse>) -> &mut Self {
        test_handle(
            &mut self.table,
            self.deps, &self.env, self.address.clone(),
            msg, expected
        ); self
    }

    pub fn status (&mut self) -> User {
        match Rewards::query_status(
            &*self.deps, self.env.block.time, Some(self.address.clone()), Some(String::from(""))
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

impl<'a> BadmanRole<'a> {
    pub fn at (&mut self, t: Moment) -> &mut Self {
        self.env = env(&self.address, t);
        self
    }
    pub fn cannot_configure (&mut self) -> &mut Self {
        assert_eq!(Rewards::handle(self.deps, &self.env, RewardsHandle::Configure(RewardsConfig {
            lp_token:     None,
            reward_token: None,
            reward_vk:    None,
            ratio:        None,
            threshold:    None,
            cooldown:     None,
        })), Err(StdError::unauthorized()));
        self
    }
    pub fn cannot_close_pool (&mut self) {
        assert_eq!(
            Rewards::handle(self.deps, &self.env, RewardsHandle::Close {
                message: String::from("closed")
            }),
            Err(StdError::unauthorized())
        );
    }
    pub fn cannot_drain (&mut self, reward_token: &ISnip20, key: &str) {
        assert!(
            Rewards::handle(self.deps, &self.env, RewardsHandle::Drain {
                snip20:    reward_token.link.clone(),
                key:       key.into(),
                recipient: None
            }).is_err()
        );
    }
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
    table.add_row(row![b->env.block.time, b->&address, b->msg_ser.trim()]);
    let result = Rewards::handle(deps, env, msg);
    let add_result = |table: &mut Table, result: &StdResult<HandleResponse>| match result {
        Ok(ref result) => {
            for message in result.messages.iter() {
                if let CosmosMsg::Wasm(WasmMsg::Execute { .. }) = message {
                    table.add_row(row!["=>", "execute", &decode_msg(message).unwrap()]);
                } else {
                    table.add_row(row!["=>", "msg", serde_yaml::to_string(&message).unwrap()]);
                }
            }
        },
        Err(ref error) => {
            table.add_row(row!["=>", "err", error]);
        }
    };
    add_result(table, &result);
    if result != expected {
        table.add_row(row![]);
        table.add_row(row!["ERROR", "was expecting", "the following:"]);
        add_result(table, &expected);
    }
    fn decode_msg (message: &CosmosMsg) -> Option<String> {
        match message {
            CosmosMsg::Wasm(WasmMsg::Execute { ref msg, .. }) => {
                let msg: serde_json::Value = serde_json::from_slice(msg.as_slice()).unwrap();
                Some(serde_yaml::to_string(&msg).unwrap())
            },
            _ => None
        }
    }
    assert_eq!(result, expected);

}

pub struct RewardsMockQuerier { pub balance: Amount }

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20Query { Balance {} }

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20Response { Balance { amount: Amount } }

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

impl RewardsMockQuerier {
    pub fn mock_query_dispatch(
        &self, _: &ContractLink<HumanAddr>, msg: &Snip20Query
    ) -> Snip20Response {
        match msg {
            Snip20Query::Balance { .. } => Snip20Response::Balance { amount: self.balance },
            //_ => unimplemented!()
        }
    }
    pub fn increment_balance (&mut self, amount: u128) -> () {
        self.balance += amount.into();
    }
    //pub fn decrement_balance (&mut self, amount: u128) -> StdResult<()> {
        //self.balance = (self.balance - amount.into())?;
        //Ok(())
    //}
}
