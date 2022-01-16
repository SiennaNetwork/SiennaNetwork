# Sienna Rewards Migrations

## Migration flow

*Note: code links point to [commit `6bb07ab9a720258fa1e19941397c5bb346186512`](https://github.com/SiennaNetwork/sienna/tree/6bb07ab9a720258fa1e19941397c5bb346186512),
which does not contain this file.*

Consider two contracts, NEXT and PREV.

### Preparation

- [Admin of NEXT](https://github.com/SiennaNetwork/sienna/blob/fix/rewards-audit/contracts/rewards/migration.rs#L159)
  calls NEXT::EnableMigrationFrom(PREV).
  - [Storage key `$NEXT_ADDR/CAN_MIGRATE_FROM/$PREV_ADDR` is set to Some(PREV)](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L175).
- [Admin of PREV](https://github.com/SiennaNetwork/sienna/blob/fix/rewards-audit/contracts/rewards/migration.rs#L40) calls PREV::EnableMigrationTo(NEXT).
  - [Storage key `$PREV_ADDR/CAN_MIGRATE_TO/$NEXT_ADDR` is set to Some(NEXT)](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L57).

### Execution

- USER calls NEXT::RequestMigration(PREV).
  - [NEXT::handle calls NEXT::can_immigrate_from(PREV)](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L151).
    - [If `$NEXT_ADDR/CAN_MIGRATE_FROM/$PREV_ADDR` is not `Some(_)`](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L207),
      the transaction fails here. So the user can't just introduce any invalid value as `prev`.
    - [This address check](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L208)
      is redundant, as it is not possible to take `sender_link.code_hash` into account -
      there is nothing to compare it to. So it would be more gas-efficient to just store `true`
      instead of the link.

- NEXT calls PREV::ExportState(USER).
  - [PREV::handle calls PREV::can_export_state(USER)](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L36)
    - The value of `env.message.sender` is now equal to the address of NEXT,
      not the address of the user that initiated the transaction.
    - [If `$PREV_ADDR/CAN_MIGRATE_TO/$NEXT_ADDR` is not `Some(_)`](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L107),
      the transaction fails here.
    - [This address check](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L108)
      is redundant, as it is not possible to take `next_contract.code_hash` into account -
      there is nothing to compare it to. So it would be more gas-efficient to just store `true`
      instead of the link.

- PREV calls NEXT::ReceiveMigration(USER, SNAPSHOT).
  - [NEXT::handle calls NEXT::can_immigrate_from(PREV) again](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L155).
    - This was what was missing in HAL-01.
    - [If `$NEXT_ADDR/CAN_MIGRATE_FROM/$PREV_ADDR` is not `Some(_)`](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L207),
      the transaction fails here.

## HAL-04 + HAL-01

### Let us consider the first statement:

> - `handle_request_migration` function does not verify that old reward
> pool is closed and enabled to migrate to a new one (`CAN_MIGRATE_TO`),
> so it can be called at anytime and not just during migration process
> as expected.

First of all, it is talking about two separate things at the same time.

* One thing is enabling migrations via `CAN_MIGRATE_TO` +
  and `CAN_MIGRATE_FROM` state variables.
* The other thing is checking if the pool is closed.

Let's factor closing the pool out of the equation, as it is not
strictly necessary to perform a migration.

That leaves us with the claim that:

> - `handle_request_migration` function does not verify that old reward
> pool is enabled to migrate to a new one (`CAN_MIGRATE_TO`),
> so it can be called at anytime and not just during migration process
> as expected.

This is incorrect on two points:

- Migration is per-user, and has to be individually initiated by each user
  *via calling `NEXT::RequestMigration`*. Therefore, it cannot be said that
  it "is expected to be called during migration process", because *it initiates
  the individual migration process*.
- It is not true that `handle_request_migration` does not check `CAN_MIGRATE_TO`.
  This check is performed by calling `can_immigrate_from`
  in `Immigration::handle` at [`migration.rs:151`](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L151).

### Let us now consider the second statement.

> - In the function mentioned above, previous reward pool address (prev)
> is not restricted; thus users can execute any potentially malicious
> message on behalf of current reward pool.

With closing the pool out of the equation, this statement is functionally
equivalent to the first one, with the addition of a factually incorrect conclusion.

- It is not true that "previous reward pool address (prev)" is not
  restricted. If `prev` is equal to any other contract than the ones enabled
  via `CAN_MIGRATE_FROM`, `can_immigrate_from` will return an
  error and the transaction will stop here.
- The function `handle_request_migration` only ever emits one message,
  `ExportState` so it is incorrect that "users can execute any potentially
  malicious message on behalf of current reward pool". The user can only use
  this method to send a specific message, and only to specific addresses.

### Let us consider the third, and final statement.

>- handle_export_state function verifies the value of can_export_state
>function before continuing with its execution. However, this latter
>function entirely ignores sender address, which means that does not
>exist access control for handle_export_state function

The statement that "[`can_export_state`] entirely ignores sender address
is incorrect.

- The version of CosmWasm that Secret Network uses does not propagate
  the initiator address across multi-step transactions. So even if we needed
  the user's address to validate `handle_export_state`, we wouldn't have
  access to it. That's why we're explicitly passing it via the `migrant` field.
- `handle_export_state` is meant to be called by `NEXT`. You will notice
  that [the very first thing `can_export_state` does](https://github.com/SiennaNetwork/sienna/blob/6bb07ab9a720258fa1e19941397c5bb346186512/contracts/rewards/migration.rs#L99-L101)
  is make sure that the sender address is not equal to the `migrant` address,
  i.e. that this method is not called by the user themselves.
- Subsequently, it peforms a check that the sender address is whitelisted in
  `CAN_MIGRATE_TO`. So if anyone else but `NEXT` calls `PREV::ExportState`,
  they get shunned.

As for closing the pool, if the business case deems it necessary to
close the old pool when enabling migrations to the new one, we'll make sure to close it.

### Conclusion

I agree it's not a simple flow but it's the best thing I could come up with
under the circumstances. Thanks for finding the hole in `ReceiveMigration`,
and the call to productive debate - polemics are a great basis for in-depth
documentation :)

Suggestions:

* Mark HAL-01 as resolved and change recommendations to something pertaining to
  the actual security issue, because the current recommendations are exactly what
  we're doing already. "Add permission check to ImmigrationHandle::ReceiveMigration"
  would do fine.
* Reassess validity and severity of HAL-04, because (in the absence of a PoC showing
  how any of the statements from the description are true) my resolution for that is
  `NOTABUG WONTFIX` :)
