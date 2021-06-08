use cosmwasm_std::{Env, HumanAddr, ReadonlyStorage, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket, bucket, bucket_read};
use libraries::core_library::{ReserveData, UserReserveData};
use libraries::uint256::Uint256;
pub const RESERVERS_KEY: &[u8] = b"reserves";
pub const USERS_RESERVE_DATA_KEY: &[u8] = b"usersReserveData";

pub fn reservers<'a, S: Storage>(storage: &'a mut S) -> Bucket<'a, S, ReserveData> {
    bucket(RESERVERS_KEY, storage)
}

pub fn users_reserve_data<'a, S: Storage>(storage: &'a mut S) -> Bucket<'a, S, UserReserveData> {
    bucket(USERS_RESERVE_DATA_KEY, storage)
}

pub fn read_reservers<'a, S: ReadonlyStorage>(storage: &'a S) -> ReadonlyBucket<'a,S,ReserveData>{
    bucket_read(RESERVERS_KEY, storage)
}

pub fn read_users_reseve_data<'a,S: ReadonlyStorage>(storage: &'a mut S) -> ReadonlyBucket<'a,S,UserReserveData> {
    bucket_read(USERS_RESERVE_DATA_KEY, storage)

}

pub const CORE_REVISION: u64 = 0x6;

/**
 * @dev returns the revision number of the contract
 **/
pub fn get_revision() -> Uint256 {
    let res: Uint256 = CORE_REVISION.into();
    res
}

/**
 * @dev updates the state of the core as a result of a deposit action
 * @param _reserve the address of the reserve in which the deposit is happening
 * @param _user the address of the the user depositing
 * @param _amount the amount being deposited
 * @param _isFirstDeposit true if the user is depositing for the first time
 **/

pub fn update_state_on_deposit<S: ReadonlyStorage>(
    _storage: &S,
    _env:Env,
    _reserver: HumanAddr,
    _user: HumanAddr,
    _amount: Uint256,
    _is_first_deposit: bool,
) {
    let reader = read_reservers(_storage);
    let mut a = reader.load(_reserver.as_str().as_bytes()).unwrap();
    a.update_cumulative_indexes(&_env);
    unimplemented!();
}
