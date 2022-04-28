# Deploying contracts

|                     |                                   |
| ------------------- | --------------------------------- |
| **Entry point:**    | `pnpm deploy` or `pnpm -w deploy` |
| **Overview:**       | [./Deployment](./Deployment)      |
| **Implementation:** | [@fadroma/ops/Deploy](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Deploy.ts)              |
| **Specification:**  | [@fadroma/ops/Deploy.spec](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Deploy.spec.ts.md) |

```typescript
import Fadroma, { Console } from '@hackbg/fadroma'
const console = new Console('Deploy')
```

# Overview of build/upload/deploy workflow

```typescript
import { MigrationContext, buildAndUpload, buildAndUploadMany } from '@hackbg/fadroma'
import getSettings, { ONE_SIENNA } from '@sienna/settings'
import { versions, contracts, source, sources } from './Build'
import { canBuildAndUpload, inNewDeployment, inCurrentDeployment } from './misc'
```

The implementation of each deployment procedure is an `async function`
which takes a single argument, the `MigrationContext`, and performs
build, upload, and init/handle/query operations by means of the entries
of the migration context.

* `chain` and `agent`
* `builder` and `uploader`
* `deployment` and `prefix`

The migration context is populated by Each deploy command in this file begins by invoking
four pre-defined steps from Fadroma that populate
the `chain`, `agent`, `builder`, `uploader`, `deployment` and `prefix`
keys of the `MigrationContext` for subsequent command steps.

The `chain` and `agent` are taken from the environment (or `.env` file):
* **Env var:** `FADROMA_CHAIN`:       select mainnet, testnet, or devnet
* **Env var:** `SCRT_AGENT_MNEMONIC`: mnemonic of wallet used for deploy

The `builder` and `uploader` objects allow the source code of the contracts
to be reproducibly compiled and uploaded to the selected blockchain.
* **See [./Build](./Build.ts.md)** for info about how contracts are built.
* **See [./Upload](./Upload.ts.md)** for info about how contracts are uploaded.

# Pre-configured command steps

> **See also:** [How commands work](./README#how-commands-work)

```typescript
const Deployment = Fadroma.Deploy
const Deploy  = {}
const Upgrade = {}
```

The `Sienna` object is a collection of reusable, differently pre-configured
shorthands for the deployment procedures implemented in [./Subsystems](./Subsystems).

The [deployment commands](#deployment-command-definitions) available in the CLI
can then be composed out of these shorthands.

Use the `function` syntax when defining a pre-configured command
to give it a proper `name` to be printed in the console by the command runner.

# Deploy the initial TGE

> Do this with `pnpm deploy tge`

The **Sienna TGE (Token Generation Event)** is the
core of the Sienna Platform. It contains a token (SIENNA)
and two vesting contracts:

* with a complex, permanent schedule **(MGMT, short for Management)**
* one with a simple, reconfigurable schedule **(RPT, short for Remaining Pool Tokens)**.

This will create a new deployment
under `/receipts/$FADROMA_CHAIN/$TIMESTAMP`,
and deploy just the TGE contracts.

```typescript
import { deployTGE } from './Subsystems/SiennaTGE'
Deploy.TGE = deployTGE

Fadroma.command('tge', ...inNewDeployment, Deploy.TGE)
```

# Deploy auxiliary vestings

**Auxiliary vestings** work like the main TGE,
but distribute a pre-existing SNIP20 token.

```typescript
import { deployVesting } from './Subsystems/Vesting'
Deploy.Vesting = function deployVesting_HEAD ({ run }) {
  return run(deployVesting)
}

Fadroma.command('vesting',
  ...canBuildAndUpload,
  deployNewOnDevnetAppendOtherwise
  Deploy.Vesting,
  Deployment.Status
)

function deployNewOnDevnetAppendOtherwise (context) {
  return context.chain.isDevnet
    ? Deployment.New(context)
    : Deployment.Append(context)
},
```

# Deploy a SNIP20 token

```typescript
import { deployToken } from './Subsystems/Token'
Fadroma.command('token', ...inCurrentDeployment, deployToken)
```

# Deploy Sienna Swap (Factory + Exchange)

This procedure takes the active deployment (containing a TGE),
attaches a new AMM Factory to it, and uses that factory to
create the AMM Exchange liquidity pools that are configured
in the project settings, as well as each pool's correspongin
LP token.

The factory is the hub of the AMM. In order to work, it needs to be configured
with the proper contract templates, so this function builds and uploads those too (if absent).

> See also: [buildAMMTemplates](./Build.ts.md#building-the-templates-for-the-amm-factory).

```typescript
import {
  deployAMM,
  deployRouter
} from './Subsystems/SiennaSwap'

Deploy.AMM = {
  Latest: function deployAMM_HEAD ({ run }) {
    return run(deployAMM, { ammVersion: 'v2', ref: 'HEAD' })
  },
  v1: function deployAMM_v1 ({ run }) {
    return run(deployAMM, { ammVersion: 'v1' })
  },
  v2: function deployAMM_v2 ({ run }) {
    return run(deployAMM, { ammVersion: 'v2' })
  },
}

Deploy.Router = deployRouter
  
Fadroma.command('amm latest', ...inCurrentDeployment, Deploy.AMM.Latest)
Fadroma.command('amm stable', ...inCurrentDeployment, Deploy.AMM.v2)
Fadroma.command('amm legacy', ...inCurrentDeployment, Deploy.AMM.v1)
Fadroma.command('router',     ...inCurrentDeployment, Deploy.Router)
```

## Upgrade Sienna Swap

This procedure takes an existing AMM and
creates a new one with the same contract templates. 
Then, it recreates all the exchanges from the
old factory in the new one.

```typescript
import {
  upgradeAMM,
  upgradeAMMFactory_v1_to_v2,
  cloneAMMExchanges_v1_to_v2
} from './Subsystems/SiennaSwap'

Upgrade.AMM = {
  v1_to_v2: function upgradeAMM_v1_to_v2 ({ run }) {
    return run(upgradeAMM, { vOld: 'v1', vNew: 'v2' })
  },
  Factory: {
    v1_to_v2: upgradeAMMFactory_v1_to_v2
  },
  Exchanges: {
    v1_to_v2: cloneAMMExchanges_v1_to_v2
  }
}

Fadroma.command('amm v1_to_v2_all',       ...inCurrentDeployment, Upgrade.AMM.v1_to_v2)
Fadroma.command('amm v1_to_v2_factory',   ...inCurrentDeployment, Upgrade.AMM.Factory.v1_to_v2)
Fadroma.command('amm v1_to_v2_exchanges', ...inCurrentDeployment, Upgrade.AMM.Exchanges.v1_to_v2)
```

# Deploy Launchpad

```typescript
import { deployLaunchpad } from './Subsystems/SiennaLaunch'
Deploy.Launchpad = deployLaunchpad
Fadroma.command('launchpad', ...inCurrentDeployment, Deploy.Launchpad)
```

# Deploy Sienna Rewards

```typescript
import { deployRewards, deployRewardPool } from './Subsystems/SiennaRewards'
Deploy.Rewards = Object.assign(
  function deployRewards_HEAD ({ run }) {
    return run(deployRewards,   { version:   'v3', adjustRPT: true, ref: 'HEAD' })
  }, {
    v2: function deployRewards_v2 ({ run }) {
      return run(deployRewards, { version:   'v2', adjustRPT: true })
    },
    v3: function deployRewards_v3({ run }) {
      return run(deployRewards, { version:   'v3', adjustRPT: true })
    }
  }
)
Deploy.RewardPool = deployRewardPool

Fadroma.command('rewards wip', ...inCurrentDeployment, Deploy.Rewards)
Fadroma.command('rewards v2',  ...inCurrentDeployment, Deploy.Rewards.v2)
Fadroma.command('rewards v3',  ...inCurrentDeployment, Deploy.Rewards.v3)
Fadroma.command('reward pool', ...inCurrentDeployment, Deploy.RewardPool)
```

## Upgrade Sienna Rewards

```typescript
Upgrade.Rewards = {
  v2_to_v3: function upgradeRewards_v2_to_v3({ run }) {
    return run(upgradeRewards, { vOld: 'v2' vNew: 'v3' })
  }
}

Fadroma.command('rewards v2_to_v3', ...inCurrentDeployment, Upgrade.Rewards.v2_to_v3)
```

# Deploy Sienna Lend

> Run with `pnpm deploy lend`

```typescript
import { deployLend } from './Subsystems/SiennaLend'
Deploy.Lend = deployLend
Fadroma.command("lend", ...inCurrentDeployment, deployLend)
```

# Deploy everything

(for different definitions of "everything")

## Latest stuff only

> Run with `pnpm deploy latest`

This is the simplest and fastest way to get up and running.
It deploys the latest development versions of everything.

```typescript
Fadroma.command('latest',
  ...inNewDeployment,
  Deploy.TGE,
  Deploy.AMM.Latest,
  Deploy.Rewards,
  Deploy.Router,
  Deploy.Lend,
  Deploy.Launchpad,
  Deployment.Status
)
```

## Full history of the deployment

> Run with `pnpm deploy all`

This is most faithful to production.
As blockchain deployments are append-only,
this goes through deploying the old versions
of contracts, then upgrading them to the latest
development versions.

```typescript
Fadroma.command('history',
  ...inNewDeployment,
  Deploy.TGE,
  Deploy.AMM.v1,
  Deploy.Rewards.v2,
  Deploy.Router,
  Deployment.Status,
  Upgrade.AMM.Factory.v1_to_v2,
  Upgrade.AMM.Exchanges.v1_to_v2,
  Upgrade.Rewards.v2_to_v3,
  Deployment.Status,
  Deploy.Lend,
  Deployment.Status
)
```

### Add AMM+Rewards on top of existing TGE

> Run with `pnpm deploy sans-tge`

The TGE is the most stable part of the project.
If you have a deployment containing a TGE (such as created by `pnpm deploy tge`)
and want to iterate on deploying the rest, this command makes it faster
by about 20 seconds.

```typescript
Fadroma.command('sans-tge',
  ...inCurrentDeployment,
  Deploy.AMM.v1,
  Deploy.Rewards.v2,
  Deployment.Status,
  Upgrade.AMM.Factory.v1_to_v2,
  Upgrade.AMM.Exchanges.v1_to_v2,
  Upgrade.Rewards.v2_to_v3,
  Deployment.Status
)
```

### Deploy latest AMM on top of existing TGE

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

```typescript
Fadroma.command('amm v2',
  ...inCurrentDeployment,
  Deploy.AMM.v2
)
```

## Legacy deployment (circa January 2022)

This contains the first live versions of TGE, AMM, and Rewards.

> Rewards start from v2 because v1.0.0 was a dead end and
> when deploying v2.0.0 as "public v1", there was a false start.

Use as basis to test the AMM v2 + Rewards v3 upgrade.

```typescript
Fadroma.command('legacy',
  ...inNewDeployment,
  Deploy.TGE,
  Deployment.Status,
  Deploy.AMM.v1,
  Deployment.Status,
  Deploy.Rewards.v2,
  Deployment.Status
)
```

## Entry point

```typescript
export default Fadroma.module(import.meta.url)
```
