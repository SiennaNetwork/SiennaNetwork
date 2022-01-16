# Sienna Development Procedures

* [Building the contracts - `pnpm -w dev build`](#building-the-contracts)
  * [Building all contracts - `pnpm -w dev build all`](#building-all-contracts)
  * [Building the TGE contracts - `pnpm -w dev build tge`](#building-the-tge-contracts)
  * [Building the AMM contracts - `pnpm -w dev build amm`](#building-the-amm-contracts)
  * [Building the rewards contract - `pnpm -w dev build rewards`](#building-the-rewards-contract)
* [Running the tests - `pnpm -w dev test`](#running-the-tests)
* [Generating the JSON schema - `pnpm -w dev schema`](#generating-the-schema)
* [Running the benchmarks and demos - `pnpm -w dev bench`, `pnpm dev -w demo`](#running-the-benchmarks-and-demos)
* [Entry point](#entry-point)

The following procedures are executed by the [Komandi](https://github.com/hackbg/fadroma/tree/21.12/packages/komandi)
library based on the command line arguments (see [Entry point](#entry-point)). Or, you can
use them from another script by importing this module.

```typescript
const commands = {}
export default commands
```

Let's populate the list of commands.

## Building the contracts

These commands allow different combinations of contracts to be built for **production**.
Run `pnpm -w dev build` to list them.

The build output for each contract consists of a WASM file in [/artifacts](../artifacts)
and a checksum in [/artifacts/checksums.sha256.txt](../artifacts/checksums.sha256.txt).

```typescript
commands['build'] = {}
```

The build procedure for any CosmWasm contract is implemented in [@fadroma/ops/ContractBuild](../libraries/fadroma-next/packages/ops/ContractBuild.ts);
the [build script](https://github.com/hackbg/fadroma/tree/21.12/packages/scrt/ScrtBuild.sh)
and [build image](https://github.com/hackbg/fadroma/tree/21.12/packages/scrt/ScrtBuild.Dockerfile)
are set in [@fadroma/scrt/ScrtContract](https://github.com/hackbg/fadroma/tree/21.12/packages/scrt/ScrtContract.ts).

### Building all contracts

Run `pnpm -w dev build all` to compile all contracts defined below:

```typescript
import {
  SiennaSNIP20Contract, MGMTContract, RPTContract,
  FactoryContract, AMMContract, AMMSNIP20Contract, LPTokenContract,
  RewardsContract, SwapRouterContract,
  IDOContract, LaunchpadContract,
} from '@sienna/api'

commands['build']['all'] = () => Promise.all([
  new SiennaSNIP20Contract().build(),
  new MGMTContract().build(),
  new RPTContract().build(),
  new AMMContract().build(),
  new AMMSNIP20Contract().build(),
  new LPTokenContract().build(),
  new FactoryContract().build(),
  new RewardsContract().build(),
  new IDOContract().build(),
  new LaunchpadContract().build(),
  new SwapRouterContract().build()
])
```

### Building the TGE contracts

Run `pnpm -w dev build tge` to compile the contracts for the Token Generation Event:

```typescript
commands['build']['tge'] = () => Promise.all([
  new SiennaSNIP20Contract().build(),
  new MGMTContract().build(),
  new RPTContract().build()
])
```

### Building the AMM contracts

Run `pnpm -w dev build amm` to compile the contracts for Sienna Swap/AMM:

```typescript
commands['build']['amm'] = () => Promise.all([
  new AMMContract().build(),
  new AMMSNIP20Contract().build(),
  new LPTokenContract().build(),
  new SwapRouterContract().build(),
  new FactoryContract().build(),
  new LaunchpadContract().build(),
  new IDOContract().build(),
  new RewardsContract().build(),
  new SwapRouterContract().build()
])
```

### Building the rewards contract

Run `pnpm -w dev build rewards` to compile the contract for Sienna Rewards:

```typescript
commands['build']['rewards'] = () => Promise.all([
  new RewardsContract().build(),
])
```

#### Building legacy rewards

Run `pnpm -w dev build rewards-v2` to compile Rewards v2 from git tag `rewards-2.1.2`.

```typescript
commands['build']['rewards'] = () => Promise.all([
  new RewardsContract().build(),
])
```

## Running the tests

Use `pnpm -w dev test` to run the available JavaScript integration tests.

Use `cargo test -p $CRATE` to test individual crates, as listed in [/Cargo.toml](../Cargo.toml).

**Troubleshooting:** Tests exit before they finish? See [/contracts/router/route.test.ts.md](../contracts/router/route.test.ts.md#the-catch)
for info and a possible workaround.

```typescript
import routerClientTests from '../contracts/router/test/client.test.ts.md'
commands['test'] = {}
commands['test']['router'] = {}
commands['test']['router']['client'] = routerClientTests
commands['test']['router']['integration'] = async () => {
  const tests = await import('../contracts/router/test/integration.test.ts.md')
  await tests.allDone
}
```

## Generating the JSON schema

```typescript
import { resolve } from 'path'
import { readdirSync, readFileSync, writeFileSync } from 'fs'

import TOML              from 'toml'
import { schemaToTypes } from '@fadroma/scrt'
import { cargo }         from '@hackbg/tools'
import { abs }           from '@sienna/settings'

commands['schema'] = async () => {

  for (const dir of [
    "amm-snip20",
    "exchange",
    "factory",
    "ido",
    "launchpad",
    "lp-token",
    "mgmt",
    "rewards",
    "router",
    "rpt",
    "snip20-sienna",
  ]) {

    // Generate JSON schema
    const cargoToml = abs('contracts', dir, 'Cargo.toml')
    const {package:{name}} = TOML.parse(readFileSync(cargoToml, 'utf8'))
    cargo('run', '-p', name, '--example', 'schema')

    // Collect generated schema definitions
    const schemaDir = abs('contracts', dir, 'schema')
    const schemas = readdirSync(schemaDir)
      .filter(x=>x.endsWith('.json'))
      .map(x=>resolve(schemaDir, x))

    // Remove `For_HumanAddr` suffix from generic structs
    // This does a naive find'n' replace, not sure what it'll do for
    // types that are genericized over HumanAddr AND something else?
    for (const schema of schemas) {
      const content = readFileSync(schema, 'utf8')
      writeFileSync(schema, content.replace(/_for_HumanAddr/g, ''), 'utf8')
    }

    // Generate type definitions from JSON schema
    await schemaToTypes(...schemas)

  }

}
```

## Running the benchmarks and demos

```typescript
import { rewardsBenchmark } from '@sienna/benchmarks'

commands['bench'] = {
  rewards: rewardsBenchmark,
  ido:     notImplemented
}

commands['demo'] = {
  tge:     notImplemented,
  rewards: notImplemented
}

function notImplemented () {
  console.log(`\nThis command is on vacation. 🌴 ⛱️  🐬\n`)
  process.exit(1)
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

