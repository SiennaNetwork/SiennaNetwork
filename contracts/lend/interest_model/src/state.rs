use lend_shared::{
    core::JumpRateInterest,
    fadroma::{
        cosmwasm_std::{StdResult, Storage},
        storage::{load, save},
    },
};

static KEY_INTEREST_MODEL: &[u8] = b"interest_model";

pub fn save_interest_model(storage: &mut impl Storage, model: &JumpRateInterest) -> StdResult<()> {
    save(storage, KEY_INTEREST_MODEL, model)
}

pub fn load_interest_model(storage: &impl Storage) -> StdResult<JumpRateInterest> {
    Ok(load(storage, KEY_INTEREST_MODEL)?.unwrap())
}
