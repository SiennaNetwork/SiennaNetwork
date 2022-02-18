# Sienna Development Procedures

* [Run the tests - `pnpm -w dev test`](#run-the-tests)
* [Compile for production - `pnpm -w dev build all`](#compile-for-production)
* [Generate JSON schema - `pnpm -w dev schema`](#generate-json-schema)
* [This script's entry point - `pnpm -w dev`](#entry-point)

The following procedures are executed by the [Komandi](https://github.com/hackbg/fadroma/tree/21.12/packages/komandi)
library based on the command line arguments (see [Entry point](#entry-point)). Or, you can
use them from another script by importing this module.

```typescript
import Fadroma, { bold, timestamp, Console } from '@hackbg/fadroma'
const console = new Console('@sienna/ops')
```

The content below populates the list of commands that are invoked with `pnpm -w dev`,
while taking the time to elaborate on what each command does and what there is to be
known about it.

## Run the tests

Use `pnpm -w dev test` to run the available JavaScript integration tests.

Use `cargo test -p $CRATE` to test individual crates, as listed in [/Cargo.toml](../Cargo.toml).

**Troubleshooting:** Tests exit before they finish? See [/contracts/router/route.test.ts.md](../contracts/router/route.test.ts.md#the-catch)
for info and a possible workaround.

```typescript
/*import routerClientTests from '../contracts/router/test/client.test.ts.md'
commands['test'] = {}
commands['test']['router'] = {}
commands['test']['router']['client'] = routerClientTests
commands['test']['router']['integration'] = async () => {
  const tests = await import('../contracts/router/test/integration.test.ts.md')
  await tests.allDone
}*/
```

## Compile for production

`pnpm -w dev build all` compiles all contracts for production.

The build output consists of two files being written to [/artifacts](../artifacts):
* `contract-name@version.wasm` (gitignored)
* `contract-name@version.wasm.sha256` (not gitignored).

Run `pnpm -w dev build all` compile to list the subsets of contracts that can be built.

```typescript
import { buildLend } from '@sienna/lend'

/* don't fight the platform, follow it! */
const parallel = (...commands) => input => Promise.all(commands.map(command=>command(input)))

import { buildTge } from '@sienna/tge'
Fadroma.command('build tge',
  buildTge)

import { buildTokens, buildAmm } from '@sienna/amm'
Fadroma.command('build amm',
  parallel(buildTokens, buildAmm))

import { buildRewards } from '@sienna/amm'
Fadroma.command('build rewards',
  parallel(buildTokens, buildRewards))

import { buildIdo } from '@sienna/amm'
Fadroma.command('build ido',
  parallel(buildTokens, buildIdo))

import { buildRouter } from '@sienna/amm'
Fadroma.command('build router',
  parallel(buildTokens, buildRouter))

import { buildLend } from '@sienna/lend'
Fadroma.command('build lend',
  parallel(buildTokens, buildLend))

Fadroma.command('build all',
  parallel(/* remain flexible */
    buildTge,
    buildTokens,
    buildAmm,
    buildRewards,
    buildRouter
    buildLend
  ))

import { buildLatestAMMAndRewards } from '@sienna/amm'
Fadroma.command('build latest amm-and-rewards',
  buildLatestAMMAndRewards
)
```

Which contracts each `build*` command builds is defined in:
* [`@sienna/tge/build.ts`](../contracts/tge/build.ts')
* [`@sienna/amm/build.ts`](../contracts/amm/build.ts')
* [`@sienna/lend/build.ts`](../contracts/tge/build.ts')

The builder procedure is implemented in [`@fadroma/ops/Build`](https://github.com/hackbg/fadroma/tree/22.01/packages/ops/Build.ts).

The [image of the build container](https://github.com/hackbg/fadroma/tree/22.01/packages/scrt/ScrtBuild.Dockerfile)
and the [build script that runs in it](https://github.com/hackbg/fadroma/tree/22.01/packages/scrt/ScrtBuild.sh)
are set in [@fadroma/scrt/Scrt](https://github.com/hackbg/fadroma/tree/22.01/packages/scrt/Scrt.ts).

### Generate JSON schema

The contracts have the capability to output their API schema in the form of JSON schema.

From this, we create TypeScript type definitions via `json-schema-to-typescript`.

These type definitions are imported by the `Contract` classes.

```typescript
import { generateSchema } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
Fadroma.command('schema', () => generateSchema(workspace, [
  "tge/mgmt",
  "tge/rpt",
  "tge/snip20-sienna",

  "amm/amm-snip20",
  "amm/exchange",
  "amm/factory",
  "amm/ido",
  "amm/launchpad",
  "amm/lp-token",
  "amm/rewards",
  "amm/router",

  "lend/interest_model",
  "lend/market",
  "lend/oracle",
  "lend/overseer",
  "lend/mock_band_oracle"
])
```

## Tests

### Smoke test of contract classes

This makes sure each client can be constructed,
and thus checks there are no "shallow" errors, e.g.
syntax errors, broken module imports/exports.

```typescript
import * as API from '@sienna/api'
Fadroma.command('test clients', () => {
  new API.SiennaSnip20Contract().client()
  new API.MGMTContract().client()
  new API.RPTContract().client()
  new API.AMMFactoryContract['v1']().client()
  new API.AMMFactoryContract['v2']().client()
  new API.AMMExchangeContract['v1']().client()
  new API.AMMExchangeContract['v2']().client()
  new API.AMMSNIP20Contract().client()
  new API.LPTokenContract().client()
  new API.RewardsContract['v2']().client()
  new API.RewardsContract['v3']().client()

  // TODO: these don't have clients yet
  new API.LaunchpadContract()
  new API.IDOContract()
  new API.InterestModelContract()
  new API.LendMarketContract()
  new API.LendOracleContract()
  new API.LendOverseerContract()
}
```

## Entry point

```typescript
export default Fadroma.module(import.meta.url)
```
