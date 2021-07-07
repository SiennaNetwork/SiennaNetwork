use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
//use snafu::GenerateBacktrace;
use fadroma::scrt::cosmwasm_std::{Uint128, CanonicalAddr, StdResult, StdError};

/// Unit of time
pub type Seconds = u64;

/// Unit of account
pub const ONE_SIENNA: u128 = 1000000000000000000u128;

/// A reward pool distributes rewards from its balance among liquidity providers
/// depending on how much liquidity they have provided and for what duration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RewardPool {
    providers:   Vec<Provider>,
    last_update: Seconds
}

impl RewardPool {
    /// Create an empty reward pool
    pub fn new (now: Seconds) -> Self {
        RewardPool {
            providers:   vec![],
            last_update: now
        }
    }

    /// Set the current amount of liquidity provided by an address
    pub fn set (&mut self, now: Seconds, address: CanonicalAddr, amount: Uint128) {
        let mut found = false;
        // If self is a known provider, update it
        for provider in self.providers.iter_mut() {
            if provider.address == address {
                provider.current = amount;
                found = true;
                break
            }
        }
        // If self is the first time self address provides liquidity, append it.
        // Maybe prepending it to the list is more efficient, maybe not?
        // Different implementations can be tested using Cargo's "features" feature.
        if !found {
            self.providers.push(Provider {
                address, since: now, current: amount, lifetime: amount, claimed: Uint128::zero()
            })
        }
        // This is also where periods of zero liquidity begin/end, and therefore their duration
        // may be kept track of and later subtracted from everyone's age, so as not to reward
        // providers for periods of zero liquidity. This will make the rewards budget last longer.
    }

    /// Add liquidity
    pub fn lock (
        &mut self, now: Seconds, address: CanonicalAddr, amount: Uint128
    ) -> Uint128 {
        let mut found = false;
        for provider in self.providers.iter_mut() {
            if provider.address == address {
                provider.current += amount;
                found = true;
                break
            }
        }
        if !found {
            self.providers.push(Provider {
                address, since: now, current: amount, lifetime: amount, claimed: Uint128::zero()
            })
        }
        self.update(now);
        return amount
    }

    /// Remove liquidity
    pub fn retrieve (
        &mut self, now: Seconds, address: CanonicalAddr, amount: Uint128
    ) -> StdResult<Uint128> {
        for provider in self.providers.iter_mut() {
            if provider.address == address {
                if provider.current < amount {
                    return Err(StdError::GenericErr {
                        msg: "not enough funds locked".into(),
                        backtrace: None
                    })
                }
                provider.current = (provider.current - amount)?;
                self.update(now);
                return Ok(amount)
            }
        }
        Err(StdError::GenericErr {
            msg: "not a provider".into(),
            backtrace: None
        })
    }

    /// Calculate how much a provider can claim, subtract it from the total balance, and return it.
    pub fn claim (
        &mut self, now: Seconds, balance: Uint128, address: CanonicalAddr
    ) -> StdResult<Uint128> {
        let mut total = Uint128::zero();
        let mut selected = None;
        for provider in self.providers.iter_mut() {
            total += provider.lifetime;
            if provider.address == address {
                selected = Some(provider)
            }
        }
        match selected {
            None => Err(StdError::GenericErr { msg: "not a provider".into(), backtrace: None }),
            Some(mut provider) => {
                // A minimum provider age might need to be enforced here,
                // since it takes the contract 24h to achieve equilibrium.
                let lifetime_reward = balance.multiply_ratio(provider.lifetime, total);
                let reward = (lifetime_reward - provider.claimed)?;
                provider.claimed = lifetime_reward;
                self.update(now);
                Ok(reward)
            }
        }
    }

    /// Update each provider's lifetime-provided liquidity by a multiple of elapsed seconds
    pub fn update (&mut self, now: Seconds) {
        // Update the clock
        let elapsed = now - self.last_update;
        self.last_update = now;
        for provider in self.providers.iter_mut() {
            // What matters here is the proportion, so if it turns out that
            // multiplying by the number of seconds creates a risk of u128 overflow,
            // the same overall result (at a slightly reduced precision) can be obtained
            // by multiplying by seconds / SOME_INTERVAL would yield the same overall result.
            // Setting the interval to the block time should be a safe default in any case.
            // Or measure the interval in block height - but I don't know if multiple providers
            // claiming at different seconds during the same block would cause any problems.
            provider.lifetime += provider.current.multiply_ratio(elapsed, 1u128);
        }
    }

}

/// A liquidity provider's address and parameters. This is a data object;
/// the calculations are implemented in the methods of RewardPool, so as not to
/// deepen the stack for what's effectively a couple of simple arithmetical operations.
/// Further optimization may be achievable by storing different fields separately and
/// only deserializing those that are needed for the individual operations above -
/// depending on how much influence memory layout has on gas costs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Provider {
    address:  CanonicalAddr,
    since:    Seconds,
    current:  Uint128,
    lifetime: Uint128,
    claimed:  Uint128
}
