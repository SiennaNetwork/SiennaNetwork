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
import { deployRewards } from '@sienna/amm'
commands['deploy']['rewards-side-by-side'] = async () => {
  const {chain, admin} = await init(process.env.CHAIN_NAME)
  if (chain.isMainnet) {
    console.log('This command is not intended for mainnet.')
    process.exit(1)
  }
  if (!chain.deployments.active) {
    console.log('This command requires an active deployment.')
    process.exit(1)
  }
  chain.deployments.printActive()
  const { name: prefix } = chain.deployments.active
  const options = { chain, admin, prefix }
  const v2Suffix = `@v2+${timestamp()}`
  const v3Suffix = `@v3+${timestamp()}`
  const v2 = await deployRewards('v2', { ...options, suffix: v2Suffix, split: 0.5, ref: 'rewards-2.1.2' })
  const v3 = await deployRewards('v3', { ...options, suffix: v3Suffix, split: 0.5, ref: 'HEAD' })
  const rptConfig = [
    ...v2.rptConfig,
    ...v3.rptConfig
  ]
  const RPT = chain.deployments.active.getContract(RPTContract, 'SiennaRPT', admin)
  await RPT.configure(rptConfig)
  console.log({rptConfig})
  console.table([
    ...v2.deployedContracts,
    ...v3.deployedContracts
  ].reduce((table, contract)=>{
    table[contract.init.label] = {
      address:  contract.init.address,
      codeId:   contract.blob.codeId,
      codeHash: contract.blob.codeHash
    }
    return table
  }, {}))
}

import { deployLegacyFactory } from '@sienna/amm'
commands['deploy']['legacy-factory'] = async () => {
  const {chain, admin} = await init(process.env.CHAIN_NAME)
  if (chain.isMainnet) {
    console.log('This command is not intended for mainnet.')
    process.exit(1)
  }
  if (!chain.deployments.active) {
    console.log('This command requires an active deployment.')
    process.exit(1)
  }
  chain.deployments.printActive()
  await deployLegacyFactory(chain, admin)
}
```

## Perform an upgrade

### Upgrading the reward pools in a deployment

This command closes a specified reward pool in the currently selected deployment
(see [Select the active deployment](#select-the-active-deployment)) and deploys a new one
with the latest version of the code.

```typescript
import { replaceRewardPool, printRewardsContracts } from '@sienna/amm'
commands['upgrade'] = {

  async ['rewards'] (id?: string) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    if (id) {
      await replaceRewardPool(chain, admin, id)
    } else {
      printRewardsContracts(chain)
    }
  }

}
```

## Helper commands for auditing the contract logic

```typescript
import { rewardsAudit } from '@sienna/amm'
commands['audit'] = {

  rewards: rewardsAudit

}
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
