# Sienna Deployment Procedures

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
(for example, see [`needsDeployment`](#needsdeployment)).

## Chains

The active [`Chain`](https://github.com/hackbg/fadroma/blob/22.01/packages/ops/Chain.ts)
is selected via the `FADROMA_CHAIN` environment variable.

Run this script with `FADROMA_CHAIN` set to an empty value, to list possible values.

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
such contracts is called a `Deployment`.

```typescript
import { createNewDeployment, needsDeployment, selectDeployment } from '@hackbg/fadroma'
Fadroma.command('status', needsDeployment)
Fadroma.command('select', selectDeployment)
Fadroma.command('deploy new', createNewDeployment)
```

### needsDeployment

`needsDeployment` acts as a context modifier: it populates the
`deployment` and `prefix` arguments to subsequent commands -

## Contracts

### Making a new full deployment

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
import { deployTGE } from '@sienna/tge'
import { deployAMM, deployRewards, upgradeAMM, upgradeRewards } from '@sienna/amm'
Fadroma.command('deploy all',
  deployTGE,
  deployAMM.v1,
  deployRewards.v2,
  needsDeployment,
  upgradeAMM.v1_to_v2,
  upgradeRewards.v2_to_v3,
  needsDeployment)
```

### Deploy just the TGE

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
Fadroma.command('deploy tge', needsDeployment, deployTGE)
```

### Add the AMM and Rewards to the TGE

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

```typescript
Fadroma.command('deploy amm', needsDeployment, deployAMM.v2)
```

### Deploying Rewards v2 and v3 side-by-side

Used to test the migration from v2 to v3 pools.

```typescript
import { deployRewardsSideBySide } from '@sienna/amm'
Fadroma.command('deploy rewards v2', needsDeployment, deployRewards.v2)
Fadroma.command('deploy rewards v3', needsDeployment, deployRewards.v3)
Fadroma.command('deploy rewards v2_and_v3', needsDeployment, deployRewards.v2_and_v3)
```

### Deploying a v1 factory

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap to which it adds a Factory instance
built from `main`.

```typescript
import { deployAMMFactory } from '@sienna/amm'
Fadroma.command('deploy factory v1', needsDeployment, deployAMMFactory.v1)
```

## Upgrades and migrations

### Migrating to `@sienna/factory v2.0.0` + `@sienna/rewards v3.0.0`

```typescript
import { upgradeFactoryAndRewards } from '@sienna/amm'
Fadroma.command('upgrade amm v1_to_v2', needsDeployment, upgradeAMM.v1_to_v2)
Fadroma.command('upgrade rewards v2_to_v3', needsDeployment, upgradeRewards.v2_to_v3)
```

## Helper commands for auditing the contract logic

This spins up a rewards contract on localnet and lets you interact with it.

```typescript
import { rewardsAudit } from '@sienna/amm'
Fadroma.command('audit rewards', rewardsAudit)
```

## Entry point

```typescript
export default Fadroma.module(import.meta.url)
```
