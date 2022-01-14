use lend_shared::fadroma::{
    ensemble::{ContractHarness, MockDeps},
    from_binary, to_binary, Binary, Env, HandleResponse, InitResponse, StdResult,
};

use amm_snip20;
use lend_oracle;
use lend_overseer;

pub struct Token;

pub struct Overseer;

impl ContractHarness for Overseer {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        lend_overseer::init(deps, env, from_binary(&msg)?, lend_overseer::DefaultImpl)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        lend_overseer::handle(deps, env, from_binary(&msg)?, lend_overseer::DefaultImpl)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let result = lend_overseer::query(deps, from_binary(&msg)?, lend_overseer::DefaultImpl)?;

        to_binary(&result)
    }
}

pub struct Oracle;

impl ContractHarness for Oracle {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        lend_oracle::init(deps, env, from_binary(&msg)?, lend_oracle::DefaultImpl)
    }

    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        lend_oracle::handle(deps, env, from_binary(&msg)?, lend_oracle::DefaultImpl)
    }

    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        let result = lend_oracle::query(deps, from_binary(&msg)?, lend_oracle::DefaultImpl)?;

        to_binary(&result)
    }
}
