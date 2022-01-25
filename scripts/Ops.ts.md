# Sienna Deployment Procedures

```typescript
import Fadroma, { bold, timestamp } from '@hackbg/fadroma'
import from '@hackbg/fadroma'
```

## Listing supported networks

All the commands below must be prefixed with a chain ID, e.g. `pnpm -w ops $CHAIN $COMMAND`.
To list the available chains, run `pnpm -w ops` with no parameters.

## Reset the localnet

Commands that spawn localnets (such as benchmarks and integration tests)
will do their best to clean up after themselves. However, if you need to
reset the localnet manually, the `pnpm -w ops $LOCALNET reset` command
(where `$LOCALNET` is `localnet-1.0` or `localnet-1.2`) will stop the
currently running localnet container, and will delete the localnet data under `/receipts`.

```typescript
Fadroma.command('reset', async ({ chain, admin }) => {
  if (!chain.node) {
    throw new Error(`${bold(process.env.CHAIN_NAME)}: not a localnet`)
  }
  return chain.node.terminate()
})
```

## Select the active deployment

**FIXME**: In the code, deployments are referred to as "instances", which is less specific.

```typescript
Fadroma.command('select', async ({ chain, admin, args: [ id ] }) => {
  const {chain} = await init(process.env.CHAIN_NAME)
  const list = chain.deployments.list()
  if (list.length < 1) {
    console.log('\nNo known deployments.')
  }
  if (id) {
    await chain.deployments.select(id)
  } else if (list.length > 0) {
    console.log(`\nKnown deployments:`)
    for (let instance of chain.deployments.list()) {
      if (instance === chain.deployments.active.name) instance = bold(instance)
      console.log(`  ${instance}`)
    }
  }
  chain.deployments.printActive()
})
```

## Deploy contracts

### Deploy all contracts

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
Fadroma.command('deploy all',
  deployVesting,
  deploySwap,
  ({chain})=>chain.deployments.printActive())
```

### Deploy the TGE

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
import { deployVesting } from '@sienna/tge'
Fadroma.command('deploy vesting',
  deployVesting)
```

### Deploy the AMM

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

```typescript
import { deploySwap } from '@sienna/amm'
Fadroma.command('deploy swap',
  deploySwap)
```

### Deploying Rewards v2 and v3 side-by-side

Used to test the migration from v2 to v3 pools.

```typescript
import { deployRewardsSideBySide } from '@sienna/amm'
Fadroma.command('deploy rewards-side-by-side',
  deployRewardsSideBySide)
```

### Deploying a v1 factory

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap to which it adds a Factory instance
built from `main`.

```typescript
import { deployLegacyFactory } from '@sienna/amm'
Fadroma.command('deploy legacy-factory',
  deployLegacyFactory)
```

## Upgrades and migrations

### Migrating to `@sienna/factory v2.0.0` + `@sienna/rewards v3.0.0`

```typescript
import { migrateFactoryAndRewards } from '@sienna/amm'
Fadroma.command('migrate factory-and-rewards',
  migrateFactoryAndRewards)
```

### Replacing a single reward pool in a deployment with an updated version

This command closes a specified reward pool in the currently selected deployment
(see [Select the active deployment](#select-the-active-deployment)) and deploys a new one
with the latest version of the code.

```typescript
import { replaceRewardPool, printRewardsContracts } from '@sienna/amm'
Fadroma.command('migrate reward-pool',
  async ({ chain, admin, args: [ id ] }) => {
    if (id) {
      await replaceRewardPool(chain, admin, id)
    } else {
      printRewardsContracts(chain)
    }
  })
```

## Helper commands for auditing the contract logic

This spins up a rewards contract on localnet and lets you interact with it.

```typescript
import { rewardsAudit } from '@sienna/amm'
Fadroma.command('audit rewards',
  rewardsAudit)
```

## Entry point

```typescript
export default Fadroma.module(import.meta.url)
```
