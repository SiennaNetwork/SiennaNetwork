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
such contracts is called a `Deployment`.

```typescript
import { Deployments } from '@hackbg/fadroma'
import { SiennaSNIP20Contract, MGMTContract, RPTContract } from '@sienna/api'
Fadroma.command('status',
  Deployments.activate,
  SiennaSNIP20Contract.status,
  MGMTContract.status,
  RPTContract.status)
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
import { AMMFactoryContract, RewardsContract } from '@sienna/amm'
Fadroma.command('deploy legacy',
  Deployments.new,
  deployTGE,
  Deployments.status,
  AMMFactoryContract.v1.deployAMM,
  Deployments.status,
  RewardsContract.v2.deploy,
  Deployments.status)
Fadroma.command('test legacy',
  Deployments.activate)
```

### Upgrading legacy to latest

```typescript
Fadroma.command('upgrade amm v1_to_v2',
  Deployments.activate,
  AMMFactoryContract.v1.upgradeAMM.to_v2)

Fadroma.command('upgrade rewards v2_to_v3',
  Deployments.activate,
  RewardsContract.v2.upgrade.to_v3)
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
  AMMFactoryContract.v1.deployAMM,
  RewardsContract.v2.deploy,
  Deployments.status,
  AMMFactoryContract.v1.upgradeAMM.to_v2,
  RewardsContract.v2.upgrade.to_v3,
  Deployments.status)
```

### Deploy just the TGE

This creates a new deployment under `/receipts/$CHAIN_ID/$TIMESTAMP`.

```typescript
Fadroma.command('deploy tge',
  Deployments.new,
  deployTGE)
```

### Deploy just the Lend

```typescript
import { deployLend } from "@sienna/lend"
Fadroma.command("deploy lend", Deployments.new, deployLend)
```

### Add the AMM and Rewards to the TGE

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

```typescript
Fadroma.command('deploy amm',
  Deployments.activate,
  AMMFactoryContract.v2.deployAMM)
```

### Deploying Rewards v2 and v3 side-by-side

Used to test the migration from v2 to v3 pools.

```typescript
Fadroma.command('deploy rewards v2',
  Deployments.activate,
  RewardsContract.v2.deploy)

Fadroma.command('deploy rewards v3',
  Deployments.activate,
  RewardsContract.v3.deploy)

Fadroma.command('deploy rewards v2_and_v3',
  Deployments.activate,
  RewardsContract.deploy_v2_v3)
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
import { fixReceipts } from '@sienna/receipts'
Fadroma.command('fix receipts',
  Deployments.activate,
  fixReceipts)
```

## Integration test

This is a multi-stage integration test covering the migration
from Sienna AMM v1 + Sienna Rewards v2
to Sienna AMM v2 and Sienna Rewards v3.
This involves recreating all the AMM and rewards contracts.

```typescript
import { schedule } from '@sienna/settings'
import { AMMSNIP20Contract } from '@sienna/amm'
const integrationTest = {
  setup: async function integrationTestSetup ({ chain: { isLocalnet }, agent: { address } }) {
    if (!isLocalnet) {
      throw new Error('@sienna/mgmt: This command is for localnet only.')
    }
    const scheduleMod = JSON.parse(JSON.stringify(schedule))
    console.warn('Redirecting MintingPool/LPF to admin balance. Only run this on localnet.')
    scheduleMod.pools[5].accounts[0].address = address
    console.warn('Changing RPT to vest every 10 seconds. Only run this on localnet.')
    scheduleMod.pools[5].accounts[1].interval = 10
    console.warn('Setting viewing key of agent to empty string.')
    return { schedule: scheduleMod }
  },
  claim: async function integrationTestClaim ({
    agent, deployment,
    MGMT = deployment.getThe('MGMT', new MGMTContract({agent}))
  }) {
    console.warn('Integration test: claiming from LPF')
    await MGMT.tx().claim()
  },
  getLPTokens: v => async function integrationTestGetLPTokens ({
    agent, deployment,
    FACTORY = deployment.getThe(`AMM[${v}].Factory`, new AMMFactoryContract[v]({agent}))
    SIENNA  = deployment.getThe('SIENNA',            new SiennaSNIP20Contract({agent})),
    SSCRT   = deployment.getThe('Placeholder.sSCRT', new AMMSNIP20Contract({agent, name: 'sSCRT'})),
  }) {
    const { EXCHANGE, LP_TOKEN } = await FACTORY.getExchange(SIENNA.asCustomToken, SSCRT.asCustomToken)
    await agent.bundle(async agent=>{
      await SIENNA.tx(agent).setViewingKey("")
      await LP_TOKEN.tx(agent).setViewingKey("")
    })
    console.info(bold('Initial LP token balance:'), await LP_TOKEN.q().balance(agent.address, ""))
    await agent.bundle(async agent=>{
      await SIENNA.tx(agent).increaseAllowance("1000", EXCHANGE.address)
      await SSCRT.tx(agent).increaseAllowance("1000", EXCHANGE.address)
      await EXCHANGE.tx(agent).add_liquidity({
        token_0: SIENNA.asCustomToken,
        token_1: SSCRT.asCustomToken
      }, "1000", "1000")
    })
    console.info(bold('New LP token balance:'), await LP_TOKEN.q().balance(agent.address, ""))
    return { EXCHANGE, LP_TOKEN, SIENNA }
  },
  stakeLPTokens: v => async function integrationTestStakeLPTokens ({
    agent, deployment,
    SIENNA  = deployment.getThe('SIENNA', new SiennaSNIP20Contract({agent})),
    RPT     = deployment.getThe('RPT',    new RPTContract({agent})),
    SSSSS   = deployment.getThe(`Rewards[${v}].SSSSS`,        new RewardsContract[v]({agent})),
    REWARDS = deployment.getThe(`Rewards[${v}].SIENNA-SSCRT`, new RewardsContract[v]({agent}))
  }) {
    console.info(bold('Initial SIENNA balance:'), await SIENNA.q().balance(agent.address, ""))
    const LP_TOKEN = await REWARDS.lpToken()
    await agent.bundle(async agent=>{
      await LP_TOKEN.tx(agent).increaseAllowance("100", REWARDS.address)
      await REWARDS.tx(agent).lock("100")
      await SIENNA.tx(agent).increaseAllowance("100", SSSSS.address)
      await SSSSS.tx(agent).lock("100")
    })
    console.info(bold('SIENNA balance after staking:'), await SIENNA.q().balance(agent.address, ""))
    await agent.bundle(async agent=>{
      await RPT.tx(agent).vest()
      await SSSSS.tx(agent).set_viewing_key("")
      await REWARDS.tx(agent).set_viewing_key("")
    })
    console.log(await SSSSS.q(agent).pool_info())
    console.log(await SSSSS.q(agent).user_info())
    try {
      await SSSSS.tx(agent).claim()
    } catch (e) {
      console.error(bold(`Could not claim from SSSSS ${v}:`, e.message))
    }
    console.log(await REWARDS.q(agent).pool_info())
    console.log(await REWARDS.q(agent).user_info())
    try {
      await REWARDS.tx(agent).claim()
    } catch (e) {
      console.error(bold(`Could not claim from Rewards ${v}:`, e.message))
    }
    console.info(bold('SIENNA balance after claiming:'), await SIENNA.q().balance(agent.address, ""))
  },
  vestV3: async function integrationTestVestV3 ({
    agent, deployment,
    RPT     = deployment.getThe('RPT',new RPTContract({agent})),
    SSSSS   = deployment.getThe(`Rewards[v3].SSSSS`,        new RewardsContract['v3']({agent})),
    REWARDS = deployment.getThe(`Rewards[v3].SIENNA-SSCRT`, new RewardsContract['v3']({agent}))
  }) {
    console.info('Before vest')
    console.log(await SSSSS.q(agent).user_info())
    console.log(await REWARDS.q(agent).user_info())
    await RPT.tx(agent).vest()
    console.info('After vest')
    console.log(await SSSSS.q(agent).user_info())
    console.log(await REWARDS.q(agent).user_info())
    await agent.bundle(async agent=>{
      await SSSSS.tx(agent).epoch()
      await REWARDS.tx(agent).epoch()
    })
    console.info('After epoch')
    console.log(await SSSSS.q(agent).user_info())
    console.log(await REWARDS.q(agent).user_info())
    await agent.bundle(async agent=>{
      await SSSSS.tx(agent).claim()
      await REWARDS.tx(agent).claim()
    })
    console.info('After claim')
    console.log(await SSSSS.q(agent).user_info())
    console.log(await REWARDS.q(agent).user_info())
  }
}

const integrationTests = {

  1: [ Deployments.new,                           // Start in a blank deployment
       integrationTest.setup,                     // Add test user to MGMT schedule
       deployTGE,                                 // Deploy the TGE as normal
       MGMTContract.progress,                     // User's progress before claiming
       integrationTest.claim,                     // Try to claim
       MGMTContract.progress ],                   // User's progress after claiming

  2: [ Deployments.activate,                      // Use the current deployment
       AMMFactoryContract['v1'].deployAMM,        // Deploy AMM v1
       RewardsContract['v2'].deploy ],            // Deploy Rewards v2

  3: [ Deployments.activate,                      // Use the current deployment
       integrationTest.getLPTokens('v1'),         // Stake SIENNA and SSCRT to get LP tokens
       integrationTest.stakeLPTokens('v2') ],     // Stake LP tokens to get SIENNA

  4: [ Deployments.activate,                      // Use the current deployment
       AMMFactoryContract['v1'].upgradeAMM.to_v2, // Upgrade AMM v1 to v2
       RewardsContract['v2'].upgrade.to_v3,       // Upgrade Rewards from v2 to v3
       integrationTest.getLPTokens('v2'),         // Stake SIENNA and SSCRT to get LP tokens
       integrationTest.stakeLPTokens('v3') ],     // Stake LP tokens to get SIENNA

  5: [ Deployments.activate,                      // Use the current deployment
       RewardsContract['v3'].upgrade.to_v3,       // Upgrade Rewards from v3 to another v3 to test user migrations
       integrationTest.vestV3 ]                   // Vest and call epoch

}

Fadroma.command('integration test 1', ...integrationTests[1])
Fadroma.command('integration test 2', ...integrationTests[2])
Fadroma.command('integration test 3', ...integrationTests[3])
Fadroma.command('integration test 4', ...integrationTests[4])
Fadroma.command('integration test 5', ...integrationTests[5])
Fadroma.command('integration tests',
  ...integrationTests[1],
  ...integrationTests[2],
  ...integrationTests[3],
  ...integrationTests[4],
  ...integrationTests[5])
```

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```
