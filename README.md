<div align="center">

[![](/logo.svg)](https://sienna.network/)

[![Twitter Follow](https://img.shields.io/twitter/follow/sienna_network?style=plastic&logo=twitter)](https://twitter.com/sienna_network)

[![Coverage Status](https://coveralls.io/repos/github/SiennaNetwork/sienna/badge.svg?branch=dev&t=s6kRdI)](https://coveralls.io/github/SiennaNetwork/sienna?branch=dev)
[![Contributions welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg?style=flat)](CONTRIBUTING.md)
[![Code style](https://img.shields.io/badge/code%20style-open--minded-%239013fe)](CONTRIBUTING.md#coding-style)

</div>

## Mainnet addresses

MOVED: See [receipts/README.md](./receipts/README.md#mainnet-addresses) for the up-to-date
mainnet addresses of all production contracts.

## Development quickstart

```sh
git clone git@git.sienna.network:siennanetwork/contracts.git sienna-contracts
cd sienna-contracts
git submodule update --init --recursive
nix-shell # optional
pnpm i
pnpm -w dev build
pnpm -w ops localnet-1.2 deploy all
```

The smart contracts are written in Rust, targeting SecretNetwork's fork of `cosmwasm-std 0.10`
on `wasm32-unknown-unknown`.

See [scripts/Dev.ts.md](./scripts/Dev.ts.md)
and [scripts/Ops.ts.md](./scripts/Ops.ts.md)
for workflow commands.

See also:
* **[Git submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules)** documentation,
  and most importantly the `git submodule update --init --recursive` command.
* **[Pnpm](https://pnpm.io/)** package manager, and most importantly
  its [Workspaces feature](https://pnpm.io/workspaces).

## Repository content

* [artifacts](./artifacts) contains the compiled smart contracts (gitignored)
  and their checksums (not gitignored).

* [benchmarks](./benchmarks) contains gas benchmarks and demos.

* [contracts](./contracts) contains the Rust source code of the smart contracts.

* [docs](./docs) contains various project documentation.

* [experiments](./experiments) is a playground for experimental solutions

* [frontends](./frontends) contains clients for the smart contracts, written in JS/TS.
  Some of them are transcluded as git submodules pointing to other repos.

* [libraries](./libraries) contains Rust libraries used by one or more smart contracts.
  * [fadroma-next](./libraries/fadroma-next) is a Git submodule pointing to the top of
    the Fadroma deployment framework, which takes care of building and uploading the
    contracts behind the scenes.

* [receipts](./receipts) contains the responses to upload and init transactions
  performed by the framework, grouped by chain ID. This lets you keep track of uploaded
  contracts.

* [scripts](./scripts) contains utility scripts pertaining to the whole repo.
  * Of those, [Dev.ts.md](./scripts/Dev.ts.md) and
    [Ops.ts.md](./scripts/Ops.ts.md) contain the entrypoints
    for the main workflow commands `pnpm -w dev` and `pnpm -w ops`.

* [settings](./settings) contains the values of configurable properties for each
  smart contract, again grouped by chain ID. This is a NPM module that can be imported
  by the deploy scripts to access the settings for deploying to a specific chain
  (as testnet configuration may need to systematically differ from mainnet).

## Project phases

### TGE/Vesting

Consists of:
* `contracts/snip20-sienna`
* `contracts/mgmt`
* `contracts/rpt`

#### Usage

* To claim funds from MGMT, send it `{"claim":{}}`.
* To make RPT send funds to the reward pools, send it `{"vest":{}}`

#### Configuration

* MGMT can be reconfigured by its admin after deployment
  as long as it hasn't been launched yet.

* RPT can be freely reconfigured by its admin
  as long as the budget adds up to the original amount (2500 SIENNA).

#### Compiling from source

```sh
pnpm -w dev build tge
```

#### Running tests

```sh
cargo test -p snip20-sienna
cargo test -p mgmt
cargo test -p rpt
```

### Swap/AMM

Consists of:
* `contracts/factory`
* `contracts/exchange`
* `contracts/ido`
* `contracts/launchpad`
* `contracts/rewards`
* `contracts/router`
* `libraries/amm-shared`

Refer to [this diagram](./docs/Sienna.drawio.png) for the architectural overview.

#### Compiling from source

```sh
pnpm -w dev build amm
```

#### Running tests

```sh
cargo test -p factory
cargo test -p exchange
cargo test -p ido
cargo test -p launchpad
cargo test -p sienna-rewards
cargo test -p sienna-router
```

#### Configuring the factory

TODO

#### Configuring the reward pools

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

### Sienna Lend

TODO

## Building the code

### Sienna TGE

TODO

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
pnpm -w ops localnet-1.2 deploy all
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

### Sienna Rewards

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

- Running the project:
  - [Clone the code](#obtaining-the-code)
  - [Development environment (nix-shell)](#entering-the-development-environment)
  - [Installing dependencies](#installing-dependencies)
  - [Compile the code](#building-the-code)
  - [Run Tests](#running-the-tests)
  - [Deployment](#deployment)
    - [Post deployment configuration](#post-deployment-configuration)
  - [Usage](#usage)
  - [Extras](#extras)
- [Mainnet addresses](#mainnet-addresses)
- [Architecture](#architecture-overview)
- [Troubleshooting](#troubleshooting)
