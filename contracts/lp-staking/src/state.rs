use cosmwasm_std::HumanAddr;
use cosmwasm_utils::ContractInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub admin: HumanAddr,
    pub reward_token: ContractInfo,
    pub inc_token: ContractInfo,
    pub master: ContractInfo,
    pub viewing_key: String,
    pub prng_seed: Vec<u8>,
    pub is_stopped: bool,
    pub own_addr: HumanAddr,
    pub deadline: u64
}
