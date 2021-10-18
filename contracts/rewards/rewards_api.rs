use fadroma::scrt::{
    contract::{message, messages},
    cosmwasm_std::{HumanAddr, Uint128},
    callback::ContractInstance as ContractLink,
};
use crate::rewards_math::*;
use crate::rewards_vk::ViewingKey;

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
    ChangeAdmin {
        address: HumanAddr
    }

    SetProvidedToken {
        address:   HumanAddr,
        code_hash: String
    }

    ChangeRatio {
        numerator:   Uint128,
        denominator: Uint128
    }
    ChangeThreshold {}
    ChangeCooldown {}

    ClosePool {}
    ReleaseSnip20 {}

    CreateViewingKey {}
    SetViewingKey {}

    Lock {}
    Retrieve {}
    Claim {}
});

messages!(Query {

    Admin {}

    PoolInfo {
        at: Time
    }

    /// Requires the user's viewing key.
    UserInfo {
        at:      Time,
        address: HumanAddr,
        key:     String
    }

    /// For Keplr integration
    TokenInfo {}

    /// For Keplr integration
    Balance {}

});

messages!(Response {

    /// Response from `Query::PoolInfo`
    PoolInfo {
        lp_token:         ContractLink<HumanAddr>,
        reward_token:     ContractLink<HumanAddr>,

        it_is_now:        Time,

        pool_last_update: Time,
        pool_lifetime:    Volume,
        pool_locked:      Amount,

        #[cfg(feature="pool_closes")]
        pool_closed:      Option<String>,

        pool_balance:     Amount,
        pool_claimed:     Amount,

        #[cfg(feature="age_threshold")]
        pool_threshold:   Time,

        #[cfg(feature="claim_cooldown")]
        pool_cooldown:    Time,

        #[cfg(feature="pool_liquidity_ratio")]
        pool_liquid:      Amount
    }

    /// Response from `Query::UserInfo`
    UserInfo {
        it_is_now:        Time,

        pool_last_update: Time,
        pool_lifetime:    Volume,
        pool_locked:      Amount,

        #[cfg(feature="pool_closes")]
        pool_closed:      Option<String>,

        user_last_update: Option<Time>,
        user_lifetime:    Volume,
        user_locked:      Amount,
        user_share:       Amount,
        user_earned:      Amount,
        user_claimed:     Amount,
        user_claimable:   Amount,

        #[cfg(feature="age_threshold")]
        user_age:         Time,

        #[cfg(feature="claim_cooldown")]
        user_cooldown:    Time
    }

    Admin {
        address: HumanAddr
    }

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
