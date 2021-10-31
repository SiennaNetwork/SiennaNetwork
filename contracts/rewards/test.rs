use crate::*;
use fadroma::*;
use fadroma::secret_toolkit::snip20;
use fadroma::testing::*;
use rand::Rng;

pub type Deps = Extern<MemoryStorage, MockApi, RewardsMockQuerier>;

pub type Context = (
    Deps,    // deps
    String,  // reward_vk
    ISnip20, // reward_token
    ISnip20, // lp_token
);

pub fn entities () -> Context {
    color_backtrace::install();
    (
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

pub fn entities_init () -> Context {
    let (mut deps, VK, SIENNA, LP) = entities();
    admin(&mut deps).at(1).init(&LP, &SIENNA, VK.to_string());
    (deps, VK, SIENNA, LP)
}

pub fn env (signer: &HumanAddr, time: u64) -> Env {
    let mut env = mock_env(signer, &[]);
    env.block.time = time;
    env
}

pub struct AdminRole<'a> { pub deps: &'a mut Deps, pub address: HumanAddr, pub env: Env }
pub struct UserRole<'a>   { pub deps: &'a mut Deps, pub address: HumanAddr, pub env: Env }
pub struct BadmanRole<'a> { pub deps: &'a mut Deps, pub address: HumanAddr, pub env: Env }
pub fn admin<'a>(deps: &'a mut Deps) -> AdminRole<'a> {
    let address = HumanAddr::from("Admin");
    AdminRole { deps, env: env(&address, 0), address }
}
pub fn user<'a>(deps: &'a mut Deps, address: &str) -> UserRole<'a> {
    let address = HumanAddr::from(address);
    UserRole { deps, env: env(&address, 0), address }
}
pub fn badman<'a>(deps: &'a mut Deps) -> BadmanRole<'a> {
    let address = HumanAddr::from("Badman");
    BadmanRole { deps, env: env(&address, 0), address }
}

impl<'a> AdminRole<'a> {
    pub fn at (&mut self, t: Time) -> &mut Self {
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
        crate::Auth::init(self.deps, &self.env, &None);
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
            self.deps, &self.env, self.address.clone(),
            RewardsHandle::Configure(config),
            Ok(expected)
        );
        self
    }
    pub fn set_ratio (&mut self, ratio: (u128, u128)) -> &mut Self  {
        // TODO query!!!
        test_handle(
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
    pub fn set_threshold (&mut self, threshold: Time) -> &mut Self  {
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
    pub fn at (&mut self, t: Time) -> &mut Self {
        self.env = env(&self.address, t);
        self
    }
    pub fn later (&mut self) -> &mut Self {
        let t: Time = rand::thread_rng().gen();
        self.at(self.env.block.time + t)
    }
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
    pub fn is_unregistered (&mut self) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Err(StdError::generic_err(format!("deposit tokens for 100 more blocks to be eligible")))
        )
    }
    pub fn must_wait (&mut self, remaining: Time) -> &mut Self {
        self.test_handle(
            RewardsHandle::Claim {},
            Err(StdError::generic_err(
                format!("deposit tokens for {} more blocks to be eligible", remaining)
            ))
        )
    }

    pub fn test_handle (&mut self, msg: RewardsHandle, expected: StdResult<HandleResponse>) -> &mut Self {
        test_handle(
            self.deps, &self.env, self.address.clone(),
            msg, expected
        ); self
    }

    pub fn status (&mut self) -> User {
        match Rewards::query_status(
            &*self.deps, self.env.block.time, Some(self.address.clone()), Some(String::from(""))
        ).unwrap() {
            crate::RewardsResponse::Status { user, .. } => user.unwrap(),
            _ => panic!()
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

pub fn test_handle (
    deps: &mut Deps, env: &Env, address: HumanAddr, msg: RewardsHandle, expected: StdResult<HandleResponse>
) {
    use yansi::Paint;
    print!("\n| {} | {} | {:?} | ", env.block.time, address, Paint::yellow(&msg));
    let result = Rewards::handle(deps, env, msg);
    if result == expected {
        println!("{}", Paint::green("OK"));
    } else {
        println!("{} <-", address);
        match result {
            Ok(HandleResponse { ref messages, ref log, ref data }) => {
                println!("messages:");
                for message in messages.iter() {
                    match message {
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            ref contract_addr, ref callback_code_hash, ref msg, ref send
                        }) => {
                            println!("[{}#{}] {} {:?}",
                                Paint::red(contract_addr),
                                Paint::red(callback_code_hash),
                                Paint::red(&std::str::from_utf8(msg.as_slice()).unwrap().trim_end()),
                                Paint::red(send));
                        },
                        _ => println!("- {:?}", Paint::red(message))
                    }
                }
                println!("log:  {:?}", log);
                println!("data: {:?}", data);
            },
            Err(StdError::GenericErr { ref msg, .. } ) => {
                println!("\n{} <- {:?}", address, Paint::red(&msg))
            },
            _ => println!("\n{} <- {:?}", address, Paint::red(&result))
        };
        println!("{}", Paint::red("Was expecting:"));
    }
    match expected {
        Ok(HandleResponse { ref messages, ref log, ref data }) => {
            println!("messages:");
            for message in messages.iter() {
                match message {
                    CosmosMsg::Wasm(WasmMsg::Execute {
                        ref contract_addr, ref callback_code_hash, ref msg, ref send
                    }) => {
                            println!("[{}#{}] {} {:?}",
                                Paint::green(contract_addr),
                                Paint::green(callback_code_hash),
                                Paint::green(&std::str::from_utf8(msg.as_slice()).unwrap().trim_end()),
                                Paint::green(send));
                    },
                    _ => println!("- {:?}", Paint::green(message))
                }
            }
            println!("log:  {:?}", log);
            println!("data: {:?}", data);
        }
        Err(StdError::GenericErr { ref msg, .. } ) => {
            println!("\n{} <- {:?}", address, Paint::green(&msg))
        },
        _ => println!("\n{} <- {:?}", address, Paint::green(&result))
    };
    assert_eq!(result, expected);
}

impl<'a> BadmanRole<'a> {
    pub fn at (&mut self, t: Time) -> &mut Self {
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
            Err(e) => unimplemented!()
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
            _ => unimplemented!()
        }
    }
    pub fn increment_balance (&mut self, amount: u128) -> () {
        self.balance += amount.into();
    }
    pub fn decrement_balance (&mut self, amount: u128) -> StdResult<()> {
        self.balance = (self.balance - amount.into())?;
        Ok(())
    }
}
