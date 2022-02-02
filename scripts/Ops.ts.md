# Sienna Deployments Procedures

```typescript
import Fadroma, { bold, timestamp, Console } from '@hackbg/fadroma'
const console = new Console('@sienna/ops')
```

## How commands work

TODO: Literate doc in Fadroma instead the following haphazard explanation:

Fadroma commands are a match between of a series of keywords
(represented by a space-separated string)
and a series of [stages](https://github.com/hackbg/fadroma/blob/22.01/packages/ops/index.ts)
(represented by async functions)
that are executed in sequence with a common state object -
the [`MigrationContext`](https://github.com/hackbg/fadroma/blob/22.01/packages/ops/index.ts),
into which the values returned by each procedure can also be added
(for example, see [`Deployments.activate`](#needsdeployment)).

## Chains

The active [`Chain`](https://github.com/hackbg/fadroma/blob/22.01/packages/ops/Chain.ts)
is selected via the `FADROMA_CHAIN` environment variable.
You can set it in a `.env` file in the root of the repo.

Run this script with `FADROMA_CHAIN` set to an empty value,
to list possible values.

### Receipts

Results of uploads and inits are stored in `receipts/*/{deployments,uploads}`.
These are used to keep track of deployed contracts.
See [`../receipts`](../receipts).

### Reset localnet

Commands that spawn localnets (such as benchmarks and integration tests)
will do their best to clean up after themselves. However, if you need to
reset the localnet manually, the `pnpm -w ops reset` command will stop the
currently running localnet container, and will delete the localnet data under `/receipts`.

```typescript
Fadroma.command('reset', async ({ chain }) => {
  if (chain.node) {
    await chain.node.terminate()
  } else {
    console.warn(bold(process.env.FADROMA_CHAIN), 'not a localnet')
  }
})
```

## Deployments

The Sienna platform consists of multiple smart contracts that
depend on each other's existence and configuration. A group of
such contracts is called a `Deployments`.

```typescript
import { Deployments } from '@hackbg/fadroma'
Fadroma.command('status', Deployments.status)
Fadroma.command('select', Deployments.select)
Fadroma.command('deploy new', Deployments.new)
```

### Deployments.activate

`Deployments.activate` is a command step that acts as a context modifier:
the `deployment` and `prefix` arguments for subsequent steps are taken
from its return value by the mechanics behind `Fadroma.command`.

### Deployments.new

`Deployments.new` works similarly to `Deployments.activate`, but
creates a new empty deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.
This is how you start from a clean slate.

## Deploying contracts

### Deploying Jan 2022 state

```typescript
import { deployTGE } from '@sienna/tge'
import { deployAMM, deployRewards } from '@sienna/amm'
Fadroma.command('deploy legacy',
  Deployments.new,
  deployTGE,
  Deployments.status,
  deployAMM.v1,
  Deployments.status,
  deployRewards.v2,
  Deployments.status)
```

### Upgrading legacy to latest

```typescript
import { upgradeAMM } from '@sienna/amm'
Fadroma.command('upgrade amm v1_to_v2',
  Deployments.activate,
  upgradeAMM.v1_to_v2)

import { upgradeRewards } from '@sienna/amm'
Fadroma.command('upgrade rewards v2_to_v3',
  Deployments.activate,
  upgradeRewards.v2_to_v3)
```

### Full up-to-date deployment

Note that we go through the steps for the legacy deployment
before upgrading it to the latest version. Deploy of latest code
without migrations is currently discouraged due to implicit
temporal dependencies in contracts.

```typescript
Fadroma.command('deploy all',
  Deployments.new,
  deployTGE,
  deployAMM.v1,
  deployRewards.v2,
  Deployments.status,
  upgradeAMM.v1_to_v2,
  upgradeRewards.v2_to_v3,
  Deployments.status)
```

### Deploy just the TGE

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
Fadroma.command('deploy tge',
  Deployments.activate,
  deployTGE)
```

### Add the AMM and Rewards to the TGE

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

```typescript
Fadroma.command('deploy amm',
  Deployments.activate,
  deployAMM.v2)
```

### Deploying Rewards v2 and v3 side-by-side

Used to test the migration from v2 to v3 pools.

```typescript
Fadroma.command('deploy rewards v2',
  Deployments.activate,
  deployRewards.v2)

Fadroma.command('deploy rewards v3',
  Deployments.activate,
  deployRewards.v3)

Fadroma.command('deploy rewards v2_and_v3',
  Deployments.activate,
  deployRewards.v2_and_v3)
```

### Deploying a v1 factory

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap to which it adds a Factory instance
built from `main`.

```typescript
import { deployAMMFactory } from '@sienna/amm'
Fadroma.command('deploy factory v1',
  Deployments.activate,
  deployAMMFactory.v1)
```

## Helper commands for auditing the contract logic

This spins up a rewards contract on localnet and lets you interact with it.

```typescript
import { rewardsAudit } from '@sienna/amm'
Fadroma.command('audit rewards', rewardsAudit)
```

## Import receipts in old format

This function addes the minimum of
`{ codeId, codeHash, initTx: contractAddress }`
to AMM and Rewards pool instantiation receipts
from the mainnet deploy that were previously stored
in a non-compatible format.

```typescript
Fadroma.command('fix receipts',
  Deployments.activate,
  ({ agent, deployment }) => {
    for (const [name, data] of Object.entries(deployment.receipts)) {
      console.log(name, data)
    }
  })
```

## Entry point

```typescript
export default Fadroma.module(import.meta.url)
```
