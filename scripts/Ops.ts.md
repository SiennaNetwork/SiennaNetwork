# Sienna Deployment Procedures

```typescript
import { bold, timestamp, entrypoint } from '@hackbg/tools'
import process from 'process'
import { init } from '@fadroma/scrt'
```

## Commands

The following procedures are executed by the [Komandi](https://github.com/hackbg/fadroma/tree/21.12/packages/komandi)
library based on the command line arguments (see [Entry point](#entry-point)). Or, you can
use them from another script by importing this module.

```typescript
const commands = {}
export default commands
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
commands['reset'] = async function reset () {
  const {chain} = await init(process.env.CHAIN_NAME)
  if (!chain.node) {
    throw new Error(`${bold(process.env.CHAIN_NAME)}: not a localnet`)
  }
  return chain.node.terminate()
}
```

## Select the active deployment

**FIXME**: In the code, deployments are referred to as "instances", which is less specific.

```typescript
commands['select'] = async function select (id?: string) {
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
}
```

## Deploy contracts

```typescript
commands['deploy'] = {}
```

### Deploy all contracts

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
commands['deploy']['all'] = async function () {
  const {chain, admin} = await init(process.env.CHAIN_NAME)
  const prefix = timestamp()
  const vesting = await deployVesting({prefix, chain, admin})
  await chain.deployments.select(vesting.prefix)
  await deploySwap(vesting)
  chain.deployments.printActive()
}
```

### Deploy the TGE

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
import { deployVesting } from '@sienna/tge'
commands['deploy']['vesting'] = async function () {
  const {chain, admin} = await init(process.env.CHAIN_NAME)
  const prefix = timestamp()
  const vesting = await deployVesting({prefix, chain, admin})
  await chain.deployments.select(vesting.prefix)
  chain.deployments.printActive()
}
```

### Deploy the AMM

This command adds the contracts for Sienna Swap to the currently selected deployment
(see [Select the active deployment](#select-the-active-deployment)).

```typescript
import { deploySwap } from '@sienna/amm'
commands['deploy']['swap'] = async () => {
  const {chain, admin} = await init(process.env.CHAIN_NAME)
  if (!chain.deployments.active) await commands.deploy.vesting()
  const { name: prefix } = chain.deployments.active
  await deploySwap({ chain, admin, prefix })
  chain.deployments.printActive()
}
```

### Deploying Rewards v2 and v3 side-by-side

Prototype of future migration procedures.

```typescript
import { deployRewardsSideBySide } from '@sienna/amm'
commands['deploy']['rewards-side-by-side'] = async () => {
  const {chain, admin} = await init(process.env.CHAIN_NAME, {
    notOnMainnet:          true,
    needsActiveDeployment: true
  })
  await deployRewardsSideBySide(chain, admin)
}

import { deployLegacyFactory } from '@sienna/amm'
commands['deploy']['legacy-factory'] = async () => {
  const {chain, admin} = await init(process.env.CHAIN_NAME, {
    notOnMainnet:          true,
    needsActiveDeployment: true
  })
  await deployLegacyFactory(chain, admin)
}
```

## Upgrades and migrations

```typescript
commands['migrate'] = {}
```

### Migrating to `@sienna/amm v2.0.0` + `@sienna/rewards v3.0.0`
```typescript
import { migrateFactoryAndRewards } from '@sienna/amm'
commands['migrate']['factory-and-rewards'] = async function (id?: string) {
  const {chain, admin} = await init(process.env.CHAIN_NAME, {
    notOnMainnet:          true,
    needsActiveDeployment: true
  })
  await migrateFactoryAndRewards(chain, admin)
}
```

### Replacing a single reward pool in a deployment with an updated version

This command closes a specified reward pool in the currently selected deployment
(see [Select the active deployment](#select-the-active-deployment)) and deploys a new one
with the latest version of the code.

```typescript
import { replaceRewardPool, printRewardsContracts } from '@sienna/amm'
commands['migrate']['rewards'] = async function (id?: string) {
  const {chain, admin} = await init(process.env.CHAIN_NAME)
  if (id) {
    await replaceRewardPool(chain, admin, id)
  } else {
    printRewardsContracts(chain)
  }
}
```

## Helper commands for auditing the contract logic

This spins up a rewards contract on localnet and lets you interact with it.

```typescript
import { rewardsAudit } from '@sienna/amm'
commands['audit'] = {}
commands['audit']['rewards'] = rewardsAudit
```

## Entry point

```typescript
import { init } from '@fadroma/scrt'
import runCommands from '@hackbg/komandi'
Error.stackTraceLimit = Infinity
entrypoint(import.meta.url, main)
export async function main (
  [chainName, ...words]: Array<string>
) {

  // FIXME: a better way to pass the chain name
  // (reintroduce context object, minimally)
  process.env.CHAIN_NAME = chainName

  return await runCommands.default(
    commands,
    words,
    async (command: any) => {
      const { chain } = await init(chainName)
      chain.printIdentities()
      chain.deployments.printActive()
      console.log(`\nAvailable commands:`)
      for (const key of Object.keys(command)) {
        console.log(`  ${bold(key)}`)
      }
    }
  )
}
```
