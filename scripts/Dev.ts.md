# Sienna Development Procedures

* [Run the tests - `pnpm -w dev test`](#run-the-tests)
* [Compile for production - `pnpm -w dev build all`](#compile-for-production)
* [Generate JSON schema - `pnpm -w dev schema`](#generate-json-schema)
* [This script's entry point - `pnpm -w dev`](#entry-point)

The following procedures are executed by the [Komandi](https://github.com/hackbg/fadroma/tree/21.12/packages/komandi)
library based on the command line arguments (see [Entry point](#entry-point)). Or, you can
use them from another script by importing this module.

```typescript
const commands = {}
export default commands
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
import { buildTge } from '@sienna/tge'
import { buildTokens, buildAmm, buildIdo, buildRewards, buildRouter } from '@sienna/amm'
import { buildLend } from '@sienna/lend'
commands['build'] = {}
commands['build']['tge']     = () => buildTge()
commands['build']['amm']     = () => buildTokens().then(buildAmm())
commands['build']['rewards'] = () => buildTokens().then(buildRewards())
commands['build']['ido']     = () => buildTokens().then(buildIdo())
commands['build']['router']  = () => buildTokens().then(buildRouter())
commands['build']['lend']    = () => buildTokens().then(buildLend())
commands['build']['all'] = () => Promise.all([
  buildTge(),
  buildTokens(),
  buildAmm(),
  buildRewards(),
  buildRouter()
  buildLend()
])
```

These commands are defined in [`@sienna/tge/build.ts`](../contracts/tge/build.ts')
and [`@sienna/amm/build.ts`](../contracts/amm/build.ts'), and use a build procedure
that is implemented in [@fadroma/ops/ContractBuild](../deps/fadroma/packages/ops/ContractBuild.ts).

The [image of the build container](https://github.com/hackbg/fadroma/tree/22.01/packages/scrt/ScrtBuild.Dockerfile)
and the [build script that runs in it](https://github.com/hackbg/fadroma/tree/22.01/packages/scrt/ScrtBuild.sh)
are set in [@hackbg/fadroma/ScrtContract](https://github.com/hackbg/fadroma/tree/22.01/packages/scrt/ScrtContract.ts).

## Generate JSON schema

The contracts have the capability to output their API schema in the form of JSON schema.
From this, we create TypeScript type definitions via `json-schema-to-typescript`.
These type definitions are imported by the client classes.

```typescript
import { generateSchema } from '@hackbg/fadroma'
import { abs } from '@sienna/settings'
commands['schema'] = () => generateSchema(abs(), [
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
  "lend/overseer"
])
```

## Tests

### Test client classes

```typescript
commands['test'] = {}

import * as API from '@sienna/api'
commands['test']['clients'] = () => {
  new API.SiennaSNIP20Contract()
  new API.MGMTContract()
  new API.RPTContract()

  new API.FactoryContract()
  new API.AMMContract()
  new API.AMMSNIP20Contract()
  new API.LPTokenContract()
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
import process from 'process'
import runCommands from '@hackbg/komandi'
import { fileURLToPath } from 'url'
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const words = process.argv.slice(2)
  runCommands.default(commands, words).then(()=>process.exit(0))
}
```

