pub const CONFIG_KEY: &[u8] = b"config";
pub const REWARD_POOL_KEY: &[u8] = b"rewardpool";
pub const TOKEN_INFO_KEY: &[u8] = b"tokeninfo";
pub const VIEWING_KEY_KEY: &[u8] = b"viewingkey";

pub const RESPONSE_BLOCK_SIZE: usize = 256;

// TODO: get those as an input for specific coins, as some coins might require different scales than others
// pub const INC_TOKEN_SCALE: u128 = 1; // LP Tokens and Gov Token is at the same scale
pub const REWARD_SCALE: u128 = 1_000_000_000_000; // 10 ^ 12
