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

### Making a new full deployment

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
import { createNewDeployment, needsActiveDeployment } from '@hackbg/fadroma'
import { deployTGE } from '@sienna/tge'
import { deployAMM, deployRewards, upgradeAMM, upgradeRewards } from '@sienna/amm'
Fadroma.command('deploy all',
  createNewDeployment,
  deployTGE,
  deployAMM.v1,
  deployRewards.v2,
  upgradeAMM.v1_to_v2,
  upgradeRewards.v2_to_v3,
  needsActiveDeployment)
```

### Deploy just the TGE

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
import { deployTGE } from '@sienna/tge'
Fadroma.command('deploy tge',
  createNewDeployment,
  deployTGE)
```

### Add the AMM and Rewards to the TGE

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

```typescript
import { deployAMM } from '@sienna/amm'
Fadroma.command('deploy amm',
  needsActiveDeployment,
  deployAMM.v2)
```

### Deploying Rewards v2 and v3 side-by-side

Used to test the migration from v2 to v3 pools.

```typescript
import { deployRewardsSideBySide } from '@sienna/amm'
Fadroma.command('deploy rewards v2',
  needsActiveDeployment,
  deployRewards.v2)
Fadroma.command('deploy rewards v3',
  needsActiveDeployment,
  deployRewards.v3)
Fadroma.command('deploy rewards v2_and_v3',
  needsActiveDeployment,
  deployRewards.v2_and_v3)
```

### Deploying a v1 factory

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap to which it adds a Factory instance
built from `main`.

```typescript
import { deployAMMFactory } from '@sienna/amm'
Fadroma.command('deploy factory v1',
  needsActiveDeployment,
  deployAMMFactory.v1)
```

## Upgrades and migrations

### Migrating to `@sienna/factory v2.0.0` + `@sienna/rewards v3.0.0`

```typescript
import { upgradeFactoryAndRewards } from '@sienna/amm'
Fadroma.command('upgrade amm v1_to_v2',
  needsActiveDeployment,
  upgradeAMM.v1_to_v2)
```

### Replacing a single reward pool in a deployment with an updated version

This command closes a specified reward pool in the currently selected deployment
(see [Select the active deployment](#select-the-active-deployment)) and deploys a new one
with the latest version of the code.

```typescript
import { printRewardsContracts } from '@sienna/amm'
Fadroma.command('upgrade reward-pool',
  needsActiveDeployment,
  () => 'vacatin')
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
