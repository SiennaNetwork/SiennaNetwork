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
  Deployments.activate,
  deployTGE)
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

```typescript
import { schedule } from '@sienna/settings'
import { AMMSNIP20Contract } from '@sienna/amm'
Fadroma.command('integration test',
  // Start in a blank deployment
  Deployments.new,
  // Add the user to the MGMT schedule so that they
  // get an initial SIENNA balance without having to vest RPT.
  async function integrationTestSetup ({ chain: { isLocalnet }, agent: { address } }) {
    if (!isLocalnet) {
      throw new Error('@sienna/mgmt: This command is for localnet only.')
    }
    const scheduleMod = JSON.parse(JSON.stringify(schedule))
    console.warn('Redirecting MintingPool/LPF to admin balance. Only run this on localnet.')
    scheduleMod.pools[5].accounts[0].address = address
    return { schedule: scheduleMod }
  },
  // Deploy SIENNA, MGMT, RPT
  deployTGE,
  // Query MGMT status before claim
  MGMTContract.progress,
  // Claim
  async function integrationTestClaim ({
    agent,deployment,
    MGMT = deployment.getThe('MGMT', new MGMTContract({agent}))
  }) {
    console.warn('Integration test: claiming from LPF')
    await MGMT.tx().claim()
  },
  // Query MGMT status after claim
  MGMTContract.progress,
  // Deploy AMM
  AMMFactoryContract.v1.deployAMM,
  // Stake SIENNA and SSCRT
  async function getLPTokens ({
    agent, deployment,
    FACTORY = deployment.getThe('AMM[v1].Factory',   new AMMFactoryContract['v1']({agent}))
    SIENNA  = deployment.getThe('SIENNA',            new SiennaSNIP20Contract({agent})),
    SSCRT   = deployment.getThe('Placeholder.sSCRT', new AMMSNIP20Contract({agent})),
  }) {
    const { EXCHANGE, LP_TOKEN } = await FACTORY.getExchange(SIENNA.asCustomToken, SSCRT.asCustomToken)
    await LP_TOKEN.tx().setViewingKey("")
    console.info(bold('Initial LP token balance:'), await LP_TOKEN.q().balance(agent.address, ""))
    console.info(bold("Increase SIENNA allowance..."))
    await SIENNA.tx().increaseAllowance("1000", EXCHANGE.address)
    console.info(bold("Increase SSCRT allowance..."))
    await SSCRT.tx().increaseAllowance("1000", EXCHANGE.address)
    console.info(bold("Lock SIENNA+SSCRT into liquidity pool..."))
    await EXCHANGE.tx().add_liquidity({
      token_0:SIENNA.asCustomToken,
      token_1:SSCRT.asCustomToken
    }, "1000", "1000")
    console.info(bold('New LP token balance:'), await LP_TOKEN.q().balance(agent.address, ""))
    return { EXCHANGE, LP_TOKEN }
  },
  RewardsContract.v2.deploy,
  async function stakeLPTokens ({
    agent, deployment,
    LP_TOKEN,
    SSSSS   = deployment.getThe('Rewards[v2].SSSSS',        new RewardsContract['v2']({agent})),
    REWARDS = deployment.getThe('Rewards[v2].SIENNA-SSCRT', new RewardsContract['v2']({agent}))
  }) {
    await LP_TOKEN.tx().increaseAllowance("1000", REWARDS.address)
    await REWARDS.tx().deposit("1000")
  },
  Deployments.status,
  AMMFactoryContract.v1.upgradeAMM.to_v2,
  RewardsContract.v2.upgrade.to_v3,
  Deployments.status)
```

## Entry point

```typescript
export default Fadroma.module(import.meta.url)
```
