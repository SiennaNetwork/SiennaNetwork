# SIENNA

## Mainnet addresses

|Contract  |Address<br>Code hash|
|:---------|:-------------------|
|**SIENNA**<br>The main [SNIP20](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md) governance token.<br>[SIENNA on CoinMarketCap](https://coinmarketcap.com/currencies/sienna/)|[**`secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4`**](https://secretnodes.com/secret/chains/secret-3/accounts/secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4)<br>`c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084`|
|**SIENNA on BSC**<br>Wrapper on Binance Smart Chain<br>[SIENNA BSC token tracker](https://www.bscscan.com/token/0x130F6E4d338BFD8304F5342D759ABE5C6Bd7bA9b)|[**`0x130F6E4d338BFD8304F5342D759ABE5C6Bd7bA9b`**](https://www.bscscan.com/address/0x130F6E4d338BFD8304F5342D759ABE5C6Bd7bA9b)|N/A|
|**wSIENNA on ETH**<br>Wrapper on Ethereum<br>[wSIENNA on CoinMarketCap](https://coinmarketcap.com/currencies/sienna-erc20/)|[**`0x9b00e6E8D787b13756eb919786c9745054DB64f9`**](https://ethplorer.io/address/0x9b00e6e8d787b13756eb919786c9745054db64f9#chart=candlestick)|
|**MGMT**<br>The [main vesting contract](./contracts/mgmt) that distributes pre-defined amounts of SIENNA over time|[**`secret1kn6kvc97nvu69dqten0w9p9e95dw6d6luv3dfx`**](https://secretnodes.com/secret/chains/secret-3/accounts/secret1kn6kvc97nvu69dqten0w9p9e95dw6d6luv3dfx)<br>`b1e4c4d76a5aedd180d08d8fec99ad84ed1a8a08d6d8a32a30c8c0f9835f4fab`|
|**RPT**<br>The [remaining pool token](./contracts/rpt) distribution contract that funds the reward pools|[**`secret107j8czcysrkvxsllvhqj4mhmcegt9hx2ra3x42`**](https://secretnodes.com/secret/chains/secret-3/accounts/secret107j8czcysrkvxsllvhqj4mhmcegt9hx2ra3x42)<br>`a9bfc78d182eb8d3cbb74d4269ef1f529a607f7842d755f00fef7df13c02c5b4`|
|**Factory**<br>The [hub of Sienna Swap](./contracts/factory).|[**`secret1zvk7pvhtme6j8yw3ryv0jdtgg937w0g0ggu8yy`**](https://secretnodes.com/secret/chains/secret-3/accounts/secret1zvk7pvhtme6j8yw3ryv0jdtgg937w0g0ggu8yy)<br>`b1f8a2086c7ca3bf8a0866275885b21462829158927a2a757064ccd65a593b36`|
|**Exchanges**<br>Initial [liquidity pools](./contracts/exchange) created via the Factory|See **./artifacts/secret-3/prod/SiennaSwap_\***<br>and [settings/swapPairs-secret-3.json](./settings/swapPairs-secret-3.json)|
|**Rewards**<br>[Reward pools](./contracts/rewards) corresponding to select liquidity pools|See **./artifacts/secret-3/prod/SiennaRewards_\***<br>and [settings/rewardPairs-secret-3.json](./settings/rewardPairs-secret-3.json)|

## Contents

|Environment|Component     |TGE |Swap|Rewards|IDO|Lend|
|----|---------------------|----|----|-------|---|----|
|Rust|Smart contract(s)    |✔️   |✔️   |✔️      |✔️  | ?  |
|Rust|Unit tests           |    | ?  |✔️      | ? | ?  |
|JS  |API wrapper(s)       |✔️   |✔️   |✔️      |✔️  | ?  |
|JS  |API integration tests|✔️   | ?  |       | ? | ?  |
|JS  |Gas benchmark        |    | ?  |✔️      | ? | ?  |
|JS  |Dashboard            |    | ?  |✔️      | ? | ?  |

## Obtaining the code

This project is connected to some of its dependencies
via [Git submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules).
This means that to get everything you need to get started,
you need to clone this repo **recursively**:

```sh
git clone --recurse-submodules git@github.com:hackbg/sienna-secret-token.git
```

See [the troubleshooting section](#troubleshooting) if you forget to do that.

## Entering the development environment

The file `shell.nix` contains a reproducible
description of the development environment.
To enter the development environment,
run this in the project root:

```sh
nix-shell
```

This requires the [Nix package manager](https://nixos.org/download.html#nix-quick-install),
and contains Rust Nightly, Node.js, and PNPM.
Alternatively, you are free to bring
your own toolchain for your convenience
(and this project's portability).

## Installing dependencies

To install the dependencies of the management scripts,
run this in the project root:

```sh
pnpm i
```

## Building the code

The smart contracts are written in Rust, targeting
SecretNetwork's fork of `cosmwasm-std 0.10.*`.

### Sienna TGE

TODO

### Sienna Swap + Rewards

To obtain a production build of Sienna Rewards:

```sh
pnpm dev build rewards
```

## Running the tests

These tests cover the business logic of the contract
in a mocked out environment. They run relatively quickly,
and output any compilation errors, which makes them perfect
for iterating on contracts.

### Sienna TGE

TODO

### Sienna Swap + Rewards

In the case of Sienna Rewards, the unit tests are two-tiered:
`rewards_test.rs` tests the contract through its public API, while
`rewards_test_2.rs` tests the underlying business logic implementation
by calling the internal methods directly. To run both:

```sh
./sienna rewards test
```

## Deployment

There is some support for resuming deployments that were interrupted halfway.

### Full local deployment

```
pnpm ops localnet-1.0 deploy all
```

### Remote and multi-stage deployment

TODO

## Post-deployment configuration

After deployment the contracts should be
transferred to the master multisig account.
The CLI and API wrappers in this repo
do not support generating multisig transactions.
See [hackbg/motika](https://github.com/hackbg/motika)
for a GUI-based multisig transaction signer.

### Sienna TGE

* MGMT can be reconfigured by its admin after deployment
  as long as it hasn't been launched yet.
* RPT can be freely reconfigured by its admin
  as long as the budget adds up to the original amount (2500 SIENNA).

### Sienna Swap

TODO

### Sienna Rewards

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

## Usage

* To claim funds from MGMT, send it `{"claim":{}}`.
* To make RPT send funds to the reward pools, send it `{"vest":{}}`
* Swap: TODO
* Rewards: TODO

## Extras

### API wrappers, integration tests, and benchmarks

JS/TS modules for deploying and operating the contracts can be found
in `api/`, `ops/`, and `ops/amm-lib/`. The ones in `api/` and `ops/`
are based on Fadroma (`Contract` and `Ensemble` classes),
while the ones in `ops/amm-lib/` invoke SecretJS directly.

The API integration tests are based on [Mocha](https://mochajs.org/).

Fadroma provides a "localnet" container - an ephemeral local blockchain
that is set up and torn down between test cases.

To run the Sienna Rewards benchmark:

```sh
./sienna localnet reset
./sienna rewards benchmark
```

### Dashboard

### SNIP20

Located under `api/SNIP20.js` you'll find the wrapper for any `snip20` contract
that will expose all the required methods to call on the contract.

## Troubleshooting

If you forget `--recurse-submodules` on initial checkout,
or something goes wrong with your Git repo (both happen)
you may see this error:

```
ERR_PNPM_NO_MATCHING_VERSION_INSIDE_WORKSPACE  In libraries/fadroma-next:
No matching version found for @hackbg/ganesha@* inside the workspace
```

To fetch the missing submodules, go to the root of the repo and do this:

```sh
git submodule init
git submodule update
cd libraries/fadroma-next
git submodule init
git submodule update
```
