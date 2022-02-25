# Sienna Rewards

## Procedures

### Preparation

To install dependencies for the deployment scripts:

```sh
pnpm -w i
```

### Cleanup

To restart from a clean slate:

```sh
pnpm -w ops localnet-1.2 reset
```

### Unit testing

Unit tests are implemented in the `test/` directory.
The harness in `test/mod.rs` contains shorthands for
the test steps, and generates CSV reports for each test.

```sh
cargo test -p sienna-rewards
```

### Integration testing

Integration testing is conducted via a Web dashboard
which executes the smart contract and its dependencies
within the browser's WASM environment.

```sh
pnpm -w dash:build
pnpm -w dash
# open localhost:8080
```

### Benchmarking

```sh
pnpm -w dev bench rewards
```

### Local deployment

```sh
pnpm -w ops localnet-1.2 audit rewards deploy 60
```

For convenience, `60` is a custom bonding period, in seconds (the default is 86400, i.e. 24h)

#### Testing the user flow

```sh
pnpm -w ops localnet-1.2 audit rewards deposit ALICE 100
sleep 60
pnpm -w ops localnet-1.2 audit rewards epoch 50
pnpm -w ops localnet-1.2 audit rewards claim ALICE
pnpm -w ops localnet-1.2 audit rewards withdraw ALICE 100
```

#### Testing the migration flow

```sh
# TODO
```

## Conceptual overview

Sienna Rewards distributes SIENNA tokens to users
who provide liquidity for Sienna Swap.

* The amount of liquidity that users may provide is
  unbounded, while the amount of rewards is pre-defined
  as part of the TGE budget in `/settings/schedule.json`.

* Therefore, an algorithm must be devised to split that
  budget among users in proportion to their liquidity
  contribution over time.

The main thing to understand about this algorithm is
that it works primarily in **event time**:

* The continously shifting distribution of rewards among users
  is updated upon each transaction, based on the elapsed linear time
  since the previous transaction.

* Time does not pass in absence of user activity; status queries
  require the current time to be provided by the user. If the time
  provided by the user is before the time of last update, the query
  fails.

## Control flows

### User flow

![](./doc/user_flow.png)

When a user provides tokens `XXX` and `YYY` to a
liquidity pool, a LP token `LP_XXX_YYY` is minted
to that user.

* The user can then deposit the LP token
  into the reward pool via the transaction:

```json
{"rewards":{"deposit":{"amount":"1"}}}
```

* This causes the reward pool to transfer 1 unit of LP_TOKEN
  from the user's balance to itself.

* From this point on, `volume` begins to accumulate:

```
user volume = user stake * time elapsed
pool volume = sum of all user stakes * time elapsed
```

After a configurable `bonding` period, the user
is eligible to `claim` the rewards:

```json
{"rewards":{"claim":{}}}
```

* If this user was the only one who staked tokens
  during that period, they get the full rewards `budget`
  for that period.

* If multiple users stake tokens, the proportion of
  the rewards that they earn is computed using the
  following formula:

```
  user volume
/ pool volume that has accumulated since T0
* reward budget that has vested since T0
```

T0 is defined as follows:

* When the user deposits, T0 is set to
  **the start of the current epoch**.

* When the user claims, T0 is reset to
  **the current time**.

When the user leaves the pool by withdrawing all LP tokens,
the state of the user is reset.

* If the user leaves the pool after the bonding period,
  rewards are auto-claimed.

* If the user leaves the pool before the bonding period,
  their contribution is reset and no rewards are claimed.

### Epoch flow

![](./doc/funding_flow.png)

Rewards are vested to the pool via the RPT contract
(`/contracts/rpt`) daily. This increments an internal counter
referred to as the **epoch clock**.

* The epoch clock is incremented by the same periodic job
  that calls `{"vest"{}}` on the RPT contract.

* Unclaimed reward budget can be manually retrieved
  and more epochs can be launched after the end of the
  rewards program if it is so desired.

### Migration flow

To migrate:

1. Deploy next version of contract
2. Enable migration to new contract in old contract
3. Enable migration from old contract in new contract
4. Call ClosePool on old contract
5. Users can now either:
  - Call `{"immigration":{"request_migration":{...}}}`
    **on the new contract** to migrate their stake from the old contract
  - Call any other method **on the old contract** to withdraw their stake
    and claim any remaining rewards.

![](./doc/migration_flow.png)

### Closing a reward pool

* A reward pool can be closed by sending it
  `{"close_pool":{"message":"Here the admin should provide info on why the pool was closed."}}`.

  * If upgrading a pool, please write the message in this format:
    `Moved to secret1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx (because...)`.

  * A closed reward pool returns each user's LP tokens
    the next time the user interacts with the pool.
    No more locking is allowed.

  * For a closed pool, time stops (liquidity shares stop changing,
    even though sending more SIENNA to the pool will allocate
    more rewards according to current liquidity shares).

  * Eligible users are able to claim rewards
    from a closed pool one last time.
    Afterwards, their LP tokens will be returned
    and their liquidity share reset to 0.
## Governance
See [README](/contracts/amm/rewards/gov/README.md)