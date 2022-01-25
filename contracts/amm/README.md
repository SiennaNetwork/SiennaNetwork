# Sienna Swap/AMM (Automated Market Maker)

This is an Automated Marked Maker Decentralized Exchange written in Rust (CosmWasm)
inspired by Uniswap Protocol.

## Actors

### Trader
A trader can exchange their token for another token through SiennaSwap using the price
determined by the liquidity pool ratio.

### Liquidity Provider
Must deposit an equivalent value of both tokens. This increases liquidity for the
corresponding pair market while maintaining the pool price.

## Contents

* `contracts/amm/factory` - **Exchange factory**.
  * Used to create exchange contracts between different tokens.
  * Can create only one exchange per token pair.
  * Stores all existing exchanges created.
* `contracts/amm/exchange` - **Exchange pair**. Exchange contracts are automated market makers
  between a token pair. These can be either SCRT or a SNIP20 compliant token.
* `contracts/amm/amm-snip20`
* `contracts/amm/ido`
* `contracts/amm/launchpad`
* `contracts/amm/rewards`
* `contracts/amm/router`
* `libraries/amm-shared`

Refer to [this diagram](./docs/amm.png) for the architectural overview.

## Run tests

```sh
cargo test -p factory
cargo test -p exchange
cargo test -p ido
cargo test -p launchpad
cargo test -p sienna-rewards
cargo test -p sienna-router
```

## Compile for production

```sh
pnpm -w dev build amm
```

## Configure

### Configuring the factory

### Configuring the reward pools

* A reward pool can be closed by sending it
  `{"close_pool":{"message":"Here the admin should provide info on why the pool was closed."}}`.

  * If upgrading a pool, please write the message in this format:
    `Moved to secret1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx (because...)`.

  * A closed reward pool returns each user's LP tokens
    the next time the user interacts with the pool.
    No more locking is allowed, and time stops
    (this means liquidity shares will stop changing,
    even though sending more SIENNA to the pool will allocate
    more rewards according to current liquidity shares).
    Eligible users are able to claim rewards
    from a closed pool one last time.
    Afterwards, their LP tokens will be returned
    and their liquidity share reset to 0.

## References
- https://github.com/enigmampc/SecretSwap
- https://github.com/terraswap/terraswap
- https://github.com/Uniswap/uniswap-v1
