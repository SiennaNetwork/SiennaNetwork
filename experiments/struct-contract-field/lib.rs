use fadroma::scrt::{cosmwasm_std::*, contract::{message, messages}};
use serde::{Serialize, de::DeserializeOwned};
use std::{rc::Rc, cell::{RefCell, RefMut}};

pub fn init <S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    let Extern { storage, api, querier } = deps;
    Contract { storage, api, querier, env }.init(msg)
}

pub fn handle <S: Storage, A: Api, Q: Querier> (
    deps: &mut Extern<S, A, Q>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    let Extern { storage, api, querier } = deps;
    Contract { storage, api, querier, env }.handle(msg)
}

pub fn query <S: Storage, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>,
    msg:  Query
) -> StdResult<Binary> {
    let Extern { storage, api, querier } = deps;
    to_binary(&(Contract { storage, api, querier, env: () }.query(msg)?))
}

struct Contract <S, A, Q, E> {
    storage: S,
    api:     A,
    querier: Q,
    env:     E
}

trait Transactions<I, H> {
    fn init   (&mut self, msg: I) -> InitResult;
    fn handle (&mut self, msg: H) -> HandleResult;
}

impl<S: Storage, A: Api, Q: Querier>
Transactions<Init, Handle> for Contract<&mut S, &mut A, &mut Q, Env> {
    fn init (&mut self, msg: Init) -> InitResult {
        Ok(InitResponse { messages: vec![], log: vec![] })
    }
    fn handle (&mut self, msg: Handle) -> HandleResult {
        Ok(HandleResponse { messages: vec![
            Model::new(self).submodel("foo".to_string())
        ], log: vec![], data: None })
    }
}

message!(Init {});
messages!(Handle {});
messages!(Query {});
messages!(Response {});

trait Queries<Q, R> {
    fn query (self, msg: Q) -> StdResult<R>;
}

impl<S: Storage, A: Api, Q: Querier>
Queries<Query, Response> for Contract<&S, &A, &Q, ()> {
    fn query (self, msg: Query) -> StdResult<Response> {
        Err(StdError::generic_err("not implemented"))
    }
}

struct Model<S> {
    storage: Rc<RefCell<S>>,
    field1:  Field<S, Uint128>
}

impl <S> Model <S> {
    pub fn new (storage: Rc<RefCell<S>>) -> Self {
        Self {
            storage: Rc::clone(&storage),
            field1: Field::new(Rc::clone(&storage), format!("field1").into())
        }
    }
    pub fn submodel (self, id: String) -> Submodel<S> {
        Submodel::new(Rc::clone(&self.storage), self, id)
    }
}

struct Submodel<S> {
    storage: Rc<RefCell<S>>,
    parent:  Model<S>,
    field2:  Field<S, Uint128>,
    field3:  Field<S, Uint128>
}

impl<S> Submodel <S> {
    pub fn new (storage: Rc<RefCell<S>>, parent: Model<S>, id: String) -> Self {
        Self {
            storage: Rc::clone(&storage),
            parent,
            field2: Field::new(Rc::clone(&storage), format!("field2/{}", id).into()),
            field3: Field::new(Rc::clone(&storage), format!("field3/{}", id).into())
        }
    }
}

struct Field <S, V> {
    storage: Rc<RefCell<S>>,
    key:     Vec<u8>,
    value:   Option<V>
}

impl<S, V> Field<S, V> {
    pub fn new (storage: Rc<RefCell<S>>, key: String) -> Self {
        Self { storage, key: key.into(), value: None }
    }
}

impl<S: ReadonlyStorage, V: Copy + DeserializeOwned> Field<S, V> {
    pub fn value (mut self) -> StdResult<V> {
        match self.value {
            Some(value) => Ok(value),
            None => {
                match self.storage.borrow().get(&self.key) {
                    Some(data) => {
                        let value = from_slice(&data)?;
                        self.value = Some(value);
                        Ok(value)
                    },
                    None => Err(StdError::generic_err("not in storage"))
                }
            }
        }
    }

}

impl<S: ReadonlyStorage + Storage, V: Serialize> Field<S, V> {
    pub fn store (mut self, value: V) -> StdResult<()> {
        {
            let mut storage: RefMut<_> = self.storage.borrow_mut();
            storage.set(&self.key, &to_vec(&value)?);
        }
        self.value = Some(value);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use fadroma::scrt::cosmwasm_std::from_binary;
    use fadroma::scrt::cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn test () {
        let mut deps = mock_dependencies(10, &[]);
        let env  = mock_env("", &[]);
        println!("{:?}", init(&mut deps, env, Init {}));
    }
}
