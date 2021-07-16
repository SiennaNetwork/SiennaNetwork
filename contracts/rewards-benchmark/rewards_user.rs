use crate::rewards_math::*;
use crate::rewards_pool::*;

use fadroma::scrt::{cosmwasm_std::{StdError, CanonicalAddr}, storage::traits2::*};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }

/// How much liquidity has each user provided since they first appeared;
/// incremented in intervals of (blocks since last update * current balance)
const TALLIED: &[u8] = b"user_lifetime/";
/// How much liquidity does each user currently provide
const BALANCE: &[u8] = b"user_current/";
/// When did each user's liquidity amount last change
const UPDATED: &[u8] = b"user_updated/";
/// How much rewards has each user claimed so far
const CLAIMED: &[u8] = b"user_claimed/";
/// For how many units of time has this user provided liquidity
const EXISTED: &[u8] = b"user_existed/";

pub struct User <S> {
    pool:    Pool<S>,
    address: CanonicalAddr
}

impl<S: ReadonlyStorage> Readonly<S> for User<S> {
    fn storage (&self) -> &S { &self.pool.storage }
}

//impl<S: ReadonlyStorage> UserReadonly<S> for User<&S> {
    //fn pool (&self) -> &Pool<S> {
        //&self.pool
    //}
    //fn address (&self) -> &[u8] {
        //self.address.as_slice()
    //}
    //// trait fields WHEN???
//}

impl<S: ReadonlyStorage> User<S> {

    //fn pool    (&self) -> &Pool<S>;

    //fn address (&self) -> &[u8];

    pub fn updated (&self) -> StdResult<Monotonic> {
        let address = self.address.as_slice();
        match self.load_ns(UPDATED, &address)? {
            Some(x) => Ok(x),
            None    => error!("UPDATED missing")
        }
    }

    pub fn existed (&self) -> StdResult<Monotonic> {
        let address = self.address.as_slice();
        Ok(self.load_ns(EXISTED, &address)?.unwrap_or(0 as Monotonic))
    }

    pub fn elapsed (&self) -> StdResult<Monotonic> {
        Ok(self.pool.now()? - self.updated()?)
    }

    pub fn balance (&self) -> StdResult<Amount> {
        let address = self.address.as_slice();
        Ok(self.load_ns(BALANCE, address)?.unwrap_or(Amount::zero()))
    }

    pub fn claimed (&self) -> StdResult<Amount> {
        let address = self.address.as_slice();
        Ok(self.load_ns(CLAIMED, address)?.unwrap_or(Amount::zero()))
    }

    pub fn age (&self) -> StdResult<Monotonic> {
        let address = self.address.as_slice();
        let existed = self.existed()?;
        let balance = self.balance()?;
        if balance > Amount::zero() {
            // if user is currently providing liquidity,
            // the time since last update gets added to the age
            Ok(existed + self.elapsed()?)
        } else {
            Ok(existed)
        }
    }

    pub fn lifetime (&self) -> StdResult<Volume> {
        let address = self.address.as_slice();
        tally(
            self.load_ns(TALLIED, address)?.unwrap_or(Volume::zero()),
            self.pool.now()? - self.updated()?,
            self.balance()?)
    }

    pub fn reward (&self, balance: Amount) -> StdResult<(Amount, Amount, Amount)> {
        let pool = self.pool.lifetime()?;
        if pool > Volume::zero() {
            let user     = self.lifetime()?;
            let budget   = self.pool.budget(balance)?;
            let ratio    = self.pool.ratio()?;
            let unlocked = Volume::from(budget)
                .multiply_ratio(user, pool)?
                .multiply_ratio(ratio.0, ratio.1)?
                .low_u128().into();
            let claimed  = self.claimed()?;
            if unlocked > claimed {
                Ok((unlocked, claimed, (unlocked - claimed)?))
            } else {
                Ok((unlocked, claimed, Amount::zero()))
            }
        } else {
            error!("pool is empty")
        }
    }

}

// and now the same for when the pool gets passed a mutable version of deps

impl <S: Storage + ReadonlyStorage> Readonly<S> for User<&mut S> {
    fn storage (&self) -> &S {
        &self.pool.storage
    }
}

impl <S: Storage + ReadonlyStorage> UserReadonly<S> for User<&mut S> {
    fn pool (&self) -> &Pool<S> {
        self.pool
    }
    fn address (&self) -> &[u8] {
        self.address.as_slice()
    }
    // trait fields WHEN???
}

// and then some more

impl <S: Storage + ReadonlyStorage> Writable<S> for User<S> {
    fn storage_mut (&mut self) -> &mut S {
        &mut self.pool.storage
    }
}

impl <S: Storage + ReadonlyStorage> User <S> {

    pub fn lock (&mut self, increment: Amount) -> StdResult<Amount> {
        let address = self.address.as_slice();

        // If current balance is > 0, increment the user's age
        // with the time since the last update
        self.save_ns(EXISTED, address, self.age()?);

        // Increment liquidity from user
        self.save_ns(BALANCE, address, self.balance()? + increment)?;

        // Remember when the user was last updated, i.e. now
        self.save_ns(UPDATED, address, self.pool.now())?;

        // Increment liquidity in pool
        self.pool.update(self.pool.balance()? + increment.into());

        // Return the amount to lock
        Ok(increment)
    }

    pub fn retrieve (&mut self, decrement: Amount) -> StdResult<Amount> {
        let balance = self.balance()?;

        // Must have enough balance to retrieve
        if balance < decrement {
            error!(format!("not enough balance ({} < {})", balance, decrement))
        } else {
            // Remove liquidity from user
            let new_user_balance = (balance - decrement)?;
            self.save_ns(BALANCE, self.address.as_slice(), new_user_balance)?;

            // Remove liquidity from pool
            self.pool.update((self.pool.balance()? - decrement.into())?);

            // Return the amount to return
            Ok(decrement)
        }
    }

    pub fn claim (&mut self, balance: Amount) -> StdResult<Amount> {
        let age       = self.age()?;
        let threshold = self.pool.threshold()?;

        // Age must be above the threshold to claim
        if age >= threshold {
            let (unlocked, _claimed, claimable) =
                self.reward(balance)?;
            if claimable > Amount::zero() {
                // If there is some new reward amount to claim:
                self.save_ns(CLAIMED, self.address.as_slice(), &unlocked)?;
                Ok(claimable)
            } else if unlocked > Amount::zero() {
                // If this user has claimed all its rewards so far:
                error!("already claimed")
            } else {
                // If this user never had any rewards to claim:
                error!("nothing to claim")
            }
        } else {
            error!(format!("{} blocks until eligible", threshold - age))
        }
    }

}
