<div align="center">

[![](/logo.svg)](https://sienna.network/)

[![Twitter Follow](https://img.shields.io/twitter/follow/sienna_network?style=plastic&logo=twitter)](https://twitter.com/sienna_network)

[![Coverage Status](https://coveralls.io/repos/github/SiennaNetwork/sienna/badge.svg?branch=dev&t=s6kRdI)](https://coveralls.io/github/SiennaNetwork/sienna?branch=dev)
[![Contributions welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg?style=flat)](CONTRIBUTING.md)
[![Code style](https://img.shields.io/badge/code%20style-open--minded-%239013fe)](CONTRIBUTING.md#coding-style)

</div>

## Mainnet addresses

MOVED: See [**`receipts/secret-4/deployments/prod.yml`**](./receipts/secret-4/deployments/prod.yml)
for the up-to-date mainnet addresses of all production contracts.

## Development quickstart

```sh
git clone <this-repo>
```

```sh
git submodule update --init --recursive
nix-shell # optional, or bring your own Cargo and PNPM
pnpm i
pnpm build latest
FADROMA_CHAIN=Scrt_1_2_Devnet pnpm deploy latest
```

The smart contracts are written in Rust, targeting SecretNetwork's fork of `cosmwasm-std 0.10`
on `wasm32-unknown-unknown`.

See [**`package.json`**](package.json) and [**`scripts/`**](./scripts/)
for the available workflow commands.

See also:

-   **[Git submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules)** documentation,
    and most importantly the `git submodule update --init --recursive` command.
-   The **[pnpm](https://pnpm.io/)** package manager, and most importantly
    its [Workspaces feature](https://pnpm.io/workspaces).

## Repository content

* [**artifacts**](./artifacts) contains the compiled smart contracts (gitignored)
  and their checksums (not gitignored).

* [**contracts**](./contracts) contains the Rust source code of the smart contracts,
  as well as the TypeScript code necessary to build them and interact with them.

  * [📖 tge](./contracts/tge) - **Token Generation Event (TGE)**: mints and vests a new token
    * **Contract:** [snip20-sienna](./contracts/tge/snip20-sienna) - Main SIENNA governance token
    * **Contract:** [mgmt](./contracts/tge/mgmt) - Vesting management contract
    * **Contract:** [rpt](./contracts/tge/rpt) - Remaining pool token splitter contract
    * **Script:**   [deploy.ts](./contracts/tge/deploy.ts) - TGE deployment

  * [📖 amm](./contracts/amm) - **Automated Market Maker (AMM)**: Sienna Swap and friends
    * **Contract:** [amm-snip20](./contracts/amm/amm-snip20) - Vanilla SNIP20 token usable by the AMM
    * **Contract:** [factory](./contracts/amm/factory) - Sienna Swap Factory
    * **Contract:** [exchange](./contracts/amm/exchange) - Sienna Swap Exchange
    * **Contract:** [lp-token](./contracts/amm/lp-token) - Sienna Swap LP Token
    * **Contract:** [router](./contracts/amm/router) - Sienna Swap Router
    * **Contract:** [📖 rewards](./contracts/amm/rewards) - Sienna Rewards
    * **Contract:** [launchpad](./contracts/amm/launchpas) - Sienna Launchpad
    * **Contract:** [ido](./contracts/amm/ido) - Sienna IDO
    * **Script:**   [deploy.ts](./contracts/amm/deploy.ts) - AMM deployment
    * **Script:**   [upgrade.ts](./contracts/amm/upgrade.ts) - AMM migrations

  * [📖 lend](./contracts/lend) - **Sienna Lend** - Private Lending Protocol
    * **Contract:** [market](./contracts/lend/market)
    * **Contract:** [oracle](./contracts/lend/oracle)
    * **Contract:** [overseer](./contracts/lend/overseer)
    * **Contract:** [interest_model](./contracts/lend/interest_model)

* [**deps**](./deps) contains submodules of our foundational frameworks.
  * [fadroma](./deps/fadroma) is a Git submodule pointing to the top of
    the [Fadroma deployment framework](https://github.com/hackbg/fadroma), which takes care of building and uploading the
    contracts behind the scenes.

* [**frontends**](./frontends) contains clients for the smart contracts, written in JS/TS.
  Some of them are transcluded as git submodules pointing to other repos.
  * [siennajs](./frontends/siennajs) - current client library
  * [@sienna/api](./frontends/api) - upcoming mixed deploy/client library
  * [dashboard](./frontends/dashboard) - rewards simulation dashboard
  * [reward-pools-monitor](./frontends/reward-pools-monitor) - query status of reward pools
  * [claim](./frontends/claim) - TGE claim frontend
  * [vest](./frontends/vest) - TGE vest frontend

* [**libraries**](./libraries) contains Rust libraries used by one or more smart contracts.
  * [amm-shared](./libraries/amm-shared) defines the contract API of Sienna Swap.
  * [lend-shared](./libraries/lend-shared) defines the contract API of Sienna Lend.

* [**receipts**](./receipts) contains the responses to upload and init transactions
  performed by the framework, grouped by chain ID. This lets you keep track of uploaded
  contracts.
  * [secret-4/deployments/prod](./receipts/secret-4/deployments/prod.yml) - current mainnet deployment
  * [pulsar-2/deployments/.active](./receipts/pulsar-2/deployments/.active) - current testnet deployment

* [**scripts**](./scripts) contains utility scripts pertaining to the whole repo.
  * [Build.ts.md](./scripts/Build.ts.md) - build with `pnpm build`
  * [Chain.ts.md](./scripts/Chain.ts.md)
  * [Configure.ts.md](./scripts/Configure.ts.md)
  * [Deploy.ts.md](./scripts/Deploy.ts.md) - deploy with `pnpm deploy`
  * [Deployment.ts.md](./scripts/Deployment.ts.md)
  * [Generate.ts.md](./scripts/Generate.ts.md)
  * [Live.ts.md](./scripts/Live.ts.md)
  * [Receipts.ts.md](./scripts/Receipts.ts.md)
  * [Schema.ts.md](./scripts/Schema.ts.md)
  * [Status.ts.md](./scripts/Status.ts.md)
  * [Test.ts.md](./scripts/Test.ts.md) - test with `pnpm test`
  * [Tokens.ts.md](./scripts/Tokens.ts.md)
  * [Upload.ts.md](./scripts/Upload.ts.md)

* [**settings**](./settings) contains the values of configurable properties for each
  smart contract, again grouped by chain ID. This is a NPM module that can be imported
  by the deploy scripts to access the settings for deploying to a specific chain
  (as testnet configuration may need to systematically differ from mainnet).
  * [schedule.json](./settings/schedule.json) - vesting schedule

## Post-deployment configuration

After deployment the contracts should be
transferred to the master multisig account.
The CLI and API wrappers in this repo
do not support generating multisig transactions.

See [hackbg/motika](https://github.com/hackbg/motika)
for a GUI-based multisig transaction signer.
