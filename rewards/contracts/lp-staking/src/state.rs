use cosmwasm_std::HumanAddr;
use scrt_finance::ContractInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub admin: HumanAddr,
    pub reward_token: ContractInfo,
    pub inc_token: ContractInfo,
    pub master: ContractInfo,
    /// Will be set for the inc_token and reward_token
    pub viewing_key: String,
    pub prng_seed: Vec<u8>,
    pub is_stopped: bool,
    pub own_addr: HumanAddr,
    pub deadline: u64
}
