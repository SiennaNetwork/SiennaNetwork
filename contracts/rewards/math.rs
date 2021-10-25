//! Continuous rewards are based on a simple accumulating function.
//!
//! The current value of the pool/user `liquidity` and pool `age` parameters
//! is based on the previous value + the current value multiplied by the time that the current
//! value has been active.
//!
//! Both parameters' current values depends on the results of the `lock` and `unlock`
//! user transactions. So when one of these is invoked, the "previous" values are updated;
//! then, during read-only queries the new "current" can be computes based on
//! the stored value and the elapsed time since the last such transaction.

use fadroma::{
    scrt::{StdResult, StdError, Uint128},
    scrt_uint256::Uint256
};

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Time   = u64;

/// Amount of funds
pub type Amount = Uint128;

/// Liquidity = amount (u128) * time (u64)
pub type Volume = Uint256;

/// A ratio represented as tuple (nom, denom)
pub type Ratio  = (Uint128, Uint128);

/// 100% with 6 digits after the decimal
pub const HUNDRED_PERCENT: u128 = 100000000u128;

/// Seconds in 24 hours
pub const DAY: Time = 86400;

/// Calculate the current total based on the stored total and the time since last update.
pub fn accumulate (
    total_before_last_update: Volume,
    time_updated_last_update: Time,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    total_before_last_update + Volume::from(value_after_last_update)
        .multiply_ratio(time_updated_last_update, 1u128)? }

pub trait Diminish<
    T: From<u64> + From<Self>,
    N: Eq + From<u64
>>: Copy {
    /// Divide self on num/denom; throw if num > denom or if denom == 0
    fn diminish         (self, num: N, denom: N) -> StdResult<T>;
    /// Diminish, but return 0 if denom == 0
    fn diminish_or_max  (self, num: N, denom: N) -> StdResult<T> {
        if denom == 0u64.into() {
            Ok(self.into()) }
        else {
            self.diminish(num, denom) } }
    /// Diminish, but return self if denom == 0
    fn diminish_or_zero (self, num: N, denom: N) -> StdResult<T> {
        if denom == 0u64.into() {
            Ok(0u64.into()) }
        else {
            self.diminish(num, denom) } } }

impl Diminish<Self, Time> for Volume {
    fn diminish (self, num: Time, denom: Time) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function")) }
        else {
            Ok(self.multiply_ratio(num, denom)?) } } }

impl Diminish<Self, Volume> for Volume {
    fn diminish (self, num: Uint256, denom: Uint256) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function")) }
        else {
            Ok(self.multiply_ratio(num, denom)?) } } }

impl Diminish<Self, Amount> for Amount {
    fn diminish (self, num: Amount, denom: Amount) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function")) }
        else {
            Ok(self.multiply_ratio(num, denom)) } } }

impl Diminish<Self, Time> for Amount {
    fn diminish (self, num: Time, denom: Time) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function")) }
        else {
            Ok(self.multiply_ratio(num, denom)) } } }
