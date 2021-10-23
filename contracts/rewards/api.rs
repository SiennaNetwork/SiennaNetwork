use fadroma::scrt::{
    contract::{message, messages},
    cosmwasm_std::{HumanAddr, Uint128},
    callback::ContractInstance as ContractLink,
};
use crate::math::*;
use crate::auth::ViewingKey;

message!(Init {
    admin:        Option<HumanAddr>,
    lp_token:     Option<ContractLink<HumanAddr>>,
    reward_token: ContractLink<HumanAddr>,
    viewing_key:  ViewingKey,
    ratio:        Option<Ratio>,
    threshold:    Option<Time>,
    cooldown:     Option<Time>
});

messages!(Handle {
    Auth(AuthHandle)
    Migration(MigrationHandle)
    Rewards(RewardsHandle)
    ReleaseSnip20 {
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    }
});

messages!(Query {
    Auth(AuthQuery),
    Rewards(RewardsQuery),

    /// For Keplr integration
    TokenInfo {}
    /// For Keplr integration
    Balance { address: HumanAddr, key: String }
});

messages!(Response {
    Auth(AuthResponse),
    Rewards(RewardsResponse),

    /// Keplr integration
    TokenInfo {
        name:         String,
        symbol:       String,
        decimals:     u8,
        total_supply: Option<Amount>
    }

    /// Keplr integration
    Balance {
        amount: Amount
    }

});
