#![allow(dead_code)]

#[cfg(test)]
mod mgmt_test;
#[cfg(test)]
mod setup;
#[cfg(test)]
mod tge_test;

#[macro_export]
macro_rules! impl_contract_harness_default {
    ($name:ident, $contract:ident) => {
        impl ContractHarness for $name {
            fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
                $contract::init(deps, env, from_binary(&msg)?, $contract::DefaultImpl)
            }

            fn handle(
                &self,
                deps: &mut MockDeps,
                env: Env,
                msg: Binary,
            ) -> StdResult<HandleResponse> {
                $contract::handle(deps, env, from_binary(&msg)?, $contract::DefaultImpl)
            }

            fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
                $contract::query(deps, from_binary(&msg)?, $contract::DefaultImpl)
            }
        }
    };
}
