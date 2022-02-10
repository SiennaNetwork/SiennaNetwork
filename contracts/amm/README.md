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

## Upgrade a production deployment

```sh
export FADROMA_CHAIN=secret-4
export SIENNA_OLD_AMM=v1
export SIENNA_NEW_AMM=v2
export SIENNA_OLD_REWARDS=v2
export SIENNA_NEW_REWARDS=v3
export SCRT_AGENT_ADDRESS='...'
export SCRT_AGENT_MNEMONIC='...'
# sanity check
pnpm ops status
# from deployer address:
pnpm ops deploy amm v2
pnpm ops deploy rewards v3
# generate multisig transactions:
pnpm ops generate amm v1 disable
pnpm ops generate rewards v2 close-all
pnpm ops generate rpt reroute rewards v2=30:v3=70
# now, sign and broadcast the above 3 txs from the admin address;
# then, from any address:
pnpm ops rpt vest
```

## References
- https://github.com/enigmampc/SecretSwap
- https://github.com/terraswap/terraswap
- https://github.com/Uniswap/uniswap-v1
